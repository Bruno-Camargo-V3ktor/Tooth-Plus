//! # Modelos de Domínio - Ficha e Modelos de Anamnese
//!
//! Define estruturas para gestão de modelos de anamnese por clínica (Adulto e Pediátrico/Menor),
//! perguntas dinâmicas e preenchimento consentido no prontuário do paciente.

use serde::{Deserialize, Serialize};

/// Pergunta configurada no modelo de anamnese da clínica.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnamnesisQuestion {
    pub id: String,
    pub category: String,
    pub question_text: String,
    pub question_type: String, // "yes_no", "text", "multiple_choice"
    pub options: Option<Vec<String>>,
    pub required: bool,
}

/// Modelo de Ficha de Anamnese da Clínica (Adulto ou Infantil).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnamnesisTemplate {
    pub id: String,
    pub clinic_id: String,
    pub template_type: String, // "adult" ou "minor"
    pub title: String,
    pub questions: Vec<AnamnesisQuestion>,
    pub created_at: String,
    pub updated_at: String,
}

/// Resposta individual a uma pergunta de anamnese na ficha do paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AnamnesisResponseItem {
    pub question_id: String,
    pub category: String,
    pub question_text: String,
    pub question_type: String,
    pub answer_boolean: Option<bool>,
    pub answer_text: Option<String>,
    pub notes: Option<String>,
}

/// Requisição para salvar ou atualizar o modelo de anamnese padrão da clínica.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SaveAnamnesisTemplateRequest {
    pub clinic_id: String,
    pub template_type: String,
    pub title: String,
    pub questions: Vec<AnamnesisQuestion>,
}

/// Requisição para sincronizar/atualizar a anamnese do paciente com o modelo mais recente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SyncAnamnesisRequest {
    pub clinic_id: String,
    pub template_type: Option<String>,
}
