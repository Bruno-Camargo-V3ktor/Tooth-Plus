//! # Módulo de Agendamentos e Agenda Odontológica (Backend)
//!
//! Agrega sub-módulos para criação, listagem, atualização, recursos de calendário e
//! transição de status de consultas e procedimentos odontológicos.

pub mod resources;
pub mod scheduling;
pub mod status;

pub use resources::*;
pub use scheduling::*;
pub use status::*;

use serde::Deserialize;
use shared::appointments::{AppointmentStatus, AppointmentType};
use surrealdb::types::{RecordId, SurrealValue};

/// Query string padrão com `clinic_id`.
#[derive(Deserialize)]
pub struct ClinicQuery {
    pub clinic_id: String,
}

/// Registro do agendamento vindo do banco de dados SurrealDB.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbAppointmentRecord {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub patient_id: Option<RecordId>,
    pub patient_name: Option<String>,
    pub title: String,
    pub scheduled_for: chrono::DateTime<chrono::Utc>,
    pub duration_minutes: i32,
    pub status: String,
    pub appointment_type: String,
    pub financial_amount_cents: Option<i64>,
    pub financial_type: Option<String>,
    pub cancellation_reason: Option<String>,
}

/// Profissional vinculado ao agendamento via relação de grafo `assigned_to`.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbAssignedRecord {
    pub user_id: RecordId,
    pub user_name: Option<String>,
    pub role_in_appointment: String,
    pub split_percentage: i32,
}

/// Item de estoque consumido no atendimento via relação `consumes`.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbConsumedRecord {
    pub item_id: RecordId,
    pub item_name: Option<String>,
    pub quantity_planned: i32,
    pub quantity_used: Option<i32>,
}

/// Recurso para preenchimento de selects (membros, pacientes, itens).
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbResourceRecord {
    pub id: RecordId,
    pub name: String,
    pub extra_info: Option<String>,
}

/// Converte string em `RecordId`.
pub(crate) fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
}

/// Normaliza ID de clínica para `clinic:UUID`.
pub(crate) fn clinic_record_id(id: &str) -> String {
    if id.starts_with("clinic:") {
        id.to_string()
    } else {
        format!("clinic:{}", id)
    }
}

/// Normaliza ID de agendamento para `appointment:UUID`.
pub(crate) fn appointment_record_id(id: &str) -> String {
    if id.starts_with("appointment:") {
        id.to_string()
    } else {
        format!("appointment:{}", id)
    }
}

/// Normaliza ID de paciente para `patient:UUID`.
pub(crate) fn patient_record_id(id: &str) -> String {
    if id.starts_with("patient:") {
        id.to_string()
    } else {
        format!("patient:{}", id)
    }
}

/// Normaliza ID de item de inventário para `inventory_item:UUID`.
pub(crate) fn inventory_record_id(id: &str) -> String {
    if id.starts_with("inventory_item:") {
        id.to_string()
    } else {
        format!("inventory_item:{}", id)
    }
}

/// Converte string em enum `AppointmentStatus`.
pub(crate) fn parse_status(s: &str) -> AppointmentStatus {
    match s {
        "confirmed" => AppointmentStatus::Confirmed,
        "in_progress" => AppointmentStatus::InProgress,
        "completed" => AppointmentStatus::Completed,
        "canceled" => AppointmentStatus::Canceled,
        "no_show" => AppointmentStatus::NoShow,
        _ => AppointmentStatus::Pending,
    }
}

/// Converte string em enum `AppointmentType`.
pub(crate) fn parse_type(s: &str) -> AppointmentType {
    match s {
        "treatment" => AppointmentType::Treatment,
        "surgery" => AppointmentType::Surgery,
        "return" => AppointmentType::Return,
        "meeting" => AppointmentType::Meeting,
        "other" => AppointmentType::Other,
        _ => AppointmentType::Consultation,
    }
}

/// Converte enum `AppointmentStatus` para string do banco.
pub(crate) fn status_to_str(s: &AppointmentStatus) -> &'static str {
    match s {
        AppointmentStatus::Pending => "pending",
        AppointmentStatus::Confirmed => "confirmed",
        AppointmentStatus::InProgress => "in_progress",
        AppointmentStatus::Completed => "completed",
        AppointmentStatus::Canceled => "canceled",
        AppointmentStatus::NoShow => "no_show",
    }
}

/// Converte enum `AppointmentType` para string do banco.
pub(crate) fn type_to_str(t: &AppointmentType) -> &'static str {
    match t {
        AppointmentType::Consultation => "consultation",
        AppointmentType::Treatment => "treatment",
        AppointmentType::Surgery => "surgery",
        AppointmentType::Return => "return",
        AppointmentType::Meeting => "meeting",
        AppointmentType::Other => "other",
    }
}
