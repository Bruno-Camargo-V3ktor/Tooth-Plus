use super::API_BASE;
use reqwest::Client;
use shared::patients::{
    CreatePatientExamRequest, CreatePatientRequest, CreatePatientTreatmentRequest,
    Patient, PatientAnamnesis, PatientDetailsResponse, PatientExam, PatientListResponse,
    PatientTreatment, SaveAnamnesisRequest, UpdatePatientRequest,
};

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_patients(
    token: &str,
    clinic_id: &str,
    search: Option<&str>,
) -> Result<PatientListResponse, String> {
    let mut url = format!("{}/patients?clinic_id={}", API_BASE, clinic_id);
    if let Some(s) = search {
        if !s.is_empty() {
            url.push_str(&format!("&search={}", s));
        }
    }

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao carregar pacientes.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientListResponse>()
            .await
            .map_err(|_| "Erro ao processar lista de pacientes.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao buscar pacientes.".into() } else { err })
    }
}

pub async fn create_patient(
    token: &str,
    req: CreatePatientRequest,
) -> Result<Patient, String> {
    let url = format!("{}/patients", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao cadastrar paciente.".to_string())?;

    if res.status().is_success() {
        res.json::<Patient>()
            .await
            .map_err(|_| "Erro ao processar dados do paciente criado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao cadastrar paciente.".into() } else { err })
    }
}

pub async fn fetch_patient_details(
    token: &str,
    patient_id: &str,
    clinic_id: &str,
) -> Result<PatientDetailsResponse, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}?clinic_id={}", API_BASE, clean_id, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao buscar prontuário do paciente.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientDetailsResponse>()
            .await
            .map_err(|_| "Erro ao processar prontuário do paciente.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao buscar prontuário do paciente.".into() } else { err })
    }
}

pub async fn update_patient(
    token: &str,
    patient_id: &str,
    req: UpdatePatientRequest,
) -> Result<Patient, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}", API_BASE, clean_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao atualizar paciente.".to_string())?;

    if res.status().is_success() {
        res.json::<Patient>()
            .await
            .map_err(|_| "Erro ao processar paciente atualizado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao atualizar paciente.".into() } else { err })
    }
}

pub async fn delete_patient(
    token: &str,
    patient_id: &str,
    clinic_id: &str,
) -> Result<(), String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}?clinic_id={}", API_BASE, clean_id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao excluir paciente.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao excluir paciente.".into() } else { err })
    }
}

pub async fn save_patient_anamnesis(
    token: &str,
    patient_id: &str,
    req: SaveAnamnesisRequest,
) -> Result<PatientAnamnesis, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}/anamnesis", API_BASE, clean_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao salvar anamnese.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientAnamnesis>()
            .await
            .map_err(|_| "Erro ao processar ficha de anamnese.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao salvar anamnese.".into() } else { err })
    }
}

pub async fn create_patient_exam(
    token: &str,
    patient_id: &str,
    req: CreatePatientExamRequest,
) -> Result<PatientExam, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}/exams", API_BASE, clean_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao adicionar exame.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientExam>()
            .await
            .map_err(|_| "Erro ao processar exame adicionado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao adicionar exame.".into() } else { err })
    }
}

pub async fn create_patient_treatment(
    token: &str,
    patient_id: &str,
    req: CreatePatientTreatmentRequest,
) -> Result<PatientTreatment, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}/treatments", API_BASE, clean_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao registrar tratamento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatment>()
            .await
            .map_err(|_| "Erro ao processar procedimento adicionado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() { "Erro ao registrar procedimento.".into() } else { err })
    }
}
