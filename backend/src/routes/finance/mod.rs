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
use surrealdb::types::{RecordId, SurrealValue};

/// Parâmetro de busca comum com identificador da clínica.
#[derive(Deserialize, Debug, Clone)]
pub struct ClinicQuery {
    pub clinic_id: String,
}

/// Auxiliar para conversão de ID do SurrealDB.
pub(crate) fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
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
    pub direction: String,
    pub amount_cents: i64,
    pub description: String,
    pub category: String,
    pub status: String,
    pub due_date: DateTime<Utc>,
    pub paid_date: Option<DateTime<Utc>>,
    pub payment_method: Option<String>,
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
        "canceled" => TransactionStatus::Canceled,
        "refunded" => TransactionStatus::Refunded,
        _ => TransactionStatus::Pending,
    }
}
