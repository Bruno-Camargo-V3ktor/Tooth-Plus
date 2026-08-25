//! # Módulo de Integração e Serviço de Pacientes (PatientsApi)

use super::mock_db::DB;
use shared::patients::{
    CreatePatientExamRequest, CreatePatientRequest, CreatePatientTreatmentRequest,
    Patient, PatientAnamnesis, PatientDetailsResponse, PatientExam, PatientKpis,
    PatientListResponse, PatientTreatment, SaveAnamnesisRequest,
    UpdatePatientRequest,
};

pub struct PatientsApi;

impl PatientsApi {
    /// Lista pacientes com suporte a busca textual e cálculo de KPIs.
    pub async fn list_patients(search: Option<&str>) -> Result<PatientListResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let filtered: Vec<Patient> = match search {
            Some(query) if !query.trim().is_empty() => {
                let q = query.trim().to_lowercase();
                db.patients
                    .iter()
                    .filter(|p| {
                        p.full_name.to_lowercase().contains(&q)
                            || p.phone.contains(&q)
                            || p.document_cpf.as_deref().unwrap_or("").contains(&q)
                    })
                    .cloned()
                    .collect()
            }
            _ => db.patients.clone(),
        };

        let total = filtered.len();
        let kpis = PatientKpis {
            total_patients: total,
            new_this_month: total.saturating_sub(1),
            pending_documents_count: 1,
            active_treatments_count: db.appointments.len(),
        };

        Ok(PatientListResponse {
            items: filtered,
            kpis,
            total,
        })
    }

    /// Obtém um paciente por ID.
    pub async fn get_patient_by_id(patient_id: &str) -> Result<Patient, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        db.patients
            .iter()
            .find(|p| p.id == patient_id)
            .cloned()
            .ok_or_else(|| format!("Paciente {} não encontrado.", patient_id))
    }

    /// Obtém o prontuário clínico detalhado e completo do paciente.
    pub async fn get_patient_details(patient_id: &str) -> Result<PatientDetailsResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let patient = db
            .patients
            .iter()
            .find(|p| p.id == patient_id)
            .cloned()
            .ok_or_else(|| format!("Paciente {} não encontrado.", patient_id))?;

        let anamnesis = db
            .anamneses
            .iter()
            .find(|a| a.patient_id == patient_id)
            .cloned();

        let exams = db
            .exams
            .iter()
            .filter(|e| e.patient_id == patient_id)
            .cloned()
            .collect();

        let treatments = db
            .patient_treatments
            .iter()
            .filter(|t| t.patient_id == patient_id)
            .cloned()
            .collect();

        let treatment_plans = db
            .treatment_plans
            .iter()
            .filter(|tp| tp.patient_id == patient_id)
            .cloned()
            .collect();

        Ok(PatientDetailsResponse {
            patient,
            anamnesis,
            exams,
            treatments,
            treatment_plans,
            documents: vec![],
        })
    }

    /// Cadastra um novo paciente.
    pub async fn create_patient(req: CreatePatientRequest) -> Result<Patient, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let clean_id = format!(
            "patient:{}",
            req.full_name
                .to_lowercase()
                .replace(' ', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        );

        let new_patient = Patient {
            id: clean_id,
            clinic_id: req.clinic_id,
            full_name: req.full_name,
            document_cpf: req.document_cpf,
            document_rg: req.document_rg,
            phone: req.phone,
            email: req.email,
            birth_date: req.birth_date,
            gender: req.gender,
            marital_status: req.marital_status,
            profession: req.profession,
            emergency_contact_name: req.emergency_contact_name,
            emergency_contact_phone: req.emergency_contact_phone,
            address_street: req.address_street,
            address_number: req.address_number,
            address_complement: req.address_complement,
            address_neighborhood: req.address_neighborhood,
            address_city: req.address_city,
            address_state: req.address_state,
            address_zip: req.address_zip,
            insurance_plan: req.insurance_plan,
            insurance_number: req.insurance_number,
            legal_guardians: vec![],
            legal_guardian_name: req.legal_guardian_name,
            legal_guardian_cpf: req.legal_guardian_cpf,
            has_signature_password: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        db.patients.insert(0, new_patient.clone());
        Ok(new_patient)
    }

    /// Atualiza dados cadastrais de um paciente existente.
    pub async fn update_patient(patient_id: &str, req: UpdatePatientRequest) -> Result<Patient, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let patient = db
            .patients
            .iter_mut()
            .find(|p| p.id == patient_id)
            .ok_or_else(|| format!("Paciente {} não encontrado.", patient_id))?;

        patient.full_name = req.full_name;
        patient.phone = req.phone;
        if let Some(cpf) = req.document_cpf { patient.document_cpf = Some(cpf); }
        if let Some(rg) = req.document_rg { patient.document_rg = Some(rg); }
        if let Some(em) = req.email { patient.email = Some(em); }
        if let Some(bd) = req.birth_date { patient.birth_date = Some(bd); }
        if let Some(g) = req.gender { patient.gender = Some(g); }
        if let Some(ms) = req.marital_status { patient.marital_status = Some(ms); }
        if let Some(prof) = req.profession { patient.profession = Some(prof); }
        if let Some(ecn) = req.emergency_contact_name { patient.emergency_contact_name = Some(ecn); }
        if let Some(ecp) = req.emergency_contact_phone { patient.emergency_contact_phone = Some(ecp); }
        if let Some(st) = req.address_street { patient.address_street = Some(st); }
        if let Some(num) = req.address_number { patient.address_number = Some(num); }
        if let Some(comp) = req.address_complement { patient.address_complement = Some(comp); }
        if let Some(nb) = req.address_neighborhood { patient.address_neighborhood = Some(nb); }
        if let Some(ct) = req.address_city { patient.address_city = Some(ct); }
        if let Some(st) = req.address_state { patient.address_state = Some(st); }
        if let Some(zip) = req.address_zip { patient.address_zip = Some(zip); }
        if let Some(ip) = req.insurance_plan { patient.insurance_plan = Some(ip); }
        if let Some(in_num) = req.insurance_number { patient.insurance_number = Some(in_num); }
        if let Some(lgn) = req.legal_guardian_name { patient.legal_guardian_name = Some(lgn); }
        if let Some(lgc) = req.legal_guardian_cpf { patient.legal_guardian_cpf = Some(lgc); }

        patient.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(patient.clone())
    }

    /// Remove um paciente do cadastro.
    pub async fn delete_patient(patient_id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let initial_len = db.patients.len();
        db.patients.retain(|p| p.id != patient_id);

        if db.patients.len() == initial_len {
            return Err(format!("Paciente {} não encontrado para exclusão.", patient_id));
        }

        Ok(())
    }

    /// Salva ou atualiza a anamnese do paciente.
    pub async fn save_anamnesis(patient_id: &str, req: SaveAnamnesisRequest) -> Result<PatientAnamnesis, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let anamnesis = PatientAnamnesis {
            id: Some(format!("anamnesis:{}", patient_id.replace("patient:", ""))),
            patient_id: patient_id.to_string(),
            clinic_id: req.clinic_id,
            template_type: req.template_type,
            custom_responses: req.custom_responses.unwrap_or_default(),
            allergies: req.allergies,
            continuous_medications: req.continuous_medications,
            systemic_diseases: req.systemic_diseases,
            is_pregnant: req.is_pregnant,
            has_bleeding_disorder: req.has_bleeding_disorder,
            smoker: req.smoker,
            bruxism: req.bruxism,
            chief_complaint: req.chief_complaint,
            clinical_notes: req.clinical_notes,
            updated_at: chrono::Utc::now().to_rfc3339(),
            signature_status: Some("pending".to_string()),
            signing_token: None,
            signed_at: None,
            signed_pdf_url: None,
        };

        if let Some(existing) = db.anamneses.iter_mut().find(|a| a.patient_id == patient_id) {
            *existing = anamnesis.clone();
        } else {
            db.anamneses.push(anamnesis.clone());
        }

        Ok(anamnesis)
    }

    /// Adiciona um exame complementar ao prontuário.
    pub async fn create_exam(patient_id: &str, req: CreatePatientExamRequest) -> Result<PatientExam, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let exam = PatientExam {
            id: format!("exam:{}", db.exams.len() + 1),
            patient_id: patient_id.to_string(),
            clinic_id: req.clinic_id,
            title: req.title,
            exam_type: req.exam_type,
            requested_by_user_id: None,
            requested_by_user_name: Some("Dr. Roberto Alencar".to_string()),
            status: "concluido".to_string(),
            requested_date: req.requested_date.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            result_date: req.result_date,
            file_urls: req.file_urls,
            clinical_interpretation: req.clinical_interpretation,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        db.exams.push(exam.clone());
        Ok(exam)
    }

    /// Registra uma nova evolução / procedimento clínico no prontuário.
    pub async fn create_treatment(patient_id: &str, req: CreatePatientTreatmentRequest) -> Result<PatientTreatment, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let treatment = PatientTreatment {
            id: format!("treat:{}", db.patient_treatments.len() + 1),
            patient_id: patient_id.to_string(),
            clinic_id: req.clinic_id,
            dentist_user_id: req.dentist_user_id,
            dentist_user_name: Some("Dr. Lucas Mendes".to_string()),
            appointment_id: req.appointment_id,
            appointment_date: None,
            document_id: req.document_id,
            exam_id: req.exam_id,
            treatment_plan_id: req.treatment_plan_id,
            treatment_plan_item_id: req.treatment_plan_item_id,
            transaction_id: req.transaction_id,
            financial_status: req.financial_status,
            procedure_category: req.procedure_category,
            procedure_name: req.procedure_name,
            tooth_number: req.tooth_number,
            surfaces: req.surfaces,
            materials_used: req.materials_used,
            status: req.status,
            cost_cents: req.cost_cents,
            post_care_instructions: req.post_care_instructions,
            clinical_notes: req.clinical_notes,
            performed_at: req.performed_at,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        db.patient_treatments.push(treatment.clone());
        Ok(treatment)
    }
}
