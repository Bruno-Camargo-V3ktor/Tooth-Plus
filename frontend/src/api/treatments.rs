//! # Cliente de API — Tratamentos Padrão e Orçamentos Clínicos
//!
//! Funções para comunicação com os endpoints de templates de tratamento
//! e planos de tratamento / orçamentos de pacientes.

use super::API_BASE;
use reqwest::Client;
use shared::treatments::{
    CreateTreatmentPlanRequest, CreateTreatmentTemplateRequest, PatientTreatmentPlan,
    TreatmentTemplate, UpdateTreatmentPlanRequest, UpdateTreatmentPlanStatusRequest,
    UpdateTreatmentTemplateRequest,
};

fn get_client() -> Client {
    Client::new()
}

// ─────────────────────────────────────────────────────────────
// Treatment Templates (Catálogo da Clínica)
// ─────────────────────────────────────────────────────────────

pub async fn fetch_treatment_templates(
    token: &str,
    clinic_id: &str,
) -> Result<Vec<TreatmentTemplate>, String> {
    let url = format!("{}/clinics/{}/treatment-templates", API_BASE, clinic_id);
    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha ao conectar com o servidor para listar tratamentos.".to_string())?;

    if res.status().is_success() {
        res.json::<Vec<TreatmentTemplate>>()
            .await
            .map_err(|e| format!("Erro ao processar catálogo de tratamentos: {}", e))
    } else {
        Err("Não foi possível carregar os tratamentos padrão.".to_string())
    }
}

pub async fn create_treatment_template(
    token: &str,
    clinic_id: &str,
    req: CreateTreatmentTemplateRequest,
) -> Result<TreatmentTemplate, String> {
    let url = format!("{}/clinics/{}/treatment-templates", API_BASE, clinic_id);
    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor.".to_string())?;

    if res.status().is_success() {
        res.json::<TreatmentTemplate>()
            .await
            .map_err(|_| "Erro ao ler resposta do servidor.".to_string())
    } else {
        Err("Não foi possível salvar o tratamento padrão.".to_string())
    }
}

pub async fn update_treatment_template(
    token: &str,
    clinic_id: &str,
    template_id: &str,
    req: UpdateTreatmentTemplateRequest,
) -> Result<TreatmentTemplate, String> {
    let url = format!(
        "{}/clinics/{}/treatment-templates/{}",
        API_BASE, clinic_id, template_id
    );
    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor.".to_string())?;

    if res.status().is_success() {
        res.json::<TreatmentTemplate>()
            .await
            .map_err(|_| "Erro ao ler resposta do servidor.".to_string())
    } else {
        Err("Não foi possível atualizar o tratamento padrão.".to_string())
    }
}

pub async fn delete_treatment_template(
    token: &str,
    clinic_id: &str,
    template_id: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/clinics/{}/treatment-templates/{}",
        API_BASE, clinic_id, template_id
    );
    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível excluir o tratamento padrão.".to_string())
    }
}

// ─────────────────────────────────────────────────────────────
// Patient Treatment Plans (Orçamentos)
// ─────────────────────────────────────────────────────────────

pub async fn fetch_patient_treatment_plans(
    token: &str,
    clinic_id: &str,
    patient_id: &str,
) -> Result<Vec<PatientTreatmentPlan>, String> {
    let url = format!(
        "{}/patients/{}/treatment-plans?clinic_id={}",
        API_BASE, patient_id, clinic_id
    );
    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha ao carregar planos de tratamento do paciente.".to_string())?;

    if res.status().is_success() {
        res.json::<Vec<PatientTreatmentPlan>>()
            .await
            .map_err(|e| format!("Erro ao processar orçamentos: {}", e))
    } else {
        Err("Não foi possível obter os orçamentos do paciente.".to_string())
    }
}

pub async fn create_treatment_plan(
    token: &str,
    patient_id: &str,
    req: CreateTreatmentPlanRequest,
) -> Result<PatientTreatmentPlan, String> {
    let url = format!("{}/patients/{}/treatment-plans", API_BASE, patient_id);
    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor ao criar orçamento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatmentPlan>()
            .await
            .map_err(|_| "Erro ao ler resposta do servidor.".to_string())
    } else {
        Err("Não foi possível criar o orçamento de tratamento.".to_string())
    }
}

pub async fn update_treatment_plan(
    token: &str,
    patient_id: &str,
    plan_id: &str,
    req: UpdateTreatmentPlanRequest,
) -> Result<PatientTreatmentPlan, String> {
    let url = format!(
        "{}/patients/{}/treatment-plans/{}",
        API_BASE, patient_id, plan_id
    );
    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor ao atualizar orçamento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatmentPlan>()
            .await
            .map_err(|_| "Erro ao ler resposta do servidor.".to_string())
    } else {
        Err("Não foi possível atualizar o orçamento de tratamento.".to_string())
    }
}

pub async fn update_treatment_plan_status(
    token: &str,
    patient_id: &str,
    plan_id: &str,
    req: UpdateTreatmentPlanStatusRequest,
) -> Result<PatientTreatmentPlan, String> {
    let url = format!(
        "{}/patients/{}/treatment-plans/{}/status",
        API_BASE, patient_id, plan_id
    );
    let res = get_client()
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor ao alterar status.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatmentPlan>()
            .await
            .map_err(|_| "Erro ao ler resposta do servidor.".to_string())
    } else {
        Err("Não foi possível atualizar o status do orçamento.".to_string())
    }
}

pub async fn delete_treatment_plan(
    token: &str,
    clinic_id: &str,
    patient_id: &str,
    plan_id: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/patients/{}/treatment-plans/{}?clinic_id={}",
        API_BASE, patient_id, plan_id, clinic_id
    );
    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível excluir o orçamento de tratamento.".to_string())
    }
}
