//! # Operações de Transações e Lançamentos Financeiros (Backend)
//!
//! Controla inserção, atualização de status/liquidação, pagamentos parciais e exclusão de movimentações.

use actix_web::{delete, patch, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

use super::{
    clinic_record_id, map_transaction, parse_direction, parse_record_id, parse_status,
    transaction_record_id, ClinicQuery, DbAppointmentPendingRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use shared::finance::{
    CreateTransactionRequest, RegisterPaymentRequest, Transaction, TransactionDirection,
    TransactionPaymentEntry, TransactionStatus, UpdateTransactionStatusRequest,
};

#[derive(Serialize, SurrealValue)]
struct InsertTransactionDb {
    clinic_id: RecordId,
    appointment_id: Option<RecordId>,
    patient_id: Option<RecordId>,
    user_id: Option<RecordId>,
    treatment_plan_id: Option<RecordId>,
    direction: String,
    amount_cents: i64,
    paid_amount_cents: i64,
    description: String,
    category: String,
    status: String,
    due_date: DateTime<Utc>,
    paid_date: Option<DateTime<Utc>>,
    payment_method: Option<String>,
    payments: Vec<serde_json::Value>,
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
    let treatment_plan_rec = req
        .treatment_plan_id
        .map(|id| parse_record_id("patient_treatment_plan", &id));

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
        TransactionStatus::Partial => "partial",
        TransactionStatus::Canceled => "canceled",
        TransactionStatus::Refunded => "refunded",
        _ => "pending",
    };

    let paid_amount_cents = if req.status == TransactionStatus::Paid {
        req.amount_cents
    } else {
        0
    };

    let initial_payments = if req.status == TransactionStatus::Paid && paid_amount_cents > 0 {
        let entry = TransactionPaymentEntry {
            id: uuid::Uuid::new_v4().to_string(),
            paid_at: Utc::now().to_rfc3339(),
            amount_cents: paid_amount_cents,
            payment_method: req.payment_method.clone().unwrap_or_else(|| "Dinheiro".into()),
            notes: Some("Pagamento integral na criação".into()),
            registered_by_user_id: Some(auth.id.clone()),
            registered_by_user_name: None,
        };
        vec![serde_json::to_value(entry).unwrap_or_default()]
    } else {
        vec![]
    };

    let insert_data = InsertTransactionDb {
        clinic_id: clinic_rec,
        appointment_id: appointment_rec,
        patient_id: patient_rec,
        user_id: user_rec,
        treatment_plan_id: treatment_plan_rec,
        direction: dir_str.into(),
        amount_cents: req.amount_cents,
        paid_amount_cents,
        description: req.description.clone(),
        category: req.category.clone(),
        status: status_str.into(),
        due_date: due_dt,
        paid_date: paid_dt,
        payment_method: req.payment_method.clone(),
        payments: initial_payments,
        installment_current: req.installment_current.unwrap_or(1),
        installment_total: req.installment_total.unwrap_or(1),
    };

    let created: Option<super::DbTransactionRow> = db
        .create("transaction")
        .content(insert_data)
        .await
        .map_err(|e| ApiError::Database(format!("Falha ao salvar transação: {}", e)))?;

    match created {
        Some(row) => Ok(HttpResponse::Created().json(map_transaction(row, req.patient_name, None))),
        None => Err(ApiError::Database("Erro ao retornar transação criada.".into())),
    }
}

/// Registra um pagamento (total ou parcial) em uma transação financeira com método de pagamento obrigatório.
#[post("/finance/{id}/pay")]
pub async fn register_transaction_payment(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<RegisterPaymentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let raw_id = path.into_inner();
    let data = body.into_inner();
    let clinic_rec_id = clinic_record_id(&data.clinic_id);

    let has_perm = check_permission(&db, &auth.id, &clinic_rec_id, "finance:write")
        .await
        .unwrap_or(false)
        || check_permission(&db, &auth.id, &clinic_rec_id, "finance:update_status")
            .await
            .unwrap_or(false);

    if !has_perm {
        return Err(ApiError::Forbidden(
            "Sem permissão para registrar pagamentos financeiros.".into(),
        ));
    }

    let method = data.payment_method.trim().to_string();
    if method.is_empty() {
        return Err(ApiError::BadRequest(
            "O método de pagamento é obrigatório.".into(),
        ));
    }

    if data.amount_cents <= 0 {
        return Err(ApiError::BadRequest(
            "O valor do pagamento deve ser maior que zero.".into(),
        ));
    }

    // Se for calculado da agenda
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

        let total_amount = app.financial_amount_cents.unwrap_or(0);
        let fin_status = if data.amount_cents >= total_amount && total_amount > 0 {
            "paid"
        } else {
            "partial"
        };

        let payment_entry = TransactionPaymentEntry {
            id: uuid::Uuid::new_v4().to_string(),
            paid_at: data.paid_date.clone().unwrap_or_else(|| Utc::now().to_rfc3339()),
            amount_cents: data.amount_cents,
            payment_method: method.clone(),
            notes: data.notes.clone(),
            registered_by_user_id: Some(auth.id.clone()),
            registered_by_user_name: None,
        };
        let p_json = serde_json::to_value(&payment_entry).unwrap_or_default();

        let insert_data = InsertTransactionDb {
            clinic_id: app.clinic_id.clone(),
            appointment_id: Some(app.id.clone()),
            patient_id: app.patient_id.clone(),
            user_id: None,
            treatment_plan_id: None,
            direction: "income".into(),
            amount_cents: total_amount,
            paid_amount_cents: data.amount_cents,
            description: format!("Consulta: {}", app.title),
            category: "Procedimento Clínico".into(),
            status: fin_status.into(),
            due_date: app.scheduled_for,
            paid_date: Some(Utc::now()),
            payment_method: Some(method),
            payments: vec![p_json],
            installment_current: 1,
            installment_total: 1,
        };

        let created: Option<super::DbTransactionRow> = db
            .create("transaction")
            .content(insert_data)
            .await
            .map_err(|e| ApiError::Database(format!("Falha ao registrar liquidação da agenda: {}", e)))?;

        return match created {
            Some(row) => Ok(HttpResponse::Ok().json(map_transaction(row, app.patient_name, None))),
            None => Err(ApiError::Database("Erro ao retornar transação liquidada.".into())),
        };
    }

    let tx_id = transaction_record_id(&raw_id);
    let mut tx_res = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", tx_id.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao carregar transação.".into()))?;

    let tx_opt: Option<super::DbTransactionRow> = tx_res.take(0).unwrap_or(None);
    let Some(tx) = tx_opt else {
        return Err(ApiError::NotFound("Transação não encontrada.".into()));
    };

    let total = tx.amount_cents;
    let prev_paid = tx.paid_amount_cents.unwrap_or(0);
    let new_paid = prev_paid + data.amount_cents;

    let fin_status = if new_paid >= total && total > 0 {
        "paid"
    } else {
        "partial"
    };

    let payment_entry = TransactionPaymentEntry {
        id: uuid::Uuid::new_v4().to_string(),
        paid_at: data.paid_date.unwrap_or_else(|| Utc::now().to_rfc3339()),
        amount_cents: data.amount_cents,
        payment_method: method.clone(),
        notes: data.notes.clone(),
        registered_by_user_id: Some(auth.id.clone()),
        registered_by_user_name: None,
    };
    let p_json = serde_json::to_value(&payment_entry).unwrap_or_default();

    db.query(
        "UPDATE type::record($id) SET
        paid_amount_cents = $paid,
        status = $status,
        payment_method = $method,
        paid_date = time::now(),
        payments = array::concat(IF payments != NONE THEN payments ELSE [] END, [$entry]),
        updated_at = time::now();",
    )
    .bind(("id", parse_record_id("transaction", &raw_id)))
    .bind(("paid", new_paid))
    .bind(("status", fin_status.to_string()))
    .bind(("method", method))
    .bind(("entry", p_json))
    .await
    .map_err(|e| ApiError::Database(format!("Erro ao atualizar pagamento: {}", e)))?;

    // Se a transação tem um plano de tratamento vinculado, sincroniza o plano
    if let Some(ref plan_rec) = tx.treatment_plan_id {
        let _ = db
            .query(
                "UPDATE type::record($pid) SET
                paid_amount_cents = $paid,
                status = IF status == 'draft' THEN 'approved' ELSE status END,
                updated_at = time::now();",
            )
            .bind(("pid", plan_rec.clone()))
            .bind(("paid", new_paid))
            .await;
    }

    let mut updated_res = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", parse_record_id("transaction", &raw_id)))
        .await
        .map_err(|_| ApiError::Database("Erro ao carregar transação atualizada.".into()))?;

    let updated: Option<super::DbTransactionRow> = updated_res.take(0).unwrap_or(None);
    match updated {
        Some(row) => Ok(HttpResponse::Ok().json(map_transaction(row, None, None))),
        None => Err(ApiError::NotFound("Transação não encontrada após pagamento.".into())),
    }
}

/// Atualiza o status de uma transação financeira (Liquidado, Pendente, Cancelado, etc).
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
        TransactionStatus::Partial => "partial",
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

        let amount = app.financial_amount_cents.unwrap_or(0);
        let insert_data = InsertTransactionDb {
            clinic_id: app.clinic_id.clone(),
            appointment_id: Some(app.id.clone()),
            patient_id: app.patient_id.clone(),
            user_id: None,
            treatment_plan_id: None,
            direction: "income".into(),
            amount_cents: amount,
            paid_amount_cents: if req.status == TransactionStatus::Paid { amount } else { 0 },
            description: format!("Consulta: {}", app.title),
            category: "Procedimento Clínico".into(),
            status: status_str.into(),
            due_date: app.scheduled_for,
            paid_date: paid_dt,
            payment_method: req.payment_method.clone(),
            payments: vec![],
            installment_current: 1,
            installment_total: 1,
        };

        let created: Option<super::DbTransactionRow> = db
            .create("transaction")
            .content(insert_data)
            .await
            .map_err(|e| ApiError::Database(format!("Falha ao registrar liquidação da agenda: {}", e)))?;

        return match created {
            Some(row) => Ok(HttpResponse::Ok().json(map_transaction(row, app.patient_name, None))),
            None => Err(ApiError::Database("Erro ao retornar transação liquidada.".into())),
        };
    }

    // Caso 2: Transação existente na tabela `transaction`
    let tx_id = parse_record_id("transaction", &raw_id);
    let mut update_query = "UPDATE type::record($id) SET status = $status".to_string();
    if paid_dt.is_some() {
        update_query.push_str(", paid_date = $paid_date");
    } else {
        update_query.push_str(", paid_date = NONE");
    }

    if req.payment_method.is_some() {
        update_query.push_str(", payment_method = $payment_method");
    }

    if req.status == TransactionStatus::Paid {
        update_query.push_str(", paid_amount_cents = amount_cents");
    }

    let mut q = db.query(&update_query)
        .bind(("id", tx_id.clone()))
        .bind(("status", status_str));

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
        Some(row) => Ok(HttpResponse::Ok().json(map_transaction(row, None, None))),
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

    let tx_id = parse_record_id("transaction", &raw_id);
    db.query("DELETE type::record($id)")
        .bind(("id", tx_id))
        .await
        .map_err(|e| ApiError::Database(format!("Falha ao excluir transação: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Lançamento financeiro removido com sucesso."
    })))
}
