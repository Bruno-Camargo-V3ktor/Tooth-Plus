//! # Relatórios Financeiros e Demonstrativo de Fluxo de Caixa (Backend)
//!
//! Consolida as receitas realizadas e a receber, despesas operacionais,
//! repasses de profissionais e pendências financeiras de atendimentos clínicos.

use super::{
    DbAppointmentPendingRow, DbTransactionRow, clinic_record_id, parse_direction, parse_record_id,
    parse_status,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use actix_web::{HttpResponse, get, web};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use shared::finance::{
    FinanceQuery, FinanceResponse, FinanceSummary, Transaction, TransactionDirection,
    TransactionStatus,
};
use surrealdb::types::ToSql;

/// Consulta os lançamentos financeiros da clínica no período com métricas agregadas.
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
        .filter_map(|r| r.appointment_id.as_ref().map(|id| id.to_sql()))
        .collect();

    let mut transactions = Vec::new();
    let mut total_income = 0i64;
    let mut total_expense = 0i64;
    let mut pending_income = 0i64;
    let mut pending_expense = 0i64;

    for row in db_rows {
        let dir = parse_direction(&row.direction);
        let status = parse_status(&row.status);

        if !has_read_all {
            if dir == TransactionDirection::Income && !has_read_income {
                continue;
            }
            if dir == TransactionDirection::Expense && !has_read_expense {
                continue;
            }
            if status == TransactionStatus::Pending && !has_read_pending {
                continue;
            }
        }

        match status {
            TransactionStatus::Paid => match dir {
                TransactionDirection::Income => total_income += row.amount_cents,
                TransactionDirection::Expense => total_expense += row.amount_cents,
            },
            TransactionStatus::Pending => match dir {
                TransactionDirection::Income => pending_income += row.amount_cents,
                TransactionDirection::Expense => pending_expense += row.amount_cents,
            },
            _ => {}
        }

        transactions.push(Transaction {
            id: row.id.to_sql(),
            clinic_id: row.clinic_id.to_sql(),
            appointment_id: row.appointment_id.map(|id| id.to_sql()),
            patient_id: row.patient_id.map(|id| id.to_sql()),
            patient_name: None,
            user_id: row.user_id.map(|id| id.to_sql()),
            user_name: None,
            direction: dir,
            amount_cents: row.amount_cents,
            description: row.description,
            category: row.category,
            status,
            due_date: row.due_date.to_rfc3339(),
            paid_date: row.paid_date.map(|d| d.to_rfc3339()),
            payment_method: row.payment_method,
            installment_current: row.installment_current,
            installment_total: row.installment_total,
            is_calculated_pending: false,
        });
    }

    if has_read_all || has_read_pending || has_read_income {
        for app in app_rows {
            let app_id_str = app.id.to_sql();
            if existing_app_ids.contains(&app_id_str) {
                continue;
            }

            let cents = app.financial_amount_cents.unwrap_or(0);
            if cents <= 0 {
                continue;
            }

            pending_income += cents;

            transactions.push(Transaction {
                id: format!("calculated:{}", app_id_str),
                clinic_id: app.clinic_id.to_sql(),
                appointment_id: Some(app_id_str),
                patient_id: app.patient_id.map(|id| id.to_sql()),
                patient_name: app.patient_name,
                user_id: None,
                user_name: None,
                direction: TransactionDirection::Income,
                amount_cents: cents,
                description: format!("Consulta: {}", app.title),
                category: "consultation".into(),
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

    let summary = FinanceSummary {
        total_income_cents: total_income,
        total_expense_cents: total_expense,
        net_balance_cents: total_income - total_expense,
        pending_income_cents: pending_income,
        pending_expense_cents: pending_expense,
        total_transactions_count: transactions.len(),
    };

    Ok(HttpResponse::Ok().json(FinanceResponse {
        summary,
        transactions,
    }))
}
