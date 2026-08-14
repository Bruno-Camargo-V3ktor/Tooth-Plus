use actix_web::{HttpResponse, delete, get, post, put, web};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use shared::files::FileUploadRequest;
use shared::stock::{
    CreateInventoryItemRequest, CreateStockMovementRequest, EquipmentStatus, InventoryItem,
    ItemType, MovementType, StockAlertItem, StockAlertSeverity, StockAlertType, StockKPIs,
    StockQuery, StockResponse, UpdateInventoryItemRequest,
};
use std::sync::Arc;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use crate::storage::StorageProvider;

#[derive(Deserialize, Debug, SurrealValue)]
struct DbInventoryItemRow {
    id: RecordId,
    clinic_id: RecordId,
    item_type: String,
    name: String,
    unit_type: String,
    current_stock: i32,
    min_stock: i32,
    cost_price_cents: i64,
    manufacturer: Option<String>,
    #[serde(default)]
    attachments: Vec<String>,
    expiration_date: Option<DateTime<Utc>>,
    batch_number: Option<String>,
    serial_number: Option<String>,
    warranty_until: Option<DateTime<Utc>>,
    next_maintenance_date: Option<DateTime<Utc>>,
    equipment_status: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbStockMovementRow {
    id: RecordId,
    item_id: RecordId,
    clinic_id: RecordId,
    user_id: Option<RecordId>,
    quantity_change: i32,
    movement_type: String,
    unit_cost_cents: Option<i64>,
    invoice_number: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
}

fn clinic_record_id(id: &str) -> String {
    if id.starts_with("clinic:") {
        id.to_string()
    } else {
        format!("clinic:{}", id)
    }
}

fn parse_item_type(s: &str) -> ItemType {
    match s {
        "chemical" => ItemType::Chemical,
        "equipment" => ItemType::Equipment,
        _ => ItemType::Material,
    }
}

fn parse_equipment_status(s: &str) -> EquipmentStatus {
    match s {
        "in_maintenance" => EquipmentStatus::InMaintenance,
        "broken" => EquipmentStatus::Broken,
        _ => EquipmentStatus::Active,
    }
}

fn parse_movement_type(s: &str) -> MovementType {
    match s {
        "manual_out" => MovementType::ManualOut,
        "appointment_out" => MovementType::AppointmentOut,
        "adjustment" => MovementType::Adjustment,
        "loss" => MovementType::Loss,
        _ => MovementType::PurchaseIn,
    }
}

fn map_db_item(row: DbInventoryItemRow) -> InventoryItem {
    InventoryItem {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        item_type: parse_item_type(&row.item_type),
        name: row.name,
        unit_type: row.unit_type,
        current_stock: row.current_stock,
        min_stock: row.min_stock,
        cost_price_cents: row.cost_price_cents,
        manufacturer: row.manufacturer,
        attachments: row.attachments,
        expiration_date: row.expiration_date.map(|d| d.to_rfc3339()),
        batch_number: row.batch_number,
        serial_number: row.serial_number,
        warranty_until: row.warranty_until.map(|d| d.to_rfc3339()),
        next_maintenance_date: row.next_maintenance_date.map(|d| d.to_rfc3339()),
        equipment_status: row.equipment_status.as_deref().map(parse_equipment_status),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.map(|d| d.to_rfc3339()),
    }
}

fn calculate_alerts(items: &[InventoryItem]) -> Vec<StockAlertItem> {
    let mut alerts = Vec::new();
    let now = Utc::now();
    let thirty_days_future = now + Duration::days(30);

    for item in items {
        if item.item_type != ItemType::Equipment && item.current_stock <= item.min_stock {
            let is_critical = item.current_stock <= (item.min_stock / 2);
            alerts.push(StockAlertItem {
                id: format!("alert_low_{}", item.id),
                item_id: item.id.clone(),
                item_name: item.name.clone(),
                item_type: item.item_type,
                alert_type: StockAlertType::LowStock,
                severity: if is_critical {
                    StockAlertSeverity::Critical
                } else {
                    StockAlertSeverity::Warning
                },
                title: "Estoque Abaixo do Mínimo".into(),
                message: format!(
                    "Item '{}' atingiu nível de alerta (Estoque: {} {}, Mínimo: {} {}).",
                    item.name, item.current_stock, item.unit_type, item.min_stock, item.unit_type
                ),
                current_value: format!("{} {}", item.current_stock, item.unit_type),
                target_value: format!("{} {}", item.min_stock, item.unit_type),
            });
        }

        if item.item_type == ItemType::Chemical {
            if let Some(ref exp_str) = item.expiration_date {
                if let Ok(exp_dt) = DateTime::parse_from_rfc3339(exp_str) {
                    let exp_utc = exp_dt.with_timezone(&Utc);
                    if exp_utc < now {
                        alerts.push(StockAlertItem {
                            id: format!("alert_exp_over_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::Expired,
                            severity: StockAlertSeverity::Critical,
                            title: "Produto Vencido".into(),
                            message: format!(
                                "Produto químico '{}' venceu em {}. Descarte ou reposição necessária.",
                                item.name,
                                exp_utc.format("%d/%m/%Y")
                            ),
                            current_value: "Vencido".into(),
                            target_value: exp_utc.format("%d/%m/%Y").to_string(),
                        });
                    } else if exp_utc <= thirty_days_future {
                        let days_left = (exp_utc - now).num_days();
                        alerts.push(StockAlertItem {
                            id: format!("alert_exp_soon_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::ExpiringSoon,
                            severity: StockAlertSeverity::Warning,
                            title: "Validade Próxima".into(),
                            message: format!(
                                "Produto químico '{}' vence em {} dias ({}).",
                                item.name,
                                days_left,
                                exp_utc.format("%d/%m/%Y")
                            ),
                            current_value: format!("{} dias restantes", days_left),
                            target_value: exp_utc.format("%d/%m/%Y").to_string(),
                        });
                    }
                }
            }
        }

        if item.item_type == ItemType::Equipment {
            if let Some(ref maint_str) = item.next_maintenance_date {
                if let Ok(maint_dt) = DateTime::parse_from_rfc3339(maint_str) {
                    let maint_utc = maint_dt.with_timezone(&Utc);
                    if maint_utc < now {
                        alerts.push(StockAlertItem {
                            id: format!("alert_maint_over_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::MaintenanceOverdue,
                            severity: StockAlertSeverity::Critical,
                            title: "Manutenção Preventiva Atrasada".into(),
                            message: format!(
                                "Equipamento '{}' estava agendado para revisão em {}.",
                                item.name,
                                maint_utc.format("%d/%m/%Y")
                            ),
                            current_value: "Atrasada".into(),
                            target_value: maint_utc.format("%d/%m/%Y").to_string(),
                        });
                    } else if maint_utc <= thirty_days_future {
                        let days_left = (maint_utc - now).num_days();
                        alerts.push(StockAlertItem {
                            id: format!("alert_maint_soon_{}", item.id),
                            item_id: item.id.clone(),
                            item_name: item.name.clone(),
                            item_type: item.item_type,
                            alert_type: StockAlertType::MaintenanceDue,
                            severity: StockAlertSeverity::Warning,
                            title: "Revisão Preventiva Próxima".into(),
                            message: format!(
                                "Equipamento '{}' deve passar por revisão preventiva em {} dias ({}).",
                                item.name,
                                days_left,
                                maint_utc.format("%d/%m/%Y")
                            ),
                            current_value: format!("Em {} dias", days_left),
                            target_value: maint_utc.format("%d/%m/%Y").to_string(),
                        });
                    }
                }
            }

            if item.equipment_status == Some(EquipmentStatus::InMaintenance) {
                alerts.push(StockAlertItem {
                    id: format!("alert_eq_in_maint_{}", item.id),
                    item_id: item.id.clone(),
                    item_name: item.name.clone(),
                    item_type: item.item_type,
                    alert_type: StockAlertType::MaintenanceDue,
                    severity: StockAlertSeverity::Warning,
                    title: "Equipamento em Manutenção".into(),
                    message: format!(
                        "O equipamento '{}' está atualmente em assistência técnica.",
                        item.name
                    ),
                    current_value: "Em Manutenção".into(),
                    target_value: "Aguardando Retorno".into(),
                });
            } else if item.equipment_status == Some(EquipmentStatus::Broken) {
                alerts.push(StockAlertItem {
                    id: format!("alert_eq_broken_{}", item.id),
                    item_id: item.id.clone(),
                    item_name: item.name.clone(),
                    item_type: item.item_type,
                    alert_type: StockAlertType::MaintenanceOverdue,
                    severity: StockAlertSeverity::Critical,
                    title: "Equipamento Inoperante".into(),
                    message: format!(
                        "O equipamento '{}' está registrado como danificado/inoperante.",
                        item.name
                    ),
                    current_value: "Inoperante".into(),
                    target_value: "Reparo Urgente".into(),
                });
            }
        }
    }

    alerts
}

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

#[post("/stock/{id}/movement")]
pub async fn create_stock_movement(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<CreateStockMovementRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let item_id_str = path.into_inner();
    let item_rec_id = parse_record_id("inventory_item", &item_id_str);
    let clinic_str = clinic_record_id(&req.clinic_id);
    let clinic_rec_id = parse_record_id("clinic", &clinic_str);

    let has_movement = check_permission(&db, &auth.id, &clinic_str, "stock:movement")
        .await
        .unwrap_or(false);
    let has_write = check_permission(&db, &auth.id, &clinic_str, "stock:write")
        .await
        .unwrap_or(false);

    if !has_movement && !has_write {
        return Err(ApiError::Forbidden(
            "Sem permissão para registrar movimentações de estoque.".into(),
        ));
    }

    let mov_type_str = match req.movement_type {
        MovementType::PurchaseIn => "purchase_in",
        MovementType::ManualOut => "manual_out",
        MovementType::AppointmentOut => "appointment_out",
        MovementType::Adjustment => "adjustment",
        MovementType::Loss => "loss",
    };

    let auth_rec_id = parse_record_id("user", &auth.id);

    let query = "
        BEGIN TRANSACTION;

        LET $item = (SELECT * FROM $item_id);
        IF $item[0] == NONE {
            THROW 'Item de estoque não encontrado.';
        };

        UPDATE $item_id SET
            current_stock = current_stock + $qty_change,
            updated_at = time::now();

        CREATE stock_movement CONTENT {
            item_id: $item_id,
            clinic_id: $cid,
            user_id: $uid,
            quantity_change: $qty_change,
            movement_type: $mov_type,
            unit_cost_cents: $cost,
            invoice_number: $invoice,
            notes: $notes,
            created_at: time::now()
        };

        COMMIT TRANSACTION;
    ";

    let mut res = db
        .query(query)
        .bind(("item_id", item_rec_id.clone()))
        .bind(("cid", clinic_rec_id))
        .bind(("uid", auth_rec_id))
        .bind(("qty_change", req.quantity_change))
        .bind(("mov_type", mov_type_str))
        .bind(("cost", req.unit_cost_cents))
        .bind(("invoice", req.invoice_number.clone()))
        .bind(("notes", req.notes.clone()))
        .await
        .map_err(|e| {
            ApiError::Internal(format!("Erro ao registrar movimentação de estoque: {}", e))
        })?;

    let created: Option<DbStockMovementRow> = res.take(3).map_err(|e| {
        ApiError::Internal(format!("Erro ao ler comprovante de movimentação: {}", e))
    })?;

    let Some(mov) = created else {
        return Ok(HttpResponse::Ok().body("Movimentação registrada com sucesso."));
    };

    Ok(HttpResponse::Created().json(shared::stock::StockMovement {
        id: mov.id.to_sql(),
        item_id: mov.item_id.to_sql(),
        item_name: None,
        clinic_id: mov.clinic_id.to_sql(),
        user_id: mov.user_id.map(|u| u.to_sql()),
        user_name: None,
        quantity_change: mov.quantity_change,
        movement_type: parse_movement_type(&mov.movement_type),
        unit_cost_cents: mov.unit_cost_cents,
        invoice_number: mov.invoice_number,
        notes: mov.notes,
        created_at: mov.created_at.to_rfc3339(),
    }))
}

#[post("/stock/{clinic_id}/upload")]
pub async fn upload_stock_document(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<FileUploadRequest>,
    db: web::Data<Db>,
    storage: web::Data<Arc<dyn StorageProvider>>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let clinic_str = clinic_record_id(&clinic_id);

    let has_write = check_permission(&db, &auth.id, &clinic_str, "stock:write")
        .await
        .unwrap_or(false);
    let has_movement = check_permission(&db, &auth.id, &clinic_str, "stock:movement")
        .await
        .unwrap_or(false);

    if !has_write && !has_movement {
        return Err(ApiError::Forbidden(
            "Sem permissão para anexar documentos no estoque.".into(),
        ));
    }

    let data = req.into_inner();
    let ext = data.filename.rsplit('.').next().unwrap_or("pdf");
    let file_url = storage
        .upload_file("stock/documents", ext, &data.base64_content)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro no upload: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "url": file_url,
        "filename": data.filename
    })))
}
