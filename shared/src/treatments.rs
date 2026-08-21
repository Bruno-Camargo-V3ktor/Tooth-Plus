//! # Modelos de Domínio – Tratamentos Padrão e Orçamentos Clínicos
//!
//! Define os tipos compartilhados entre frontend e backend para:
//! - `TreatmentTemplate` — catálogo de procedimentos padrão da clínica.
//! - `PatientTreatmentPlan` — orçamento/plano de tratamento por paciente.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────
// Status enums
// ─────────────────────────────────────────────────────────────

/// Status global de um plano de tratamento (orçamento).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TreatmentPlanStatus {
    #[default]
    Draft,
    Approved,
    InProgress,
    Completed,
    Canceled,
}

impl TreatmentPlanStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "Rascunho",
            Self::Approved => "Aprovado",
            Self::InProgress => "Em Andamento",
            Self::Completed => "Concluído",
            Self::Canceled => "Cancelado",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Draft => "plan-status-draft",
            Self::Approved => "plan-status-approved",
            Self::InProgress => "plan-status-inprogress",
            Self::Completed => "plan-status-completed",
            Self::Canceled => "plan-status-canceled",
        }
    }
}

/// Status individual de um item dentro do plano (por dente/procedimento).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TreatmentItemStatus {
    #[default]
    Pending,
    InProgress,
    Done,
    Canceled,
}

impl TreatmentItemStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pendente",
            Self::InProgress => "Em Andamento",
            Self::Done => "Concluído",
            Self::Canceled => "Cancelado",
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Treatment Template (catálogo da clínica)
// ─────────────────────────────────────────────────────────────

/// Procedimento padrão reutilizável cadastrado pela clínica.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreatmentTemplate {
    pub id: String,
    pub clinic_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub default_price_cents: i64,
    pub estimated_duration_minutes: Option<i32>,
    pub dental_regions: Vec<String>,
    pub target_teeth: Vec<String>,
    pub required_materials: Vec<String>,
    pub required_equipment: Vec<String>,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateTreatmentTemplateRequest {
    pub clinic_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub default_price_cents: i64,
    pub estimated_duration_minutes: Option<i32>,
    #[serde(default)]
    pub dental_regions: Vec<String>,
    #[serde(default)]
    pub target_teeth: Vec<String>,
    #[serde(default)]
    pub required_materials: Vec<String>,
    #[serde(default)]
    pub required_equipment: Vec<String>,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateTreatmentTemplateRequest {
    pub clinic_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub default_price_cents: i64,
    pub estimated_duration_minutes: Option<i32>,
    #[serde(default)]
    pub dental_regions: Vec<String>,
    #[serde(default)]
    pub target_teeth: Vec<String>,
    #[serde(default)]
    pub required_materials: Vec<String>,
    #[serde(default)]
    pub required_equipment: Vec<String>,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
    pub is_active: bool,
}

// ─────────────────────────────────────────────────────────────
// Patient Treatment Plan (orçamento)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TreatmentPlanItem {
    pub id: String,
    pub template_id: Option<String>,
    pub procedure_name: String,
    pub category: Option<String>,
    pub tooth_number: Option<String>,
    pub dental_region: Option<String>,
    #[serde(default)]
    pub surfaces: Vec<String>,
    pub price_cents: i64,
    pub status: TreatmentItemStatus,
    pub appointment_id: Option<String>,
    pub clinical_notes: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatientTreatmentPlan {
    pub id: String,
    pub patient_id: String,
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub dentist_user_name: Option<String>,
    pub transaction_id: Option<String>,
    pub title: String,
    pub status: TreatmentPlanStatus,
    pub items: Vec<TreatmentPlanItem>,
    pub total_price_cents: i64,
    pub notes: Option<String>,
    pub planned_start_date: Option<String>,
    pub planned_end_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CreateTreatmentPlanItemRequest {
    pub template_id: Option<String>,
    pub procedure_name: String,
    pub category: Option<String>,
    pub tooth_number: Option<String>,
    pub dental_region: Option<String>,
    #[serde(default)]
    pub surfaces: Vec<String>,
    pub price_cents: i64,
    pub clinical_notes: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateTreatmentPlanRequest {
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub items: Vec<CreateTreatmentPlanItemRequest>,
    pub notes: Option<String>,
    pub planned_start_date: Option<String>,
    pub planned_end_date: Option<String>,
    pub approve_immediately: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateTreatmentPlanRequest {
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub title: String,
    pub items: Vec<CreateTreatmentPlanItemRequest>,
    pub notes: Option<String>,
    pub planned_start_date: Option<String>,
    pub planned_end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateTreatmentPlanStatusRequest {
    pub clinic_id: String,
    pub status: TreatmentPlanStatus,
}
