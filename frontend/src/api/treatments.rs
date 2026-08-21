//! # API Client para Tratamentos e Orçamentos
//!
//! Funções assíncronas para comunicação com o backend dos módulos de
//! Catálogo de Tratamentos (Templates) e Orçamentos de Pacientes (Treatment Plans).

use super::API_BASE;
use reqwest::Client;

fn get_client() -> Client {
    Client::new()
}
use shared::treatments::{
    CreateTreatmentPlanRequest, CreateTreatmentTemplateRequest, PatientTreatmentPlan,
    TreatmentTemplate, UpdateTreatmentPlanRequest, UpdateTreatmentPlanStatusRequest,
    UpdateTreatmentTemplateRequest,
};

fn clean_id(raw: &str) -> &str {
    let s = raw.strip_prefix("patient:").unwrap_or(raw);
    let s = s.strip_prefix("patient_treatment_plan:").unwrap_or(s);
    let s = s.strip_prefix("clinic:").unwrap_or(s);
    s.trim_matches(|c| c == '⟨' || c == '⟩')
}

// ─────────────────────────────────────────────────────────────
// Treatment Templates (Catálogo de Procedimentos Padrão)
// ─────────────────────────────────────────────────────────────

pub async fn fetch_treatment_templates(
    token: &str,
    clinic_id: &str,
) -> Result<Vec<TreatmentTemplate>, String> {
    let url = format!("{}/clinics/{}/treatment-templates", API_BASE, clean_id(clinic_id));
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
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível carregar os tratamentos padrão.".to_string()
        } else {
            err
        })
    }
}

pub async fn create_treatment_template(
    token: &str,
    clinic_id: &str,
    req: CreateTreatmentTemplateRequest,
) -> Result<TreatmentTemplate, String> {
    let url = format!("{}/clinics/{}/treatment-templates", API_BASE, clean_id(clinic_id));
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
            .map_err(|e| format!("Erro ao ler resposta do servidor: {}", e))
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível salvar o tratamento padrão.".to_string()
        } else {
            err
        })
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
        API_BASE, clean_id(clinic_id), clean_id(template_id)
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
            .map_err(|e| format!("Erro ao ler resposta do servidor: {}", e))
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível atualizar o tratamento padrão.".to_string()
        } else {
            err
        })
    }
}

pub async fn delete_treatment_template(
    token: &str,
    clinic_id: &str,
    template_id: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/clinics/{}/treatment-templates/{}",
        API_BASE, clean_id(clinic_id), clean_id(template_id)
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
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível excluir o tratamento padrão.".to_string()
        } else {
            err
        })
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
        API_BASE, clean_id(patient_id), clean_id(clinic_id)
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
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível obter os orçamentos do paciente.".to_string()
        } else {
            err
        })
    }
}

pub async fn create_treatment_plan(
    token: &str,
    patient_id: &str,
    req: CreateTreatmentPlanRequest,
) -> Result<PatientTreatmentPlan, String> {
    let url = format!("{}/patients/{}/treatment-plans", API_BASE, clean_id(patient_id));
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
            .map_err(|e| format!("Erro ao ler resposta do servidor: {}", e))
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível criar o orçamento de tratamento.".to_string()
        } else {
            err
        })
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
        API_BASE, clean_id(patient_id), clean_id(plan_id)
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
            .map_err(|e| format!("Erro ao ler resposta do servidor: {}", e))
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível atualizar o orçamento de tratamento.".to_string()
        } else {
            err
        })
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
        API_BASE, clean_id(patient_id), clean_id(plan_id)
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
            .map_err(|e| format!("Erro ao ler resposta do servidor: {}", e))
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível atualizar o status do orçamento.".to_string()
        } else {
            err
        })
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
        API_BASE, clean_id(patient_id), clean_id(plan_id), clean_id(clinic_id)
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
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível excluir o orçamento de tratamento.".to_string()
        } else {
            err
        })
    }
}

pub async fn pay_treatment_plan(
    token: &str,
    patient_id: &str,
    plan_id: &str,
    req: shared::finance::RegisterPaymentRequest,
) -> Result<PatientTreatmentPlan, String> {
    let url = format!(
        "{}/patients/{}/treatment-plans/{}/pay",
        API_BASE, clean_id(patient_id), clean_id(plan_id)
    );
    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor ao registrar pagamento do orçamento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatmentPlan>()
            .await
            .map_err(|e| format!("Erro ao ler resposta do pagamento: {}", e))
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Não foi possível registrar o pagamento do orçamento.".into()
        } else {
            err
        })
    }
}
