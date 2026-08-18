//! # Transição de Status e Baixa Automática de Estoque/Financeiro (Backend)
//!
//! Controla as mudanças de estado do atendimento (pendente, confirmado, em atendimento,
//! concluído, cancelado ou não compareceu), executando a dedução do estoque e lançamento
//! automático de receitas e comissões no módulo financeiro quando a consulta é concluída.

use super::{
    appointment_record_id, clinic_record_id, inventory_record_id, status_to_str,
    ClinicQuery, DbAssignedRecord,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{patch, web, HttpResponse};
use shared::appointments::{AppointmentStatus, UpdateAppointmentStatusRequest};
use surrealdb::types::{SurrealValue, ToSql};

/// Atualiza o status do agendamento e executa automações financeiras e de estoque ao concluir.
#[patch("/appointments/{id}/status")]
pub async fn update_appointment_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<UpdateAppointmentStatusRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let app_id = path.into_inner();
    let data = req.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);
    let app_rec = appointment_record_id(&app_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "appointments:write")
        .await
        .unwrap_or(false)
        && !check_permission(&db, &auth.id, &clinic_rec, "agenda:write")
            .await
            .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para alterar status de agendamentos.".into(),
        ));
    }

    let status_str = status_to_str(&data.status);

    db.query(
        "UPDATE type::record($app_id) SET
            status              = $status,
            cancellation_reason = $reason",
    )
    .bind(("app_id", app_rec.clone()))
    .bind(("status", status_str))
    .bind(("reason", data.cancellation_reason.clone()))
    .await
    .map_err(|_| ApiError::Database("Falha ao atualizar status do agendamento.".into()))?;

    if data.status == AppointmentStatus::Completed {
        if let Some(ref items) = data.consumed_items {
            for item in items {
                let item_rec = inventory_record_id(&item.item_id);
                let qty_to_deduct = item.quantity_used.unwrap_or(item.quantity_planned);

                db.query(
                    "UPDATE consumes SET quantity_used = $qty
                    WHERE in = type::record($app_id) AND out = type::record($item_id)",
                )
                .bind(("app_id", app_rec.clone()))
                .bind(("item_id", item_rec.clone()))
                .bind(("qty", qty_to_deduct))
                .await
                .ok();

                if qty_to_deduct > 0 {
                    db.query(
                        "UPDATE type::record($item_id) SET current_stock = current_stock - $qty",
                    )
                    .bind(("item_id", item_rec.clone()))
                    .bind(("qty", qty_to_deduct))
                    .await
                    .ok();

                    db.query(
                        "CREATE stock_movement SET
                            item_id         = type::record($item_id),
                            quantity_change = -$qty,
                            movement_type   = 'appointment_consumed'",
                    )
                    .bind(("item_id", item_rec))
                    .bind(("qty", qty_to_deduct))
                    .await
                    .ok();
                }
            }
        }

        #[derive(serde::Deserialize, SurrealValue)]
        struct AppFinanceInfo {
            financial_amount_cents: Option<i64>,
            financial_type: Option<String>,
            title: String,
        }

        let mut fin_resp = db
            .query(
                "SELECT financial_amount_cents, financial_type, title FROM type::record($app_id)",
            )
            .bind(("app_id", app_rec.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao verificar dados financeiros.".into()))?;

        let fin_info: Option<AppFinanceInfo> = fin_resp.take(0).unwrap_or(None);

        if let Some(fi) = fin_info {
            if let (Some(amount), Some(ft)) = (fi.financial_amount_cents, fi.financial_type) {
                if amount > 0 && ft == "income" {
                    db.query(
                        "CREATE financial_transaction SET
                            clinic_id      = type::record($clinic_id),
                            appointment_id = type::record($app_id),
                            amount_cents   = $amount,
                            direction      = 'income',
                            status         = 'completed',
                            category       = 'appointment_revenue',
                            description    = $desc",
                    )
                    .bind(("clinic_id", clinic_rec.clone()))
                    .bind(("app_id", app_rec.clone()))
                    .bind(("amount", amount))
                    .bind(("desc", format!("Receita de Atendimento: {}", fi.title)))
                    .await
                    .ok();

                    let mut assigned_resp = db
                        .query(
                            "SELECT
                                out              AS user_id,
                                role_in_appointment,
                                split_percentage
                            FROM assigned_to
                            WHERE in = type::record($app_id) AND split_percentage > 0",
                        )
                        .bind(("app_id", app_rec.clone()))
                        .await
                        .unwrap();

                    let assigned: Vec<DbAssignedRecord> = assigned_resp.take(0).unwrap_or_default();

                    for a in assigned {
                        let commission_cents = (amount * a.split_percentage as i64) / 100;
                        if commission_cents > 0 {
                            db.query(
                                "CREATE financial_transaction SET
                                    clinic_id      = type::record($clinic_id),
                                    appointment_id = type::record($app_id),
                                    user_id        = type::record($user_id),
                                    amount_cents   = $amount,
                                    direction      = 'expense',
                                    status         = 'pending',
                                    category       = 'commission',
                                    description    = $desc",
                            )
                            .bind(("clinic_id", clinic_rec.clone()))
                            .bind(("app_id", app_rec.clone()))
                            .bind(("user_id", a.user_id.to_sql()))
                            .bind(("amount", commission_cents))
                            .bind((
                                "desc",
                                format!("Comissão ({}%): {}", a.split_percentage, fi.title),
                            ))
                            .await
                            .ok();
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json("Status atualizado com sucesso."))
}
