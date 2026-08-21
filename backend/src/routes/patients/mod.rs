//! # Módulo de Pacientes e Prontuário Clínico (Backend)
//!
//! Agrega sub-módulos para cadastro e manutenção de pacientes, ficha de anamnese,
//! exames e laudos, histórico de procedimentos e gestão de segurança/senhas de assinatura.

pub mod anamnesis;
pub mod crud;
pub mod photos;
pub mod security;
pub mod treatment_plans;
pub mod treatments;

pub use anamnesis::*;
pub use crud::*;
pub use photos::*;
pub use security::*;
pub use treatment_plans::*;
pub use treatments::*;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use shared::patients::Patient;
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
    pub legal_guardians: Option<serde_json::Value>,
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

/// Linha da tabela `anamnesis_template` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbAnamnesisTemplateRow {
    pub id: RecordId,
    pub clinic_id: RecordId,
    pub template_type: String,
    pub title: String,
    pub questions: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Linha da tabela `patient_anamnesis` no banco de dados.
#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbAnamnesisRow {
    pub id: Option<RecordId>,
    pub patient_id: RecordId,
    pub clinic_id: RecordId,
    pub template_type: Option<String>,
    pub custom_responses: Option<serde_json::Value>,
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
    pub signature_status: Option<String>,
    pub signing_token: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub signed_pdf_url: Option<String>,
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
    pub document_id: Option<RecordId>,
    pub exam_id: Option<RecordId>,
    pub treatment_plan_id: Option<RecordId>,
    #[serde(default)]
    pub treatment_plan_item_id: Option<String>,
    pub transaction_id: Option<RecordId>,
    #[serde(default)]
    pub financial_status: Option<String>,
    pub procedure_category: Option<String>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub surfaces: Option<Vec<String>>,
    pub materials_used: Option<Vec<String>>,
    pub status: Option<String>,
    pub cost_cents: Option<i64>,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
    pub performed_at: Option<DateTime<Utc>>,
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
    pub requires_patient_signature: Option<bool>,
    pub requires_doctor_signature: Option<bool>,
    pub allow_any_dentist_signature: Option<bool>,
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


/// Converte a linha de banco de dados `DbPatientRow` no modelo compartilhado `Patient` descriptografando documentos sem mascaramento.
pub(crate) fn map_patient(row: DbPatientRow) -> Patient {
    let plain_cpf = if let Some(ref enc) = row.document_cpf_encrypted {
        let dec = crate::security::crypto::decrypt_deterministic(enc).unwrap_or_else(|_| {
            row.document_cpf.clone().unwrap_or_default()
        });
        if dec.is_empty() {
            None
        } else {
            Some(dec)
        }
    } else if let Some(ref raw_cpf) = row.document_cpf {
        if raw_cpf.is_empty() {
            None
        } else {
            Some(raw_cpf.clone())
        }
    } else {
        None
    };

    let plain_rg = row.document_rg;

    let has_pwd =
        row.password_hash.is_some() && !row.password_hash.as_deref().unwrap_or("").is_empty();

    let mut raw_guardian_cpfs: Vec<String> = Vec::new();
    let mut guardians: Vec<shared::patients::PatientGuardian> = row
        .legal_guardians
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    for g in &mut guardians {
        if let Some(ref cpf) = g.document_cpf {
            let real_val = crate::security::crypto::decrypt_deterministic(cpf).unwrap_or_else(|_| cpf.clone());
            if !real_val.is_empty() {
                raw_guardian_cpfs.push(real_val.clone());
                g.document_cpf = Some(real_val);
            }
        }
        if let Some(ref rg) = g.document_rg {
            let real_val = crate::security::crypto::decrypt_deterministic(rg).unwrap_or_else(|_| rg.clone());
            if !real_val.is_empty() {
                g.document_rg = Some(real_val);
            }
        }
    }

    let decrypted_guardian_cpf = row.legal_guardian_cpf.and_then(|s| {
        crate::security::crypto::decrypt_deterministic(&s).ok().or(Some(s))
    });

    let final_cpf = plain_cpf.or_else(|| {
        decrypted_guardian_cpf.clone().or_else(|| raw_guardian_cpfs.first().cloned())
    });

    Patient {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        full_name: row.full_name,
        document_cpf: final_cpf,
        document_rg: plain_rg,
        legal_guardians: guardians,
        legal_guardian_name: row.legal_guardian_name,
        legal_guardian_cpf: decrypted_guardian_cpf,
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

/// Converte a linha de banco de dados `DbAnamnesisRow` no modelo compartilhado `PatientAnamnesis`.
pub(crate) fn map_anamnesis(a: DbAnamnesisRow) -> shared::patients::PatientAnamnesis {
    shared::patients::PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
        template_type: a.template_type,
        custom_responses: a
            .custom_responses
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default(),
        allergies: a.allergies.unwrap_or_default(),
        continuous_medications: a.continuous_medications,
        systemic_diseases: a.systemic_diseases.unwrap_or_default(),
        is_pregnant: a.is_pregnant.unwrap_or(false),
        has_bleeding_disorder: a.has_bleeding_disorder.unwrap_or(false),
        smoker: a.smoker.unwrap_or(false),
        bruxism: a.bruxism.unwrap_or(false),
        chief_complaint: a.chief_complaint,
        clinical_notes: a.clinical_notes,
        updated_at: a
            .updated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        signature_status: a.signature_status,
        signing_token: a.signing_token,
        signed_at: a.signed_at.map(|d| d.to_rfc3339()),
        signed_pdf_url: a.signed_pdf_url,
    }
}

