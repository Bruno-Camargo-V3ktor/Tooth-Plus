use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Patient {
    pub id: String,
    pub clinic_id: String,
    pub full_name: String,
    pub document_cpf: String,
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
    pub has_signature_password: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePatientRequest {
    pub clinic_id: String,
    pub full_name: String,
    pub document_cpf: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePatientRequest {
    pub clinic_id: String,
    pub full_name: String,
    pub document_cpf: String,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePatientExamRequest {
    pub clinic_id: String,
    pub title: String,
    pub exam_type: String,
    pub requested_date: Option<String>,
    pub result_date: Option<String>,
    pub file_urls: Vec<String>,
    pub clinical_interpretation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PatientKpis {
    pub total_patients: usize,
    pub new_this_month: usize,
    pub pending_documents_count: usize,
    pub active_treatments_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientListResponse {
    pub items: Vec<Patient>,
    pub kpis: PatientKpis,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientDetailsResponse {
    pub patient: Patient,
    pub anamnesis: Option<PatientAnamnesis>,
    pub exams: Vec<PatientExam>,
    pub treatments: Vec<PatientTreatment>,
    pub documents: Vec<crate::documents::PatientDocument>,
}
