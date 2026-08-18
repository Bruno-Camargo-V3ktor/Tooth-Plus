//! # Modelos de Domínio - Agenda e Agendamentos Odontológicos
//!
//! Este módulo define os status, tipos de consulta, estruturas de agendamentos
//! e recursos do calendário clínico.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentStatus {
    Pending,
    Confirmed,
    InProgress,
    Completed,
    Canceled,
    NoShow,
}

impl AppointmentStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pendente",
            Self::Confirmed => "Confirmado",
            Self::InProgress => "Em Atendimento",
            Self::Completed => "Concluído",
            Self::Canceled => "Cancelado",
            Self::NoShow => "Não Compareceu",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Pending => "app-status-pending",
            Self::Confirmed => "app-status-confirmed",
            Self::InProgress => "app-status-in-progress",
            Self::Completed => "app-status-completed",
            Self::Canceled => "app-status-canceled",
            Self::NoShow => "app-status-no-show",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentType {
    Consultation,
    Treatment,
    Surgery,
    Return,
    Meeting,
    Other,
}

impl AppointmentType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Consultation => "Consulta",
            Self::Treatment => "Tratamento",
            Self::Surgery => "Cirurgia",
            Self::Return => "Retorno",
            Self::Meeting => "Reunião",
            Self::Other => "Outro",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Consultation => "type-badge type-consultation",
            Self::Treatment => "type-badge type-treatment",
            Self::Surgery => "type-badge type-surgery",
            Self::Return => "type-badge type-return",
            Self::Meeting => "type-badge type-meeting",
            Self::Other => "type-badge type-other",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AssignedUserDto {
    pub user_id: String,
    pub user_name: Option<String>,
    pub role_in_appointment: String,
    pub split_percentage: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConsumedItemDto {
    pub item_id: String,
    pub item_name: Option<String>,
    pub quantity_planned: i32,
    pub quantity_used: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppointmentResponse {
    pub id: String,
    pub clinic_id: String,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub title: String,
    pub scheduled_for: String,
    pub duration_minutes: i32,
    pub status: AppointmentStatus,
    pub appointment_type: AppointmentType,
    pub financial_amount_cents: Option<i64>,
    pub financial_type: Option<String>,
    pub cancellation_reason: Option<String>,
    pub assigned_users: Vec<AssignedUserDto>,
    pub consumed_items: Vec<ConsumedItemDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateAppointmentRequest {
    pub clinic_id: String,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub title: String,
    pub scheduled_for: String,
    pub duration_minutes: i32,
    pub appointment_type: AppointmentType,
    pub financial_amount_cents: Option<i64>,
    pub financial_type: Option<String>,
    pub assigned_users: Vec<AssignedUserDto>,
    pub consumed_items: Vec<ConsumedItemDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateAppointmentRequest {
    pub title: Option<String>,
    pub scheduled_for: Option<String>,
    pub duration_minutes: Option<i32>,
    pub appointment_type: Option<AppointmentType>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub financial_amount_cents: Option<i64>,
    pub financial_type: Option<String>,
    pub assigned_users: Option<Vec<AssignedUserDto>>,
    pub consumed_items: Option<Vec<ConsumedItemDto>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateAppointmentStatusRequest {
    pub status: AppointmentStatus,
    pub cancellation_reason: Option<String>,
    pub consumed_items: Option<Vec<ConsumedItemDto>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgendaResourceOption {
    pub id: String,
    pub name: String,
    pub extra_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgendaResourcesResponse {
    pub team_members: Vec<AgendaResourceOption>,
    pub patients: Vec<AgendaResourceOption>,
    pub inventory_items: Vec<AgendaResourceOption>,
}
