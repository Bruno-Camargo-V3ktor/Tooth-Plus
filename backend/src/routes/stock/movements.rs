//! # Movimentações e Ajustes de Estoque (Backend)
//!
//! Controla as entradas de compras, saídas manuais, perdas, devoluções e
//! histórico detalhado de movimentações no estoque.

use super::{
    clinic_record_id, parse_movement_type, parse_record_id, DbStockMovementRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{post, web, HttpResponse};
use shared::stock::{CreateStockMovementRequest, MovementType};
use surrealdb::types::ToSql;

/// Registra uma movimentação (entrada, saída, perda ou ajuste) atualizando o saldo atômico do item.
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
        UPDATE inventory_item SET
            current_stock = current_stock + $qty_change,
            updated_at = time::now()
        WHERE id = $item_id;

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
    ";

    let mut res = db
        .query(query)
        .bind(("item_id", item_rec_id))
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




    let created: Option<DbStockMovementRow> = res.take(1).unwrap_or(None);

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
