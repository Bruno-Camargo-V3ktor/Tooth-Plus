//! # Gestão de Itens de Estoque e Alertas (Backend)
//!
//! Controla a consulta consolidada com KPIs e alertas inteligentes, cadastro de
//! novos insumos/equipamentos, atualização de níveis e exclusão de itens.

use super::{
    calculate_alerts, clinic_record_id, map_db_item, parse_movement_type, parse_record_id,
    DbInventoryItemRow, DbStockMovementRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use shared::stock::{
    CreateInventoryItemRequest, EquipmentStatus, InventoryItem, ItemType, StockKPIs,
    StockQuery, StockResponse, StockAlertType, UpdateInventoryItemRequest,
};
use surrealdb::types::ToSql;

/// Retorna a lista de itens de estoque, alertas calculados, movimentações recentes e KPIs.
#[get("/stock")]
pub async fn get_stock_data(
    auth: AuthenticatedUser,
    query: web::Query<StockQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_str = clinic_record_id(&query.clinic_id);
    let clinic_rec_id = parse_record_id("clinic", &clinic_str);

    let has_read = check_permission(&db, &auth.id, &clinic_str, "stock:read")
        .await
        .unwrap_or(false);

    if !has_read {
        return Err(ApiError::Forbidden(
            "Sem permissão para visualizar o estoque desta clínica.".into(),
        ));
    }

    let mut query_str = "SELECT * FROM inventory_item WHERE clinic_id = $cid".to_string();
    if let Some(ref t) = query.item_type {
        if !t.is_empty() && t != "all" {
            query_str.push_str(" AND item_type = $item_type");
        }
    }
    query_str.push_str(" ORDER BY name ASC");

    let mut sql_req = db.query(query_str).bind(("cid", clinic_rec_id.clone()));
    if let Some(ref t) = query.item_type {
        if !t.is_empty() && t != "all" {
            sql_req = sql_req.bind(("item_type", t.clone()));
        }
    }

    let mut res = sql_req
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao consultar itens de estoque: {}", e)))?;

    let db_items: Vec<DbInventoryItemRow> = res
        .take(0)
        .map_err(|e| ApiError::Internal(format!("Erro ao ler itens de estoque: {}", e)))?;

    let items: Vec<InventoryItem> = db_items.into_iter().map(map_db_item).collect();

    let mut mov_res = db
        .query(
            "SELECT * FROM stock_movement WHERE clinic_id = $cid ORDER BY created_at DESC LIMIT 30",
        )
        .bind(("cid", clinic_rec_id))
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao consultar movimentações: {}", e)))?;

    let db_movements: Vec<DbStockMovementRow> = mov_res.take(0).unwrap_or_default();

    let recent_movements: Vec<shared::stock::StockMovement> = db_movements
        .into_iter()
        .map(|m| shared::stock::StockMovement {
            id: m.id.to_sql(),
            item_id: m.item_id.to_sql(),
            item_name: None,
            clinic_id: m.clinic_id.to_sql(),
            user_id: m.user_id.map(|u| u.to_sql()),
            user_name: None,
            quantity_change: m.quantity_change,
            movement_type: parse_movement_type(&m.movement_type),
            unit_cost_cents: m.unit_cost_cents,
            invoice_number: m.invoice_number,
            notes: m.notes,
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    let alerts = calculate_alerts(&items);

    let materials_count = items
        .iter()
        .filter(|i| i.item_type == ItemType::Material)
        .count();
    let chemicals_count = items
        .iter()
        .filter(|i| i.item_type == ItemType::Chemical)
        .count();
    let equipments_count = items
        .iter()
        .filter(|i| i.item_type == ItemType::Equipment)
        .count();
    let total_inventory_value_cents: i64 = items
        .iter()
        .map(|i| (i.current_stock.max(0) as i64) * i.cost_price_cents)
        .sum();

    let low_stock_alerts_count = alerts
        .iter()
        .filter(|a| a.alert_type == StockAlertType::LowStock)
        .count();
    let expiring_alerts_count = alerts
        .iter()
        .filter(|a| {
            a.alert_type == StockAlertType::ExpiringSoon || a.alert_type == StockAlertType::Expired
        })
        .count();
    let maintenance_alerts_count = alerts
        .iter()
        .filter(|a| {
            a.alert_type == StockAlertType::MaintenanceDue
                || a.alert_type == StockAlertType::MaintenanceOverdue
        })
        .count();

    let kpis = StockKPIs {
        total_items_count: items.len(),
        materials_count,
        chemicals_count,
        equipments_count,
        total_inventory_value_cents,
        low_stock_alerts_count,
        expiring_alerts_count,
        maintenance_alerts_count,
    };

    Ok(HttpResponse::Ok().json(StockResponse {
        kpis,
        items,
        alerts,
        recent_movements,
    }))
}

/// Cria um novo item no estoque com suporte a campos de rastreabilidade (lote, validade, garantia).
#[post("/stock")]
pub async fn create_stock_item(
    auth: AuthenticatedUser,
    req: web::Json<CreateInventoryItemRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_str = clinic_record_id(&req.clinic_id);
    let clinic_rec_id = parse_record_id("clinic", &clinic_str);

    let has_write = check_permission(&db, &auth.id, &clinic_str, "stock:write")
        .await
        .unwrap_or(false);

    if !has_write {
        return Err(ApiError::Forbidden(
            "Sem permissão para cadastrar itens no estoque.".into(),
        ));
    }

    let item_type_str = match req.item_type {
        ItemType::Material => "material",
        ItemType::Chemical => "chemical",
        ItemType::Equipment => "equipment",
    };

    let status_str = req.equipment_status.map(|s| match s {
        EquipmentStatus::Active => "active",
        EquipmentStatus::InMaintenance => "in_maintenance",
        EquipmentStatus::Broken => "broken",
    });

    let exp_dt = req
        .expiration_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let war_dt = req
        .warranty_until
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let maint_dt = req
        .next_maintenance_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let query = "
        CREATE inventory_item CONTENT {
            clinic_id: $cid,
            item_type: $item_type,
            name: $name,
            unit_type: $unit_type,
            current_stock: $current_stock,
            min_stock: $min_stock,
            cost_price_cents: $cost_price_cents,
            manufacturer: $manufacturer,
            attachments: $attachments,
            expiration_date: $expiration_date,
            batch_number: $batch_number,
            serial_number: $serial_number,
            warranty_until: $warranty_until,
            next_maintenance_date: $next_maintenance_date,
            equipment_status: $equipment_status,
            created_at: time::now()
        }
    ";

    let mut res = db
        .query(query)
        .bind(("cid", clinic_rec_id.clone()))
        .bind(("item_type", item_type_str))
        .bind(("name", req.name.clone()))
        .bind(("unit_type", req.unit_type.clone()))
        .bind(("current_stock", req.current_stock))
        .bind(("min_stock", req.min_stock))
        .bind(("cost_price_cents", req.cost_price_cents))
        .bind(("manufacturer", req.manufacturer.clone()))
        .bind(("attachments", req.attachments.clone()))
        .bind(("expiration_date", exp_dt))
        .bind(("batch_number", req.batch_number.clone()))
        .bind(("serial_number", req.serial_number.clone()))
        .bind(("warranty_until", war_dt))
        .bind(("next_maintenance_date", maint_dt))
        .bind(("equipment_status", status_str))
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao criar item de estoque: {}", e)))?;

    let created: Option<DbInventoryItemRow> = res
        .take(0)
        .map_err(|e| ApiError::Internal(format!("Erro ao recuperar item criado: {}", e)))?;

    let Some(item_row) = created else {
        return Err(ApiError::Internal(
            "Falha ao registrar item no banco.".into(),
        ));
    };

    if req.current_stock > 0 {
        let auth_rec_id = parse_record_id("user", &auth.id);
        let _ = db
            .query(
                "
                CREATE stock_movement CONTENT {
                    item_id: $item_id,
                    clinic_id: $cid,
                    user_id: $uid,
                    quantity_change: $qty,
                    movement_type: 'purchase_in',
                    unit_cost_cents: $cost,
                    notes: 'Estoque inicial cadastrado',
                    created_at: time::now()
                }
            ",
            )
            .bind(("item_id", item_row.id.clone()))
            .bind(("cid", clinic_rec_id))
            .bind(("uid", auth_rec_id))
            .bind(("qty", req.current_stock))
            .bind(("cost", req.cost_price_cents))
            .await;
    }

    Ok(HttpResponse::Created().json(map_db_item(item_row)))
}

/// Atualiza as propriedades e quantidade cadastrada de um item de estoque.
#[put("/stock/{id}")]
pub async fn update_stock_item(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<UpdateInventoryItemRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let item_id_str = path.into_inner();
    let item_rec_id = parse_record_id("inventory_item", &item_id_str);
    let clinic_str = clinic_record_id(&req.clinic_id);

    let has_write = check_permission(&db, &auth.id, &clinic_str, "stock:write")
        .await
        .unwrap_or(false);

    if !has_write {
        return Err(ApiError::Forbidden(
            "Sem permissão para atualizar itens no estoque.".into(),
        ));
    }

    let item_type_str = match req.item_type {
        ItemType::Material => "material",
        ItemType::Chemical => "chemical",
        ItemType::Equipment => "equipment",
    };

    let status_str = req.equipment_status.map(|s| match s {
        EquipmentStatus::Active => "active",
        EquipmentStatus::InMaintenance => "in_maintenance",
        EquipmentStatus::Broken => "broken",
    });

    let exp_dt = req
        .expiration_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let war_dt = req
        .warranty_until
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let maint_dt = req
        .next_maintenance_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let query = "
        UPDATE $item_id MERGE {
            item_type: $item_type,
            name: $name,
            unit_type: $unit_type,
            current_stock: $current_stock,
            min_stock: $min_stock,
            cost_price_cents: $cost_price_cents,
            manufacturer: $manufacturer,
            attachments: $attachments,
            expiration_date: $expiration_date,
            batch_number: $batch_number,
            serial_number: $serial_number,
            warranty_until: $warranty_until,
            next_maintenance_date: $next_maintenance_date,
            equipment_status: $equipment_status,
            updated_at: time::now()
        }
    ";

    let mut res = db
        .query(query)
        .bind(("item_id", item_rec_id))
        .bind(("item_type", item_type_str))
        .bind(("name", req.name.clone()))
        .bind(("unit_type", req.unit_type.clone()))
        .bind(("current_stock", req.current_stock))
        .bind(("min_stock", req.min_stock))
        .bind(("cost_price_cents", req.cost_price_cents))
        .bind(("manufacturer", req.manufacturer.clone()))
        .bind(("attachments", req.attachments.clone()))
        .bind(("expiration_date", exp_dt))
        .bind(("batch_number", req.batch_number.clone()))
        .bind(("serial_number", req.serial_number.clone()))
        .bind(("warranty_until", war_dt))
        .bind(("next_maintenance_date", maint_dt))
        .bind(("equipment_status", status_str))
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao atualizar item: {}", e)))?;

    let updated: Option<DbInventoryItemRow> = res
        .take(0)
        .map_err(|e| ApiError::Internal(format!("Erro ao recuperar item atualizado: {}", e)))?;

    let Some(item_row) = updated else {
        return Err(ApiError::BadRequest("Item não encontrado.".into()));
    };

    Ok(HttpResponse::Ok().json(map_db_item(item_row)))
}

/// Exclui um item cadastrado no estoque.
#[delete("/stock/{id}")]
pub async fn delete_stock_item(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<StockQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let item_id_str = path.into_inner();
    let item_rec_id = parse_record_id("inventory_item", &item_id_str);
    let clinic_str = clinic_record_id(&query.clinic_id);

    let has_delete = check_permission(&db, &auth.id, &clinic_str, "stock:delete")
        .await
        .unwrap_or(false);

    if !has_delete {
        return Err(ApiError::Forbidden(
            "Sem permissão para remover itens do estoque.".into(),
        ));
    }

    let _ = db
        .query("DELETE $item_id")
        .bind(("item_id", item_rec_id))
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao excluir item: {}", e)))?;

    Ok(HttpResponse::Ok().body("Item excluído com sucesso."))
}
