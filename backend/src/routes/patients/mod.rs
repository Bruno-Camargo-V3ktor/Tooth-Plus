//! # Módulo de Pacientes e Prontuário Clínico (Backend)
//!
//! Agrega sub-módulos para cadastro e manutenção de pacientes, ficha de anamnese,
//! exames e laudos, histórico de procedimentos e gestão de segurança/senhas de assinatura.

pub mod anamnesis;
pub mod crud;
pub mod photos;
pub mod security;
pub mod treatments;

pub use anamnesis::*;
pub use crud::*;
pub use photos::*;
pub use security::*;
pub use treatments::*;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use shared::patients::Patient;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Converte uma string bruta de ID no tipo `RecordId` do SurrealDB.
pub(crate) fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
}

/// Normaliza o ID da clínica para o formato prefixado `clinic:UUID`.
pub(crate) fn clinic_record_id(clinic_id: &str) -> String {
    if clinic_id.starts_with("clinic:") {
        clinic_id.to_string()
    } else {
        format!("clinic:{}", clinic_id)
    }
}

/// Linha da tabela `patient` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbPatientRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub full_name: String,
    pub document_cpf: Option<String>,
    pub document_cpf_encrypted: Option<String>,
    pub document_cpf_hash: Option<String>,
    pub document_rg: Option<String>,
    pub legal_guardian_name: Option<String>,
    pub legal_guardian_cpf: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub marital_status: Option<String>,
    pub profession: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub address_street: Option<String>,
    pub address_number: Option<String>,
    pub address_complement: Option<String>,
    pub address_neighborhood: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub address_zip: Option<String>,
    pub insurance_plan: Option<String>,
    pub insurance_number: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Linha da tabela `patient_anamnesis` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbAnamnesisRow {
    pub id: Option<RecordId>,
    pub patient_id: RecordId,
    pub clinic_id: RecordId,
    pub allergies: Option<Vec<String>>,
    pub continuous_medications: Option<String>,
    pub systemic_diseases: Option<Vec<String>>,
    pub is_pregnant: Option<bool>,
    pub has_bleeding_disorder: Option<bool>,
    pub smoker: Option<bool>,
    pub bruxism: Option<bool>,
    pub chief_complaint: Option<String>,
    pub clinical_notes: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Linha da tabela `patient_exam` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbExamRow {
    pub id: RecordId,
    pub patient_id: RecordId,
    pub clinic_id: RecordId,
    pub title: String,
    pub exam_type: String,
    pub requested_by_user_id: Option<RecordId>,
    pub status: Option<String>,
    pub requested_date: Option<DateTime<Utc>>,
    pub result_date: Option<DateTime<Utc>>,
    pub file_urls: Option<Vec<String>>,
    pub clinical_interpretation: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Linha da tabela `patient_treatment` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbTreatmentRow {
    pub id: RecordId,
    pub patient_id: RecordId,
    pub clinic_id: RecordId,
    pub dentist_user_id: Option<RecordId>,
    pub appointment_id: Option<RecordId>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub status: Option<String>,
    pub cost_cents: Option<i64>,
    pub clinical_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Linha da tabela `patient_document` para inclusão no prontuário integrado.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbDocumentRow {
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
    pub patient_signed_at: Option<DateTime<Utc>>,
    pub patient_signature_data: Option<String>,
    pub doctor_signed_at: Option<DateTime<Utc>>,
    pub doctor_signature_data: Option<String>,
    pub patient_otp_verified: Option<bool>,
    pub final_checksum_sha256: Option<String>,
    pub audit_trail: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Converte a linha de banco de dados `DbPatientRow` no modelo compartilhado `Patient`.
pub(crate) fn map_patient(row: DbPatientRow) -> Patient {
    let decrypted_cpf = if let Some(ref enc) = row.document_cpf_encrypted {
        crate::security::crypto::decrypt_deterministic(enc).unwrap_or_else(|_| {
            row.document_cpf
                .clone()
                .unwrap_or_else(|| "CPF Protegido".into())
        })
    } else {
        row.document_cpf
            .clone()
            .unwrap_or_else(|| "Não informado".into())
    };
    let has_pwd =
        row.password_hash.is_some() && !row.password_hash.as_deref().unwrap_or("").is_empty();

    Patient {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        full_name: row.full_name,
        document_cpf: decrypted_cpf,
        document_rg: row.document_rg,
        legal_guardian_name: row.legal_guardian_name,
        legal_guardian_cpf: row.legal_guardian_cpf,
        phone: row.phone,
        email: row.email,
        birth_date: row.birth_date,
        gender: row.gender,
        marital_status: row.marital_status,
        profession: row.profession,
        emergency_contact_name: row.emergency_contact_name,
        emergency_contact_phone: row.emergency_contact_phone,
        address_street: row.address_street,
        address_number: row.address_number,
        address_complement: row.address_complement,
        address_neighborhood: row.address_neighborhood,
        address_city: row.address_city,
        address_state: row.address_state,
        address_zip: row.address_zip,
        insurance_plan: row.insurance_plan,
        insurance_number: row.insurance_number,
        has_signature_password: has_pwd,
        created_at: row
            .created_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        updated_at: row
            .updated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    }
}
