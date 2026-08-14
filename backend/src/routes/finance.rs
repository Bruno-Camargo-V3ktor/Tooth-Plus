use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use actix_web::{HttpResponse, delete, get, patch, post, web};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Deserialize;
use shared::finance::{
    CreateTransactionRequest, FinanceQuery, FinanceResponse, FinanceSummary, Transaction,
    TransactionDirection, TransactionStatus, UpdateTransactionStatusRequest,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

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

fn transaction_record_id(id: &str) -> String {
    if id.starts_with("transaction:") {
        id.to_string()
    } else {
        format!("transaction:{}", id)
    }
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbTransactionRow {
    id: RecordId,
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
    #[serde(default = "default_one")]
    installment_current: i32,
    #[serde(default = "default_one")]
    installment_total: i32,
}

fn default_one() -> i32 {
    1
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbAppointmentPendingRow {
    id: RecordId,
    clinic_id: RecordId,
    patient_name: Option<String>,
    patient_id: Option<RecordId>,
    title: String,
    scheduled_for: DateTime<Utc>,
    financial_amount_cents: Option<i64>,
    financial_type: Option<String>,
}

fn parse_direction(d: &str) -> TransactionDirection {
    match d {
        "expense" => TransactionDirection::Expense,
        _ => TransactionDirection::Income,
    }
}

fn parse_status(s: &str) -> TransactionStatus {
    match s {
        "paid" => TransactionStatus::Paid,
        "canceled" => TransactionStatus::Canceled,
        "refunded" => TransactionStatus::Refunded,
        _ => TransactionStatus::Pending,
    }
}

#[get("/finance")]
pub async fn get_finance_data(
    auth: AuthenticatedUser,
    query: web::Query<FinanceQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_rec = clinic_record_id(&query.clinic_id);

    let has_read_all = check_permission(&db, &auth.id, &clinic_rec, "finance:read_all")
        .await
        .unwrap_or(false);
    let has_read_income = check_permission(&db, &auth.id, &clinic_rec, "finance:read_income")
        .await
        .unwrap_or(false);
    let has_read_expense = check_permission(&db, &auth.id, &clinic_rec, "finance:read_expense")
        .await
        .unwrap_or(false);
    let has_read_pending = check_permission(&db, &auth.id, &clinic_rec, "finance:read_pending")
        .await
        .unwrap_or(false);
    let has_general_read = check_permission(&db, &auth.id, &clinic_rec, "finance:read")
        .await
        .unwrap_or(false);

    if !has_read_all
        && !has_read_income
        && !has_read_expense
        && !has_read_pending
        && !has_general_read
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para visualizar dados financeiros desta unidade.".into(),
        ));
    }

    let (start_dt, end_dt) = if let (Some(s), Some(e)) = (&query.start_date, &query.end_date) {
        let s_dt = DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .unwrap_or_else(|_| Utc::now());
        let e_dt = DateTime::parse_from_rfc3339(e)
            .map(|d| d.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(e, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(23, 59, 59).unwrap().and_utc())
            })
            .unwrap_or_else(|_| Utc::now());
        (s_dt, e_dt)
    } else {
        let year = query.year.unwrap_or_else(|| Utc::now().year());
        let month = query.month.unwrap_or_else(|| Utc::now().month());
        let start_dt = Utc
            .with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| Utc::now());

        let (next_year, next_month) = if month >= 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let end_dt = Utc
            .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| Utc::now());
        (start_dt, end_dt)
    };

    let clinic_rec_id = parse_record_id("clinic", &query.clinic_id);

    let mut tx_response = db
        .query(
            "SELECT * FROM transaction
            WHERE clinic_id = $clinic
            AND due_date >= $start_date
            AND due_date <= $end_date
            ORDER BY due_date DESC",
        )
        .bind(("clinic", clinic_rec_id.clone()))
        .bind(("start_date", start_dt))
        .bind(("end_date", end_dt))
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar transações financeiras.".into()))?;

    let db_rows: Vec<DbTransactionRow> = tx_response.take(0).unwrap_or_default();

    let mut app_response = db
        .query(
            "SELECT id, clinic_id, patient_name, patient_id, title, scheduled_for, financial_amount_cents, financial_type
            FROM appointment
            WHERE clinic_id = $clinic
            AND scheduled_for >= $start_date
            AND scheduled_for <= $end_date
            AND financial_amount_cents != NONE
            AND financial_amount_cents > 0
            ORDER BY scheduled_for DESC",
        )
        .bind(("clinic", clinic_rec_id))
        .bind(("start_date", start_dt))
        .bind(("end_date", end_dt))
        .await
        .map_err(|_| ApiError::Database("Falha ao calcular pendências da agenda.".into()))?;

    let app_rows: Vec<DbAppointmentPendingRow> = app_response.take(0).unwrap_or_default();

    let existing_app_ids: Vec<String> = db_rows
        .iter()
        .filter_map(|r| r.appointment_id.as_ref().map(|a| a.to_sql()))
        .collect();

    let mut transactions = Vec::new();
    let mut total_income_cents: i64 = 0;
    let mut total_expense_cents: i64 = 0;
    let mut pending_income_cents: i64 = 0;
    let mut pending_expense_cents: i64 = 0;

    for r in db_rows {
        let dir = parse_direction(&r.direction);
        let st = parse_status(&r.status);

        match (dir, st) {
            (TransactionDirection::Income, TransactionStatus::Paid) => {
                if has_read_income || has_read_all {
                    total_income_cents += r.amount_cents;
                }
            }
            (TransactionDirection::Expense, TransactionStatus::Paid) => {
                if has_read_expense || has_read_all {
                    total_expense_cents += r.amount_cents;
                }
            }
            (TransactionDirection::Income, TransactionStatus::Pending) => {
                if has_read_pending || has_read_all {
                    pending_income_cents += r.amount_cents;
                }
            }
            (TransactionDirection::Expense, TransactionStatus::Pending) => {
                if has_read_pending || has_read_all {
                    pending_expense_cents += r.amount_cents;
                }
            }
            _ => {}
        }

        let can_view_item = match dir {
            TransactionDirection::Income => has_read_income || has_read_all,
            TransactionDirection::Expense => has_read_expense || has_read_all,
        };

        if can_view_item {
            transactions.push(Transaction {
                id: r.id.key.to_sql(),
                clinic_id: r.clinic_id.key.to_sql(),
                appointment_id: r.appointment_id.map(|a| a.key.to_sql()),
                patient_id: r.patient_id.map(|p| p.key.to_sql()),
                patient_name: None,
                user_id: r.user_id.map(|u| u.key.to_sql()),
                user_name: None,
                direction: dir,
                amount_cents: r.amount_cents,
                description: r.description,
                category: r.category,
                status: st,
                due_date: r.due_date.to_rfc3339(),
                paid_date: r.paid_date.map(|p| p.to_rfc3339()),
                payment_method: r.payment_method,
                installment_current: r.installment_current,
                installment_total: r.installment_total,
                is_calculated_pending: false,
            });
        }
    }

    if has_read_pending || has_read_all {
        for app in app_rows {
            let full_app_id = app.id.to_sql();
            if existing_app_ids.contains(&full_app_id) {
                continue;
            }

            let amount = app.financial_amount_cents.unwrap_or(0);
            if amount <= 0 {
                continue;
            }

            let dir = match app.financial_type.as_deref() {
                Some("expense") => TransactionDirection::Expense,
                _ => TransactionDirection::Income,
            };

            if dir == TransactionDirection::Income {
                pending_income_cents += amount;
            } else {
                pending_expense_cents += amount;
            }

            let can_view_item = match dir {
                TransactionDirection::Income => has_read_income || has_read_all,
                TransactionDirection::Expense => has_read_expense || has_read_all,
            };

            if can_view_item {
                let desc = format!("Faturamento Agenda: {}", app.title);
                transactions.push(Transaction {
                    id: format!("sim_{}", app.id.key.to_sql()),
                    clinic_id: app.clinic_id.key.to_sql(),
                    appointment_id: Some(app.id.key.to_sql()),
                    patient_id: app.patient_id.map(|p| p.key.to_sql()),
                    patient_name: app.patient_name,
                    user_id: None,
                    user_name: None,
                    direction: dir,
                    amount_cents: amount,
                    description: desc,
                    category: "Procedimento Clínico".to_string(),
                    status: TransactionStatus::Pending,
                    due_date: app.scheduled_for.to_rfc3339(),
                    paid_date: None,
                    payment_method: None,
                    installment_current: 1,
                    installment_total: 1,
                    is_calculated_pending: true,
                });
            }
        }
    }

    transactions.sort_by(|a, b| b.due_date.cmp(&a.due_date));

    let net_balance_cents =
        if (has_read_income || has_read_all) && (has_read_expense || has_read_all) {
            total_income_cents - total_expense_cents
        } else if has_read_income || has_read_all {
            total_income_cents
        } else if has_read_expense || has_read_all {
            -total_expense_cents
        } else {
            0
        };
    let total_transactions_count = transactions.len();

    let summary = FinanceSummary {
        total_income_cents,
        total_expense_cents,
        net_balance_cents,
        pending_income_cents,
        pending_expense_cents,
        total_transactions_count,
    };

    Ok(HttpResponse::Ok().json(FinanceResponse {
        summary,
        transactions,
    }))
}

#[post("/finance")]
pub async fn create_transaction(
    auth: AuthenticatedUser,
    req: web::Json<CreateTransactionRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let clinic_rec = clinic_record_id(&data.clinic_id);

    let required_perm = match data.direction {
        TransactionDirection::Income => "finance:write_income",
        TransactionDirection::Expense => "finance:write_expense",
    };

    let has_specific = check_permission(&db, &auth.id, &clinic_rec, required_perm)
        .await
        .unwrap_or(false);
    let has_general = check_permission(&db, &auth.id, &clinic_rec, "finance:write")
        .await
        .unwrap_or(false);

    if !has_specific && !has_general {
        return Err(ApiError::Forbidden(
            "Sem privilégios para lançar movimentações financeiras.".into(),
        ));
    }

    let clinic_rec_id = parse_record_id("clinic", &data.clinic_id);
    let app_rec_id = data
        .appointment_id
        .as_ref()
        .map(|id| parse_record_id("appointment", id));
    let patient_rec_id = data
        .patient_id
        .as_ref()
        .map(|id| parse_record_id("patient", id));
    let user_rec_id = data.user_id.as_ref().map(|id| parse_record_id("user", id));

    let dir_str = match data.direction {
        TransactionDirection::Income => "income",
        TransactionDirection::Expense => "expense",
    };

    let status_str = match data.status {
        TransactionStatus::Paid => "paid",
        TransactionStatus::Canceled => "canceled",
        TransactionStatus::Refunded => "refunded",
        TransactionStatus::Pending => "pending",
    };

    let due_date_dt = DateTime::parse_from_rfc3339(&data.due_date)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let paid_date_dt = data.paid_date.as_ref().and_then(|p| {
        DateTime::parse_from_rfc3339(p)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    let mut response = db
        .query(
            "CREATE transaction SET
                clinic_id           = $clinic,
                appointment_id      = $appointment,
                patient_id          = $patient,
                user_id             = $user,
                direction           = $direction,
                amount_cents        = $amount,
                description         = $description,
                category            = $category,
                status              = $status,
                due_date            = $due_date,
                paid_date           = $paid_date,
                payment_method      = $payment_method,
                installment_current = $inst_cur,
                installment_total   = $inst_tot
            RETURN id",
        )
        .bind(("clinic", clinic_rec_id))
        .bind(("appointment", app_rec_id))
        .bind(("patient", patient_rec_id))
        .bind(("user", user_rec_id))
        .bind(("direction", dir_str))
        .bind(("amount", data.amount_cents))
        .bind(("description", data.description))
        .bind(("category", data.category))
        .bind(("status", status_str))
        .bind(("due_date", due_date_dt))
        .bind(("paid_date", paid_date_dt))
        .bind(("payment_method", data.payment_method))
        .bind(("inst_cur", data.installment_current.unwrap_or(1)))
        .bind(("inst_tot", data.installment_total.unwrap_or(1)))
        .await
        .map_err(|_| ApiError::Database("Falha ao criar transação financeira.".into()))?;

    #[derive(Deserialize, SurrealValue)]
    struct CreatedId {
        id: RecordId,
    }

    let created: Option<CreatedId> = response.take(0).unwrap_or(None);
    let new_id = created
        .ok_or_else(|| ApiError::Database("Transação não retornou ID.".into()))?
        .id;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": new_id.key.to_sql(),
        "message": "Lançamento financeiro registrado com sucesso."
    })))
}

#[patch("/finance/{id}/status")]
pub async fn update_transaction_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<FinanceQuery>,
    req: web::Json<UpdateTransactionStatusRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let tx_id = path.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);

    let has_status_perm = check_permission(&db, &auth.id, &clinic_rec, "finance:update_status")
        .await
        .unwrap_or(false);
    let has_write_perm = check_permission(&db, &auth.id, &clinic_rec, "finance:write")
        .await
        .unwrap_or(false);

    if !has_status_perm && !has_write_perm {
        return Err(ApiError::Forbidden(
            "Sem privilégios para atualizar status financeiro.".into(),
        ));
    }

    let data = req.into_inner();
    let status_str = match data.status {
        TransactionStatus::Paid => "paid",
        TransactionStatus::Canceled => "canceled",
        TransactionStatus::Refunded => "refunded",
        TransactionStatus::Pending => "pending",
    };

    let paid_date_dt = if data.status == TransactionStatus::Paid {
        data.paid_date
            .as_ref()
            .and_then(|p| {
                DateTime::parse_from_rfc3339(p)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .or_else(|| Some(Utc::now()))
    } else {
        None
    };

    let tx_rec = transaction_record_id(&tx_id);
    db.query(
        "UPDATE type::record($tx_id) SET
            status         = $status,
            paid_date      = $paid_date,
            payment_method = IF $payment_method != NONE THEN $payment_method ELSE payment_method END",
    )
    .bind(("tx_id", tx_rec))
    .bind(("status", status_str))
    .bind(("paid_date", paid_date_dt))
    .bind(("payment_method", data.payment_method))
    .await
    .map_err(|_| ApiError::Database("Falha ao atualizar status da transação.".into()))?;

    Ok(HttpResponse::Ok().json("Status atualizado com sucesso."))
}

#[delete("/finance/{id}")]
pub async fn delete_transaction(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<FinanceQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let tx_id = path.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "finance:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para remover transações financeiras.".into(),
        ));
    }

    let tx_rec = transaction_record_id(&tx_id);
    db.query("DELETE type::record($tx_id)")
        .bind(("tx_id", tx_rec))
        .await
        .map_err(|_| ApiError::Database("Falha ao excluir transação financeira.".into()))?;

    Ok(HttpResponse::Ok().json("Transação removida com sucesso."))
}
