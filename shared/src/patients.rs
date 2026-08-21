//! # Modelos de Domínio - Pacientes
//!
//! Este módulo define as estruturas de dados para gestão de pacientes odontológicos,
//! incluindo prontuário integrado, ficha de anamnese médica, exames com laudos/fotos,
//! histórico de procedimentos e tratamentos clínicos.

use serde::{Deserialize, Serialize};
use crate::anamnesis::AnamnesisResponseItem;

/// Responsável legal pelo paciente (obrigatório para menores de 18 anos ou incapazes).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PatientGuardian {
    pub name: String,
    pub document_cpf: Option<String>,
    pub document_rg: Option<String>,
    pub relationship: String, // "Pai", "Mãe", "Tutor Legal", "Avô/Avó", "Outro"
    pub phone: String,
    pub email: Option<String>,
}

/// Representação completa de um paciente cadastrado na clínica.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Patient {
    /// Identificador único do paciente (ex: `patient:ulid`).
    pub id: String,
    /// Identificador da clínica à qual o paciente pertence.
    pub clinic_id: String,
    /// Nome completo do paciente.
    pub full_name: String,
    /// CPF descriptografado / mascarado para exibição autorizada.
    pub document_cpf: Option<String>,
    /// Registro Geral (RG), alternativo ou complementar ao CPF.
    pub document_rg: Option<String>,
    /// Lista de responsáveis legais cadastrados (especialmente para menores).
    pub legal_guardians: Vec<PatientGuardian>,
    /// Nome do responsável legal legado.
    pub legal_guardian_name: Option<String>,
    /// CPF do responsável legal legado.
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
    pub document_cpf: Option<String>,
    pub document_rg: Option<String>,
    pub legal_guardians: Option<Vec<PatientGuardian>>,
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
}

/// Requisição para atualização cadastral do paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdatePatientRequest {
    pub clinic_id: String,
    pub full_name: String,
    pub document_cpf: Option<String>,
    pub document_rg: Option<String>,
    pub legal_guardians: Option<Vec<PatientGuardian>>,
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
}

/// Ficha médica e histórico de saúde (Anamnese Odontológica).
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PatientAnamnesis {
    pub id: Option<String>,
    pub patient_id: String,
    pub clinic_id: String,
    pub template_type: Option<String>,
    pub custom_responses: Vec<AnamnesisResponseItem>,
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
    pub signature_status: Option<String>,
    pub signing_token: Option<String>,
    pub signed_at: Option<String>,
    pub signed_pdf_url: Option<String>,
}

/// Requisição para salvar ou atualizar a ficha de anamnese do paciente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SaveAnamnesisRequest {
    pub clinic_id: String,
    pub template_type: Option<String>,
    pub custom_responses: Option<Vec<AnamnesisResponseItem>>,
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

/// Requisição para atualizar um exame existente no prontuário.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdatePatientExamRequest {
    pub clinic_id: String,
    pub title: String,
    pub exam_type: String,
    pub status: Option<String>,
    pub requested_date: Option<String>,
    pub result_date: Option<String>,
    pub file_urls: Vec<String>,
    pub clinical_interpretation: Option<String>,
}

/// Procedimento ou evolução odontológica registrada no prontuário.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PatientTreatment {
    pub id: String,
    pub patient_id: String,
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub dentist_user_name: Option<String>,
    pub appointment_id: Option<String>,
    #[serde(default)]
    pub appointment_date: Option<String>,
    pub document_id: Option<String>,
    pub exam_id: Option<String>,
    /// Vínculo com o plano/orçamento de tratamento gerador.
    pub treatment_plan_id: Option<String>,
    /// Vínculo com o item individual do orçamento.
    #[serde(default)]
    pub treatment_plan_item_id: Option<String>,
    /// ID da transação financeira vinculada.
    pub transaction_id: Option<String>,
    /// Status financeiro herdado do orçamento pai ou transação: "unpaid", "partial", "paid".
    #[serde(default)]
    pub financial_status: Option<String>,
    pub procedure_category: Option<String>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub surfaces: Option<Vec<String>>,
    pub materials_used: Option<Vec<String>>,
    pub status: String,
    pub cost_cents: i64,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
    pub performed_at: Option<String>,
    pub created_at: String,
}

/// Requisição para cadastrar um novo procedimento odontológico / evolução.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreatePatientTreatmentRequest {
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub appointment_id: Option<String>,
    pub document_id: Option<String>,
    pub exam_id: Option<String>,
    pub treatment_plan_id: Option<String>,
    #[serde(default)]
    pub treatment_plan_item_id: Option<String>,
    pub transaction_id: Option<String>,
    #[serde(default)]
    pub financial_status: Option<String>,
    pub procedure_category: Option<String>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub surfaces: Option<Vec<String>>,
    pub materials_used: Option<Vec<String>>,
    pub status: String,
    pub cost_cents: i64,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
    pub performed_at: Option<String>,
}

/// Requisição para atualizar um procedimento odontológico / evolução existente.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UpdatePatientTreatmentRequest {
    pub clinic_id: String,
    pub dentist_user_id: Option<String>,
    pub appointment_id: Option<String>,
    pub document_id: Option<String>,
    pub exam_id: Option<String>,
    pub treatment_plan_id: Option<String>,
    #[serde(default)]
    pub treatment_plan_item_id: Option<String>,
    pub transaction_id: Option<String>,
    #[serde(default)]
    pub financial_status: Option<String>,
    pub procedure_category: Option<String>,
    pub procedure_name: String,
    pub tooth_number: Option<String>,
    pub surfaces: Option<Vec<String>>,
    pub materials_used: Option<Vec<String>>,
    pub status: String,
    pub cost_cents: i64,
    pub post_care_instructions: Option<String>,
    pub clinical_notes: Option<String>,
    pub performed_at: Option<String>,
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
    pub treatment_plans: Vec<crate::treatments::PatientTreatmentPlan>,
    pub documents: Vec<crate::documents::PatientDocument>,
}

