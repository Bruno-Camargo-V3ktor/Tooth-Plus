//! # Modelos de Domínio - Pacientes
//!
//! Este módulo define as estruturas de dados para gestão de pacientes odontológicos,
//! incluindo prontuário integrado, ficha de anamnese médica, exames com laudos/fotos,
//! histórico de procedimentos e tratamentos clínicos.

use serde::{Deserialize, Serialize};

/// Representação completa de um paciente cadastrado na clínica.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Patient {
    /// Identificador único do paciente (ex: `patient:ulid`).
    pub id: String,
    /// Identificador da clínica à qual o paciente pertence.
    pub clinic_id: String,
    /// Nome completo do paciente.
    pub full_name: String,
    /// CPF descriptografado para exibição autorizada.
    pub document_cpf: String,
    /// Registro Geral (RG), alternativo ou complementar ao CPF.
    pub document_rg: Option<String>,
    /// Nome do responsável legal (para menores ou incapazes).
    pub legal_guardian_name: Option<String>,
    /// CPF do responsável legal.
    pub legal_guardian_cpf: Option<String>,
    /// Telefone principal / WhatsApp para contato e disparo de OTP.
    pub phone: String,
    /// E-mail do paciente.
    pub email: Option<String>,
    /// Data de nascimento no formato YYYY-MM-DD.
    pub birth_date: Option<String>,
    /// Gênero biológico ou identidade de gênero.
    pub gender: Option<String>,
    /// Estado civil do paciente.
    pub marital_status: Option<String>,
    /// Ocupação / Profissão.
    pub profession: Option<String>,
    /// Nome do contato de emergência.
    pub emergency_contact_name: Option<String>,
    /// Telefone do contato de emergência.
    pub emergency_contact_phone: Option<String>,
    /// Logradouro do endereço.
    pub address_street: Option<String>,
    /// Número do endereço.
    pub address_number: Option<String>,
    /// Complemento do endereço.
    pub address_complement: Option<String>,
    /// Bairro.
    pub address_neighborhood: Option<String>,
    /// Município / Cidade.
    pub address_city: Option<String>,
    /// Estado / UF (ex: SP).
    pub address_state: Option<String>,
    /// Código de Endereçamento Postal (CEP).
    pub address_zip: Option<String>,
    /// Nome da operadora ou plano de saúde/odontológico.
    pub insurance_plan: Option<String>,
    /// Número da carteirinha do convênio.
    pub insurance_number: Option<String>,
    /// Indica se o paciente já definiu sua senha de assinatura digital de 6 dígitos.
    pub has_signature_password: bool,
    /// Data e hora de criação do registro no banco.
    pub created_at: String,
    /// Data e hora da última atualização cadastral.
    pub updated_at: String,
}

/// Requisição para cadastro de novo paciente na clínica.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreatePatientRequest {
    pub clinic_id: String,
    pub full_name: String,
    pub document_cpf: String,
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
    pub signature_password: Option<String>,
}

/// Requisição para atualização cadastral do paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdatePatientRequest {
    pub clinic_id: String,
    pub full_name: String,
    pub document_cpf: String,
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
    pub new_signature_password: Option<String>,
}

/// Ficha médica e histórico de saúde (Anamnese Odontológica).
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PatientAnamnesis {
    pub id: Option<String>,
    pub patient_id: String,
    pub clinic_id: String,
    pub allergies: Vec<String>,
    pub continuous_medications: Option<String>,
    pub systemic_diseases: Vec<String>,
    pub is_pregnant: bool,
    pub has_bleeding_disorder: bool,
    pub smoker: bool,
    pub bruxism: bool,
    pub chief_complaint: Option<String>,
    pub clinical_notes: Option<String>,
    pub updated_at: String,
}

/// Requisição para salvar ou atualizar a ficha de anamnese do paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SaveAnamnesisRequest {
    pub clinic_id: String,
    pub allergies: Vec<String>,
    pub continuous_medications: Option<String>,
    pub systemic_diseases: Vec<String>,
    pub is_pregnant: bool,
    pub has_bleeding_disorder: bool,
    pub smoker: bool,
    pub bruxism: bool,
    pub chief_complaint: Option<String>,
    pub clinical_notes: Option<String>,
}

/// Registro de exame complementar (radiografias, tomografias, fotos intraorais).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientExam {
    pub id: String,
    pub patient_id: String,
    pub clinic_id: String,
    pub title: String,
    pub exam_type: String,
    pub requested_by_user_id: Option<String>,
    pub requested_by_user_name: Option<String>,
    pub status: String,
    pub requested_date: String,
    pub result_date: Option<String>,
    pub file_urls: Vec<String>,
    pub clinical_interpretation: Option<String>,
    pub created_at: String,
}

/// Requisição para registrar um novo exame ou anexar arquivos ao prontuário.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreatePatientExamRequest {
    pub clinic_id: String,
    pub title: String,
    pub exam_type: String,
    pub requested_date: Option<String>,
    pub result_date: Option<String>,
    pub file_urls: Vec<String>,
    pub clinical_interpretation: Option<String>,
}

/// Procedimento ou tratamento odontológico registrado no prontuário/odontograma.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientTreatment {
    pub id: String,
    pub patient_id: String,
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub dentist_user_name: Option<String>,
    pub appointment_id: Option<String>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub status: String,
    pub cost_cents: i64,
    pub clinical_notes: Option<String>,
    pub created_at: String,
}

/// Requisição para cadastrar um novo procedimento odontológico.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreatePatientTreatmentRequest {
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub appointment_id: Option<String>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub status: String,
    pub cost_cents: i64,
    pub clinical_notes: Option<String>,
}

/// Indicadores quantitativos (KPIs) exibidos no topo da listagem de pacientes.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PatientKpis {
    pub total_patients: usize,
    pub new_this_month: usize,
    pub pending_documents_count: usize,
    pub active_treatments_count: usize,
}

/// Resposta da listagem paginada de pacientes com métricas agregadas.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientListResponse {
    pub items: Vec<Patient>,
    pub kpis: PatientKpis,
    pub total: usize,
}

/// Prontuário clínico unificado e completo do paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientDetailsResponse {
    pub patient: Patient,
    pub anamnesis: Option<PatientAnamnesis>,
    pub exams: Vec<PatientExam>,
    pub treatments: Vec<PatientTreatment>,
    pub documents: Vec<crate::documents::PatientDocument>,
}
