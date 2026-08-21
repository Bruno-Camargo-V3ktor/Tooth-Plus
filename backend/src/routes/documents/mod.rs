//! # Módulo de Documentos e Contratos Clínicos (Backend)
//!
//! Agrega sub-módulos para gerenciamento de modelos de contratos (templates),
//! emissão de documentos clínicos para pacientes e portal público de assinatura digital.

pub mod issued_docs;
pub mod public_signing;
pub mod templates;

pub use issued_docs::*;
pub use public_signing::*;
pub use templates::*;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use shared::documents::{ContractTemplate, PatientDocument, SignatureField};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Converte uma string bruta de ID no tipo `RecordId` do SurrealDB.
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

/// Normaliza o ID da clínica para o formato prefixado `clinic:UUID`.
pub(crate) fn clinic_record_id(clinic_id: &str) -> String {
    if clinic_id.starts_with("clinic:") {
        clinic_id.to_string()
    } else {
        format!("clinic:{}", clinic_id)
    }
}

/// Linha da tabela `contract_template` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbContractTemplateRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub title: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub pdf_url: String,
    pub signature_fields: Option<serde_json::Value>,
    pub requires_patient_signature: Option<bool>,
    pub requires_doctor_signature: Option<bool>,
    pub allow_any_dentist_signature: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Linha da tabela `patient_document` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbPatientDocumentRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub patient_id: RecordId,
    pub template_id: Option<RecordId>,
    pub doctor_user_id: Option<RecordId>,
    pub appointment_id: Option<RecordId>,
    pub title: String,
    pub document_type: Option<String>,
    pub original_pdf_url: String,
    pub signed_pdf_url: Option<String>,
    pub status: Option<String>,
    pub signing_token: String,
    pub requires_patient_signature: Option<bool>,
    pub requires_doctor_signature: Option<bool>,
    pub allow_any_dentist_signature: Option<bool>,
    pub patient_signed_at: Option<DateTime<Utc>>,
    pub patient_signature_data: Option<String>,
    pub doctor_signed_at: Option<DateTime<Utc>>,
    pub doctor_signature_data: Option<String>,
    pub patient_otp_verified: Option<bool>,
    pub otp_code_hash: Option<String>,
    pub otp_expires_at: Option<DateTime<Utc>>,
    pub final_checksum_sha256: Option<String>,
    pub audit_trail: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dados da clínica necessários para cabeçalhos e configurações de e-mail/OTP.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbClinicInfo {
    pub id: RecordId,
    pub trading_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub theme_color: Option<String>,
    pub logo_url: Option<String>,
    pub whatsapp_instance: Option<String>,
    pub require_esign: Option<bool>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub smtp_from: Option<String>,
    pub smtp_tls: Option<bool>,
}

/// Dados de autenticação do paciente para validação no portal.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbPatientAuthRow {
    pub id: RecordId,
    pub full_name: String,
    pub document_cpf: Option<String>,
    pub document_cpf_encrypted: Option<String>,
    pub document_cpf_hash: Option<String>,
    pub document_rg: Option<String>,
    pub birth_date: Option<String>,
    pub legal_guardians: Option<serde_json::Value>,
    pub legal_guardian_name: Option<String>,
    pub legal_guardian_cpf: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

/// Recupera o CPF em texto claro do paciente para carimbo no PDF.
pub(crate) fn get_patient_decrypted_cpf(pat: &DbPatientAuthRow) -> String {
    if let Some(ref enc) = pat.document_cpf_encrypted {
        if let Ok(dec) = crate::security::crypto::decrypt_deterministic(enc) {
            return dec;
        }
    }
    if let Some(ref plain) = pat.document_cpf {
        if !plain.is_empty() {
            return plain.clone();
        }
    }
    "123.456.789-00".to_string()
}

/// Dados de autenticação do dentista para validação no portal.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbUserAuthRow {
    pub id: RecordId,
    pub username: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: String,
}

/// Converte a linha de banco de dados `DbContractTemplateRow` no modelo compartilhado `ContractTemplate`.
pub(crate) fn map_template(row: DbContractTemplateRow) -> ContractTemplate {
    let fields: Vec<SignatureField> = row
        .signature_fields
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    ContractTemplate {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        title: row.title,
        category: row.category.unwrap_or_else(|| "contract".into()),
        description: row.description,
        pdf_url: row.pdf_url,
        signature_fields: fields,
        requires_patient_signature: row.requires_patient_signature.unwrap_or(true),
        requires_doctor_signature: row.requires_doctor_signature.unwrap_or(false),
        allow_any_dentist_signature: row.allow_any_dentist_signature.unwrap_or(true),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

/// Converte a linha de banco de dados `DbPatientDocumentRow` no modelo compartilhado `PatientDocument`.
pub(crate) fn map_patient_document(row: DbPatientDocumentRow) -> PatientDocument {
    let is_anamnesis = row.document_type.as_deref() == Some("anamnesis");
    let req_pat = if is_anamnesis { true } else { row.requires_patient_signature.unwrap_or(true) };
    let req_doc = if is_anamnesis { false } else { row.requires_doctor_signature.unwrap_or(false) };
    let allow_any_doc = if is_anamnesis { false } else { row.allow_any_dentist_signature.unwrap_or(true) };

    let pat_ok = !req_pat || row.patient_signed_at.is_some();
    let doc_ok = !req_doc || row.doctor_signed_at.is_some();
    let has_sign = row.patient_signed_at.is_some() || row.doctor_signed_at.is_some();
    let is_done = pat_ok && doc_ok && (req_pat || req_doc) && has_sign;

    let final_status = if is_done {
        "signed".to_string()
    } else {
        row.status.unwrap_or_else(|| "pending_signatures".into())
    };

    PatientDocument {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        patient_id: row.patient_id.to_sql(),
        patient_name: None,
        template_id: row.template_id.map(|t| t.to_sql()),
        template_title: None,
        doctor_user_id: row.doctor_user_id.map(|u| u.to_sql()),
        doctor_user_name: None,
        appointment_id: row.appointment_id.map(|a| a.to_sql()),
        title: row.title,
        document_type: row.document_type.unwrap_or_else(|| "contract".into()),
        original_pdf_url: row.original_pdf_url,
        signed_pdf_url: row.signed_pdf_url,
        status: final_status,
        signing_token: row.signing_token,
        requires_patient_signature: req_pat,
        requires_doctor_signature: req_doc,
        allow_any_dentist_signature: allow_any_doc,
        patient_signed_at: row.patient_signed_at.map(|d| d.to_rfc3339()),
        patient_signature_data: row.patient_signature_data,
        doctor_signed_at: row.doctor_signed_at.map(|d| d.to_rfc3339()),
        doctor_signature_data: row.doctor_signature_data,
        patient_otp_verified: row.patient_otp_verified.unwrap_or(false),
        checksum_sha256: row.final_checksum_sha256,
        audit_trail: row.audit_trail.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}
