use super::API_BASE;
use reqwest::Client;
use shared::patients::{
    CreatePatientExamRequest, CreatePatientRequest, CreatePatientTreatmentRequest, Patient,
    PatientAnamnesis, PatientDetailsResponse, PatientExam, PatientListResponse, PatientTreatment,
    SaveAnamnesisRequest, UpdatePatientExamRequest, UpdatePatientRequest,
    UpdatePatientTreatmentRequest,
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
        Err(if err.is_empty() {
            "Erro ao buscar pacientes.".into()
        } else {
            err
        })
    }
}

pub async fn create_patient(token: &str, req: CreatePatientRequest) -> Result<Patient, String> {
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
        Err(if err.is_empty() {
            "Erro ao cadastrar paciente.".into()
        } else {
            err
        })
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
        Err(if err.is_empty() {
            "Erro ao buscar prontuário do paciente.".into()
        } else {
            err
        })
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
        Err(if err.is_empty() {
            "Erro ao atualizar paciente.".into()
        } else {
            err
        })
    }
}

pub async fn delete_patient(token: &str, patient_id: &str, clinic_id: &str) -> Result<(), String> {
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
        Err(if err.is_empty() {
            "Erro ao excluir paciente.".into()
        } else {
            err
        })
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
        Err(if err.is_empty() {
            "Erro ao salvar anamnese.".into()
        } else {
            err
        })
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
        Err(if err.is_empty() {
            "Erro ao adicionar exame.".into()
        } else {
            err
        })
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
        .map_err(|_| "Falha de conexão ao registrar procedimento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatment>()
            .await
            .map_err(|_| "Erro ao processar procedimento adicionado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao registrar procedimento.".into()
        } else {
            err
        })
    }
}

pub async fn reset_patient_signature_password(
    token: &str,
    patient_id: &str,
    clinic_id: &str,
) -> Result<String, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}/reset-password?clinic_id={}", API_BASE, clean_id, clinic_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao resetar senha do paciente.".to_string())?;

    if res.status().is_success() {
        #[derive(serde::Deserialize)]
        struct Resp {
            message: String,
        }
        let body = res.json::<Resp>().await.map_err(|_| "Erro ao processar resposta.".to_string())?;
        Ok(body.message)
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao resetar senha do paciente.".into()
        } else {
            err
        })
    }
}

pub async fn fetch_anamnesis_templates(
    token: &str,
    clinic_id: &str,
) -> Result<Vec<shared::anamnesis::AnamnesisTemplate>, String> {
    let url = format!("{}/clinics/{}/anamnesis-templates", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao buscar modelos de anamnese.".to_string())?;

    if res.status().is_success() {
        res.json::<Vec<shared::anamnesis::AnamnesisTemplate>>()
            .await
            .map_err(|_| "Erro ao processar modelos de anamnese.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao carregar modelos de anamnese.".into()
        } else {
            err
        })
    }
}

pub async fn save_anamnesis_template(
    token: &str,
    req: shared::anamnesis::SaveAnamnesisTemplateRequest,
) -> Result<shared::anamnesis::AnamnesisTemplate, String> {
    let url = format!("{}/clinics/{}/anamnesis-templates", API_BASE, req.clinic_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao salvar modelo de anamnese.".to_string())?;

    if res.status().is_success() {
        res.json::<shared::anamnesis::AnamnesisTemplate>()
            .await
            .map_err(|_| "Erro ao processar modelo de anamnese salvo.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao salvar modelo de anamnese.".into()
        } else {
            err
        })
    }
}

pub async fn sync_patient_anamnesis(
    token: &str,
    patient_id: &str,
    req: shared::anamnesis::SyncAnamnesisRequest,
) -> Result<PatientAnamnesis, String> {
    let clean_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let url = format!("{}/patients/{}/anamnesis/sync", API_BASE, clean_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao sincronizar anamnese.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientAnamnesis>()
            .await
            .map_err(|_| "Erro ao processar ficha de anamnese sincronizada.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao sincronizar anamnese.".into()
        } else {
            err
        })
    }
}

pub async fn update_patient_treatment(
    token: &str,
    patient_id: &str,
    treatment_id: &str,
    req: UpdatePatientTreatmentRequest,
) -> Result<PatientTreatment, String> {
    let clean_p_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let clean_t_id = treatment_id.strip_prefix("patient_treatment:").unwrap_or(treatment_id);
    let url = format!("{}/patients/{}/treatments/{}", API_BASE, clean_p_id, clean_t_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao atualizar procedimento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientTreatment>()
            .await
            .map_err(|_| "Erro ao processar dados do procedimento atualizado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao atualizar procedimento.".into()
        } else {
            err
        })
    }
}

pub async fn delete_patient_treatment(
    token: &str,
    patient_id: &str,
    treatment_id: &str,
    clinic_id: &str,
) -> Result<(), String> {
    let clean_p_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let clean_t_id = treatment_id.strip_prefix("patient_treatment:").unwrap_or(treatment_id);
    let url = format!("{}/patients/{}/treatments/{}?clinic_id={}", API_BASE, clean_p_id, clean_t_id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao remover procedimento.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao remover procedimento.".into()
        } else {
            err
        })
    }
}

pub async fn update_patient_exam(
    token: &str,
    patient_id: &str,
    exam_id: &str,
    req: UpdatePatientExamRequest,
) -> Result<PatientExam, String> {
    let clean_p_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let clean_e_id = exam_id.strip_prefix("patient_exam:").unwrap_or(exam_id);
    let url = format!("{}/patients/{}/exams/{}", API_BASE, clean_p_id, clean_e_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao atualizar exame.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientExam>()
            .await
            .map_err(|_| "Erro ao processar dados do exame atualizado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao atualizar exame.".into()
        } else {
            err
        })
    }
}

pub async fn delete_patient_exam(
    token: &str,
    patient_id: &str,
    exam_id: &str,
    clinic_id: &str,
) -> Result<(), String> {
    let clean_p_id = patient_id.strip_prefix("patient:").unwrap_or(patient_id);
    let clean_e_id = exam_id.strip_prefix("patient_exam:").unwrap_or(exam_id);
    let url = format!("{}/patients/{}/exams/{}?clinic_id={}", API_BASE, clean_p_id, clean_e_id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao remover exame.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao remover exame.".into()
        } else {
            err
        })
    }
}


/// Estrutura de resposta da consulta de CEP via ViaCEP / BrasilAPI.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
pub struct ViaCepResponse {
    pub cep: Option<String>,
    pub logradouro: Option<String>,
    pub complemento: Option<String>,
    pub bairro: Option<String>,
    pub localidade: Option<String>,
    pub uf: Option<String>,
    pub erro: Option<serde_json::Value>,
}

/// Consulta os dados de endereço a partir de um CEP brasileiro (8 dígitos).
pub async fn lookup_cep(cep_raw: &str) -> Result<ViaCepResponse, String> {
    let clean_cep: String = cep_raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if clean_cep.len() != 8 {
        return Err("CEP deve conter 8 dígitos".into());
    }

    let url = format!("https://viacep.com.br/ws/{}/json/", clean_cep);
    let res = get_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Falha de conexão ao buscar CEP: {}", e))?;

    if res.status().is_success() {
        let parsed = res
            .json::<ViaCepResponse>()
            .await
            .map_err(|_| "Erro ao processar dados do endereço.".to_string())?;

        if let Some(ref err) = parsed.erro {
            if err.as_bool() == Some(true) || err.as_str() == Some("true") {
                return Err("CEP não encontrado.".into());
            }
        }
        Ok(parsed)
    } else {
        Err("CEP não encontrado.".into())
    }
}



