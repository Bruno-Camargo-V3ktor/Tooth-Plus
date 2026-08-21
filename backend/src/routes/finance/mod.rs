//! # Módulo de Gestão Financeira e Fluxo de Caixa (Backend)
//!
//! Agrega submódulos para relatórios analíticos de fluxo de caixa, conciliação
//! com agendamentos odontológicos e lançamentos de receitas e despesas.

pub mod reports;
pub mod transactions;

pub use reports::*;
pub use transactions::*;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use shared::finance::{TransactionDirection, TransactionStatus};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Parâmetro de busca comum com identificador da clínica.
#[derive(Deserialize, Debug, Clone)]
pub struct ClinicQuery {
    pub clinic_id: String,
}

/// Auxiliar para conversão de ID do SurrealDB.
pub(crate) fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else if let Some(stripped) = raw.strip_prefix(&format!("{}s:", table)) {
        stripped
    } else if let Some(pos) = raw.find(':') {
        &raw[pos + 1..]
    } else {
        raw
    };
    let clean_key = key.trim_matches(|c| c == '⟨' || c == '⟩');
    RecordId::new(table, clean_key)
}

/// Normaliza identificador de clínica.
pub(crate) fn clinic_record_id(id: &str) -> String {
    if id.starts_with("clinic:") {
        id.to_string()
    } else {
        format!("clinic:{}", id)
    }
}

/// Normaliza identificador de transação.
pub(crate) fn transaction_record_id(id: &str) -> String {
    if id.starts_with("transaction:") {
        id.to_string()
    } else {
        format!("transaction:{}", id)
    }
}

/// Registro bruto de transação retornado pelo SurrealDB.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbTransactionRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub appointment_id: Option<RecordId>,
    pub patient_id: Option<RecordId>,
    pub user_id: Option<RecordId>,
    pub treatment_plan_id: Option<RecordId>,
    pub direction: String,
    pub amount_cents: i64,
    #[serde(default)]
    pub paid_amount_cents: Option<i64>,
    pub description: String,
    pub category: String,
    pub status: String,
    pub due_date: DateTime<Utc>,
    pub paid_date: Option<DateTime<Utc>>,
    pub payment_method: Option<String>,
    #[serde(default)]
    pub payments: Option<Vec<serde_json::Value>>,
    #[serde(default = "default_one")]
    pub installment_current: i32,
    #[serde(default = "default_one")]
    pub installment_total: i32,
}

fn default_one() -> i32 {
    1
}

/// Registro de consulta para cálculo de receitas pendentes da agenda.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbAppointmentPendingRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub patient_name: Option<String>,
    pub patient_id: Option<RecordId>,
    pub title: String,
    pub scheduled_for: DateTime<Utc>,
    pub financial_amount_cents: Option<i64>,
    pub financial_type: Option<String>,
}

/// Converte string para `TransactionDirection`.
pub(crate) fn parse_direction(d: &str) -> TransactionDirection {
    match d {
        "expense" => TransactionDirection::Expense,
        _ => TransactionDirection::Income,
    }
}

/// Converte string para `TransactionStatus`.
pub(crate) fn parse_status(s: &str) -> TransactionStatus {
    match s {
        "paid" => TransactionStatus::Paid,
        "partial" | "partially_paid" => TransactionStatus::Partial,
        "canceled" => TransactionStatus::Canceled,
        "refunded" => TransactionStatus::Refunded,
        _ => TransactionStatus::Pending,
    }
}

pub(crate) fn map_transaction(
    row: DbTransactionRow,
    patient_name: Option<String>,
    user_name: Option<String>,
) -> shared::finance::Transaction {
    let amount_cents = row.amount_cents;
    let paid_amount_cents = row.paid_amount_cents.unwrap_or_else(|| {
        if row.status == "paid" {
            amount_cents
        } else {
            0
        }
    });
    let remaining_amount_cents = (amount_cents - paid_amount_cents).max(0);

    let payments: Vec<shared::finance::TransactionPaymentEntry> = row
        .payments
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    shared::finance::Transaction {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        appointment_id: row.appointment_id.map(|id| id.to_sql()),
        patient_id: row.patient_id.map(|id| id.to_sql()),
        patient_name,
        user_id: row.user_id.map(|id| id.to_sql()),
        user_name,
        treatment_plan_id: row.treatment_plan_id.map(|id| id.to_sql()),
        direction: parse_direction(&row.direction),
        amount_cents,
        paid_amount_cents,
        remaining_amount_cents,
        description: row.description,
        category: row.category,
        status: parse_status(&row.status),
        due_date: row.due_date.to_rfc3339(),
        paid_date: row.paid_date.map(|d| d.to_rfc3339()),
        payment_method: row.payment_method,
        payments,
        installment_current: row.installment_current,
        installment_total: row.installment_total,
        is_calculated_pending: false,
    }
}
