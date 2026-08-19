//! # Operações de Transações e Lançamentos Financeiros (Backend)
//!
//! Controla inserção, atualização de status/liquidação e exclusão de movimentações.

use actix_web::{delete, patch, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

use super::{
    clinic_record_id, parse_direction, parse_record_id, parse_status, transaction_record_id,
    ClinicQuery, DbAppointmentPendingRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use shared::finance::{
    CreateTransactionRequest, Transaction, TransactionDirection, TransactionStatus,
    UpdateTransactionStatusRequest,
};

#[derive(Serialize, SurrealValue)]
struct InsertTransactionDb {
    clinic_id: RecordId,
    appointment_id: Option<RecordId>,
    patient_id: Option<RecordId>,
    user_id: Option<RecordId>,
    direction: String,
    amount_cents: i64,
    description: String,
    category: String,
    status: String,
    due_date: DateTime<Utc>,
    paid_date: Option<DateTime<Utc>>,
    payment_method: Option<String>,
    installment_current: i32,
    installment_total: i32,
}

/// Cria um novo lançamento financeiro (Receita ou Despesa).
#[post("/finance")]
pub async fn create_transaction(
    auth: AuthenticatedUser,
    body: web::Json<CreateTransactionRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();
    let clinic_rec_id = clinic_record_id(&req.clinic_id);

    let has_perm = check_permission(&db, &auth.id, &clinic_rec_id, "finance:write")
        .await
        .unwrap_or(false);

    if !has_perm {
        return Err(ApiError::Forbidden(
            "Sem privilégios para registrar lançamentos financeiros.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &req.clinic_id);
    let appointment_rec = req
        .appointment_id
        .map(|id| parse_record_id("appointment", &id));
    let patient_rec = req.patient_id.map(|id| parse_record_id("patient", &id));
    let user_rec = req.user_id.map(|id| parse_record_id("user", &id));

    let due_dt = DateTime::parse_from_rfc3339(&req.due_date)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(&req.due_date, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(12, 0, 0).unwrap().and_utc())
        })
        .unwrap_or_else(|_| Utc::now());

    let paid_dt = req.paid_date.as_ref().and_then(|d| {
        DateTime::parse_from_rfc3339(d)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .map(|dt| dt.and_hms_opt(12, 0, 0).unwrap().and_utc())
            })
            .ok()
    });

    let dir_str = match req.direction {
        TransactionDirection::Expense => "expense",
        _ => "income",
    };

    let status_str = match req.status {
        TransactionStatus::Paid => "paid",
        TransactionStatus::Canceled => "canceled",
        TransactionStatus::Refunded => "refunded",
        _ => "pending",
    };

    let insert_data = InsertTransactionDb {
        clinic_id: clinic_rec,
        appointment_id: appointment_rec,
        patient_id: patient_rec,
        user_id: user_rec,
        direction: dir_str.into(),
        amount_cents: req.amount_cents,
        description: req.description.clone(),
        category: req.category.clone(),
        status: status_str.into(),
        due_date: due_dt,
        paid_date: paid_dt,
        payment_method: req.payment_method.clone(),
        installment_current: req.installment_current.unwrap_or(1),
        installment_total: req.installment_total.unwrap_or(1),
    };

    let created: Option<super::DbTransactionRow> = db
        .create("transaction")
        .content(insert_data)
        .await
        .map_err(|e| ApiError::Database(format!("Falha ao salvar transação: {}", e)))?;

    match created {
        Some(row) => Ok(HttpResponse::Created().json(Transaction {
            id: row.id.to_sql(),
            clinic_id: row.clinic_id.to_sql(),
            appointment_id: row.appointment_id.map(|id| id.to_sql()),
            patient_id: row.patient_id.map(|id| id.to_sql()),
            patient_name: None,
            user_id: row.user_id.map(|id| id.to_sql()),
            user_name: None,
            direction: parse_direction(&row.direction),
            amount_cents: row.amount_cents,
            description: row.description,
            category: row.category,
            status: parse_status(&row.status),
            due_date: row.due_date.to_rfc3339(),
            paid_date: row.paid_date.map(|d| d.to_rfc3339()),
            payment_method: row.payment_method,
            installment_current: row.installment_current,
            installment_total: row.installment_total,
            is_calculated_pending: false,
        })),
        None => Err(ApiError::Database("Erro ao retornar transação criada.".into())),
    }
}

/// Atualiza o status e liquidação de um lançamento financeiro (ou efetiva pendência da agenda).
#[patch("/finance/{id}/status")]
pub async fn update_transaction_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    body: web::Json<UpdateTransactionStatusRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let raw_id = path.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);

    let has_perm = check_permission(&db, &auth.id, &clinic_rec, "finance:update_status")
        .await
        .unwrap_or(false);
    let has_general_write = check_permission(&db, &auth.id, &clinic_rec, "finance:write")
        .await
        .unwrap_or(false);

    if !has_perm && !has_general_write {
        return Err(ApiError::Forbidden(
            "Sem permissão para atualizar status financeiro.".into(),
        ));
    }

    let req = body.into_inner();

    let status_str = match req.status {
        TransactionStatus::Paid => "paid",
        TransactionStatus::Canceled => "canceled",
        TransactionStatus::Refunded => "refunded",
        _ => "pending",
    };

    let paid_dt = if req.status == TransactionStatus::Paid {
        req.paid_date.as_ref().and_then(|d| {
            DateTime::parse_from_rfc3339(d)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                        .map(|dt| dt.and_hms_opt(12, 0, 0).unwrap().and_utc())
                })
                .ok()
        }).or_else(|| Some(Utc::now()))
    } else {
        None
    };

    // Caso 1: Lançamento originado dinamicamente da Agenda
    if raw_id.starts_with("calculated:") {
        let app_raw_id = raw_id.strip_prefix("calculated:").unwrap();
        let app_rec_id = parse_record_id("appointment", app_raw_id);

        let mut app_res = db
            .query("SELECT * FROM type::record($id)")
            .bind(("id", app_rec_id.clone()))
            .await
            .map_err(|_| ApiError::Database("Erro ao buscar agendamento associado.".into()))?;

        let app_opt: Option<DbAppointmentPendingRow> = app_res.take(0).unwrap_or(None);
        let Some(app) = app_opt else {
            return Err(ApiError::NotFound("Agendamento não encontrado para liquidação.".into()));
        };

        let insert_data = InsertTransactionDb {
            clinic_id: app.clinic_id.clone(),
            appointment_id: Some(app.id.clone()),
            patient_id: app.patient_id.clone(),
            user_id: None,
            direction: "income".into(),
            amount_cents: app.financial_amount_cents.unwrap_or(0),
            description: format!("Consulta: {}", app.title),
            category: "Procedimento Clínico".into(),
            status: status_str.into(),
            due_date: app.scheduled_for,
            paid_date: paid_dt,
            payment_method: req.payment_method.clone(),
            installment_current: 1,
            installment_total: 1,
        };

        let created: Option<super::DbTransactionRow> = db
            .create("transaction")
            .content(insert_data)
            .await
            .map_err(|e| ApiError::Database(format!("Falha ao registrar liquidação da agenda: {}", e)))?;

        return match created {
            Some(row) => Ok(HttpResponse::Ok().json(Transaction {
                id: row.id.to_sql(),
                clinic_id: row.clinic_id.to_sql(),
                appointment_id: row.appointment_id.map(|id| id.to_sql()),
                patient_id: row.patient_id.map(|id| id.to_sql()),
                patient_name: app.patient_name,
                user_id: row.user_id.map(|id| id.to_sql()),
                user_name: None,
                direction: parse_direction(&row.direction),
                amount_cents: row.amount_cents,
                description: row.description,
                category: row.category,
                status: parse_status(&row.status),
                due_date: row.due_date.to_rfc3339(),
                paid_date: row.paid_date.map(|d| d.to_rfc3339()),
                payment_method: row.payment_method,
                installment_current: row.installment_current,
                installment_total: row.installment_total,
                is_calculated_pending: false,
            })),
            None => Err(ApiError::Database("Erro ao retornar transação liquidada.".into())),
        };
    }

    // Caso 2: Transação existente na tabela `transaction`
    let tx_id = transaction_record_id(&raw_id);
    let mut update_query = format!("UPDATE {} SET status = $status", tx_id);
    if paid_dt.is_some() {
        update_query.push_str(", paid_date = $paid_date");
    } else {
        update_query.push_str(", paid_date = NONE");
    }

    if req.payment_method.is_some() {
        update_query.push_str(", payment_method = $payment_method");
    }

    let mut q = db.query(&update_query).bind(("status", status_str));
    if let Some(p_dt) = paid_dt {
        q = q.bind(("paid_date", p_dt));
    }
    if let Some(ref pm) = req.payment_method {
        q = q.bind(("payment_method", pm.clone()));
    }

    q.await
        .map_err(|e| ApiError::Database(format!("Falha ao atualizar transação: {}", e)))?;

    let mut updated_res = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", tx_id))
        .await
        .map_err(|_| ApiError::Database("Erro ao carregar transação atualizada.".into()))?;

    let updated: Option<super::DbTransactionRow> = updated_res.take(0).unwrap_or(None);

    match updated {
        Some(row) => Ok(HttpResponse::Ok().json(Transaction {
            id: row.id.to_sql(),
            clinic_id: row.clinic_id.to_sql(),
            appointment_id: row.appointment_id.map(|id| id.to_sql()),
            patient_id: row.patient_id.map(|id| id.to_sql()),
            patient_name: None,
            user_id: row.user_id.map(|id| id.to_sql()),
            user_name: None,
            direction: parse_direction(&row.direction),
            amount_cents: row.amount_cents,
            description: row.description,
            category: row.category,
            status: parse_status(&row.status),
            due_date: row.due_date.to_rfc3339(),
            paid_date: row.paid_date.map(|d| d.to_rfc3339()),
            payment_method: row.payment_method,
            installment_current: row.installment_current,
            installment_total: row.installment_total,
            is_calculated_pending: false,
        })),
        None => Err(ApiError::NotFound("Transação não encontrada.".into())),
    }
}

/// Exclui um lançamento financeiro.
#[delete("/finance/{id}")]
pub async fn delete_transaction(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let raw_id = path.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);

    let has_perm = check_permission(&db, &auth.id, &clinic_rec, "finance:delete")
        .await
        .unwrap_or(false);
    let has_general_write = check_permission(&db, &auth.id, &clinic_rec, "finance:write")
        .await
        .unwrap_or(false);

    if !has_perm && !has_general_write {
        return Err(ApiError::Forbidden(
            "Sem privilégios para excluir movimentações financeiras.".into(),
        ));
    }

    if raw_id.starts_with("calculated:") {
        let app_raw_id = raw_id.strip_prefix("calculated:").unwrap();
        let app_rec_id = parse_record_id("appointment", app_raw_id);
        db.query("UPDATE type::record($id) SET financial_amount_cents = NONE")
            .bind(("id", app_rec_id))
            .await
            .map_err(|e| ApiError::Database(format!("Falha ao remover lançamento da consulta: {}", e)))?;

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Lançamento da agenda removido com sucesso."
        })));
    }

    let tx_id = transaction_record_id(&raw_id);
    db.query("DELETE type::record($id)")
        .bind(("id", tx_id))
        .await
        .map_err(|e| ApiError::Database(format!("Falha ao excluir transação: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Lançamento financeiro removido com sucesso."
    })))
}
