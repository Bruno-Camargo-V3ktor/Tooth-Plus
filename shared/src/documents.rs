//! # Modelos de Domínio - Documentos e Contratos Digitais
//!
//! Este módulo define os modelos para criação de modelos de contrato, emissão de termos
//! e contratos clínicos, portal público de assinatura digital, autenticação e validação por OTP.

use serde::{Deserialize, Serialize};

/// Campo de assinatura posicionado no PDF do contrato.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SignatureField {
    /// Identificador único do campo de assinatura.
    pub id: String,
    /// Tipo de signatário esperado ("patient" ou "doctor").
    pub signer_type: String,
    /// Número da página do PDF (base 1).
    pub page_number: u32,
    /// Posição horizontal relativa em porcentagem (0.0 a 100.0).
    pub x_pct: f32,
    /// Posição vertical relativa em porcentagem (0.0 a 100.0).
    pub y_pct: f32,
    /// Largura relativa do box de assinatura em porcentagem.
    pub width_pct: f32,
    /// Altura relativa do box de assinatura em porcentagem.
    pub height_pct: f32,
    /// Rótulo descritivo do campo (ex: "Assinatura do Paciente").
    pub label: String,
    /// Se o preenchimento da assinatura é obrigatório.
    pub is_required: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

/// Modelo/Template de contrato cadastrado na clínica.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ContractTemplate {
    pub id: String,
    pub clinic_id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Vec<SignatureField>,
    #[serde(default = "default_true")]
    pub requires_patient_signature: bool,
    #[serde(default = "default_false")]
    pub requires_doctor_signature: bool,
    #[serde(default = "default_true")]
    pub allow_any_dentist_signature: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Requisição para criação de um novo modelo de contrato.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateContractTemplateRequest {
    pub clinic_id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Vec<SignatureField>,
    pub requires_patient_signature: Option<bool>,
    pub requires_doctor_signature: Option<bool>,
    pub allow_any_dentist_signature: Option<bool>,
}

/// Requisição para atualização de modelo de contrato existente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdateContractTemplateRequest {
    pub clinic_id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Vec<SignatureField>,
    pub requires_patient_signature: Option<bool>,
    pub requires_doctor_signature: Option<bool>,
    pub allow_any_dentist_signature: Option<bool>,
}

/// Documento clínico emitido vinculado a um paciente e opcionalmente a um modelo.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientDocument {
    pub id: String,
    pub clinic_id: String,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub template_id: Option<String>,
    pub template_title: Option<String>,
    pub doctor_user_id: Option<String>,
    pub doctor_user_name: Option<String>,
    pub appointment_id: Option<String>,
    pub title: String,
    pub document_type: String,
    pub original_pdf_url: String,
    pub signed_pdf_url: Option<String>,
    pub status: String,
    pub signing_token: String,
    #[serde(default = "default_true")]
    pub requires_patient_signature: bool,
    #[serde(default = "default_false")]
    pub requires_doctor_signature: bool,
    #[serde(default = "default_true")]
    pub allow_any_dentist_signature: bool,
    pub patient_signed_at: Option<String>,
    pub patient_signature_data: Option<String>,
    pub doctor_signed_at: Option<String>,
    pub doctor_signature_data: Option<String>,
    pub patient_otp_verified: bool,
    pub checksum_sha256: Option<String>,
    #[serde(default)]
    pub audit_trail: Vec<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Requisição para emissão de um novo documento/contrato clínico.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreatePatientDocumentRequest {
    pub clinic_id: String,
    pub patient_id: String,
    pub template_id: Option<String>,
    pub doctor_user_id: Option<String>,
    pub appointment_id: Option<String>,
    pub title: String,
    pub document_type: String,
    pub pdf_url: Option<String>,
    pub signed_pdf_url: Option<String>,
    pub is_already_signed: Option<bool>,
    pub requires_patient_signature: Option<bool>,
    pub requires_doctor_signature: Option<bool>,
    pub allow_any_dentist_signature: Option<bool>,
}

use crate::patients::PatientAnamnesis;

/// Dados públicos do documento para renderização no portal de assinatura `/sign/:token`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicSigningDocumentResponse {
    pub document: PatientDocument,
    pub clinic_name: String,
    pub clinic_theme_color: String,
    pub clinic_logo_url: Option<String>,
    pub template: Option<ContractTemplate>,
    #[serde(default)]
    pub anamnesis: Option<PatientAnamnesis>,
    pub patient_phone_masked: String,
    pub patient_email_masked: Option<String>,
    pub doctor_phone_masked: Option<String>,
    pub doctor_email_masked: Option<String>,
    pub require_whatsapp_otp: bool,
    pub has_email_channel: bool,
    #[serde(default = "default_true")]
    pub requires_patient_signature: bool,
    #[serde(default = "default_false")]
    pub requires_doctor_signature: bool,
    #[serde(default = "default_true")]
    pub allow_any_dentist_signature: bool,
}

/// Autenticação de paciente no portal de assinatura (CPF + Senha).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientSignAuthRequest {
    pub cpf: String,
    pub password: String,
}

/// Autenticação de dentista no portal de assinatura (Login + Senha).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DoctorSignAuthRequest {
    pub username: String,
    pub password: String,
}

/// Resposta de sucesso na autenticação do portal de assinaturas.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SignAuthResponse {
    pub token: String,
    pub signer_type: String,
    pub signer_name: String,
}

/// Requisição para disparo de código de segurança OTP (WhatsApp ou E-mail).
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct RequestOtpRequest {
    pub channel: Option<String>,
    pub signer_type: Option<String>,
}

/// Requisição para registrar a assinatura digital e validar OTP.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitSignatureRequest {
    pub signature_base64: String,
    pub signer_type: String,
    pub otp_code: Option<String>,
    #[serde(default)]
    pub device_info: Option<String>,
}

/// Indicadores quantitativos de documentos clínicos e contratos.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct DocumentsKpis {
    pub total_documents: usize,
    pub pending_signatures: usize,
    pub completed_signed: usize,
    pub templates_count: usize,
}

/// Resposta consolidada com lista de documentos, modelos e KPIs.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DocumentsListResponse {
    pub documents: Vec<PatientDocument>,
    pub templates: Vec<ContractTemplate>,
    pub kpis: DocumentsKpis,
}

/// Verificação de cadastro prévio de paciente por CPF.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientCheckRequest {
    pub cpf: String,
}

/// Resposta da checagem de cadastro e existência de senha de assinatura.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientCheckResponse {
    pub patient_name: String,
    pub has_password: bool,
    pub phone_masked: String,
}

/// Cadastro inicial de senha de assinatura digital pelo próprio paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientRegisterPasswordRequest {
    pub cpf: String,
    pub password: String,
}
