use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureField {
    pub id: String,
    pub signer_type: String,
    pub page_number: u32,
    pub x_pct: f32,
    pub y_pct: f32,
    pub width_pct: f32,
    pub height_pct: f32,
    pub label: String,
    pub is_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractTemplate {
    pub id: String,
    pub clinic_id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Vec<SignatureField>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateContractTemplateRequest {
    pub clinic_id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Vec<SignatureField>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateContractTemplateRequest {
    pub clinic_id: String,
    pub title: String,
    pub category: String,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Vec<SignatureField>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePatientDocumentRequest {
    pub clinic_id: String,
    pub patient_id: String,
    pub template_id: Option<String>,
    pub doctor_user_id: Option<String>,
    pub appointment_id: Option<String>,
    pub title: String,
    pub document_type: String,
    pub pdf_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicSigningDocumentResponse {
    pub document: PatientDocument,
    pub clinic_name: String,
    pub clinic_theme_color: String,
    pub clinic_logo_url: Option<String>,
    pub template: Option<ContractTemplate>,
    pub patient_phone_masked: String,
    pub patient_email_masked: Option<String>,
    pub require_whatsapp_otp: bool,
    pub has_email_channel: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientSignAuthRequest {
    pub cpf: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoctorSignAuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignAuthResponse {
    pub token: String,
    pub signer_type: String,
    pub signer_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RequestOtpRequest {
    pub channel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitSignatureRequest {
    pub signature_base64: String,
    pub signer_type: String,
    pub otp_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DocumentsKpis {
    pub total_documents: usize,
    pub pending_signatures: usize,
    pub completed_signed: usize,
    pub templates_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentsListResponse {
    pub documents: Vec<PatientDocument>,
    pub templates: Vec<ContractTemplate>,
    pub kpis: DocumentsKpis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientCheckRequest {
    pub cpf: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientCheckResponse {
    pub patient_name: String,
    pub has_password: bool,
    pub phone_masked: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientRegisterPasswordRequest {
    pub cpf: String,
    pub password: String,
}
