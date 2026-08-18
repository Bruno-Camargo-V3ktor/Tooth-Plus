use super::API_BASE;
use reqwest::Client;
use shared::documents::{
    ContractTemplate, CreateContractTemplateRequest, PatientCheckRequest, PatientCheckResponse, PatientRegisterPasswordRequest, CreatePatientDocumentRequest,
    DoctorSignAuthRequest, DocumentsListResponse, PatientDocument, PatientSignAuthRequest,
    PublicSigningDocumentResponse, SignAuthResponse, SubmitSignatureRequest,
    UpdateContractTemplateRequest,
};
use shared::files::FileUploadRequest;

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_documents(
    token: &str,
    clinic_id: &str,
    patient_id: Option<&str>,
    status: Option<&str>,
) -> Result<DocumentsListResponse, String> {
    let mut url = format!("{}/documents?clinic_id={}", API_BASE, clinic_id);
    if let Some(pid) = patient_id {
        if !pid.is_empty() {
            url.push_str(&format!("&patient_id={}", pid));
        }
    }
    if let Some(st) = status {
        if !st.is_empty() {
            url.push_str(&format!("&status={}", st));
        }
    }

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao carregar documentos.".to_string())?;

    if res.status().is_success() {
        res.json::<DocumentsListResponse>()
            .await
            .map_err(|_| "Erro ao processar lista de documentos.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao carregar documentos.".into()
        } else {
            err
        })
    }
}

pub async fn create_patient_document(
    token: &str,
    req: CreatePatientDocumentRequest,
) -> Result<PatientDocument, String> {
    let url = format!("{}/documents", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao emitir documento.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientDocument>()
            .await
            .map_err(|_| "Erro ao processar documento emitido.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao emitir documento.".into()
        } else {
            err
        })
    }
}

pub async fn delete_patient_document(
    token: &str,
    doc_id: &str,
    clinic_id: &str,
) -> Result<(), String> {
    let clean_id = doc_id.strip_prefix("patient_document:").unwrap_or(doc_id);
    let url = format!(
        "{}/documents/{}?clinic_id={}",
        API_BASE, clean_id, clinic_id
    );

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao excluir documento.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao excluir documento.".into()
        } else {
            err
        })
    }
}

pub async fn fetch_templates(
    token: &str,
    clinic_id: &str,
) -> Result<Vec<ContractTemplate>, String> {
    let url = format!("{}/documents/templates?clinic_id={}", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao buscar modelos de contrato.".to_string())?;

    if res.status().is_success() {
        res.json::<Vec<ContractTemplate>>()
            .await
            .map_err(|_| "Erro ao processar modelos de contrato.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao carregar modelos.".into()
        } else {
            err
        })
    }
}

pub async fn create_template(
    token: &str,
    req: CreateContractTemplateRequest,
) -> Result<ContractTemplate, String> {
    let url = format!("{}/documents/templates", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao criar modelo de contrato.".to_string())?;

    if res.status().is_success() {
        res.json::<ContractTemplate>()
            .await
            .map_err(|_| "Erro ao processar modelo criado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao criar modelo de contrato.".into()
        } else {
            err
        })
    }
}

pub async fn update_template(
    token: &str,
    tpl_id: &str,
    req: UpdateContractTemplateRequest,
) -> Result<ContractTemplate, String> {
    let clean_id = tpl_id.strip_prefix("contract_template:").unwrap_or(tpl_id);
    let url = format!("{}/documents/templates/{}", API_BASE, clean_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao atualizar modelo de contrato.".to_string())?;

    if res.status().is_success() {
        res.json::<ContractTemplate>()
            .await
            .map_err(|_| "Erro ao processar modelo atualizado.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao atualizar modelo.".into()
        } else {
            err
        })
    }
}

pub async fn delete_template(token: &str, tpl_id: &str, clinic_id: &str) -> Result<(), String> {
    let clean_id = tpl_id.strip_prefix("contract_template:").unwrap_or(tpl_id);
    let url = format!(
        "{}/documents/templates/{}?clinic_id={}",
        API_BASE, clean_id, clinic_id
    );

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de conexão ao excluir modelo.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao excluir modelo.".into()
        } else {
            err
        })
    }
}

pub async fn upload_document_pdf(
    token: &str,
    filename: &str,
    base64_content: &str,
) -> Result<String, String> {
    let url = format!("{}/documents/upload", API_BASE);
    let payload = FileUploadRequest {
        filename: filename.to_string(),
        mime_type: "application/pdf".to_string(),
        base64_content: base64_content.to_string(),
        ..Default::default()
    };

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|_| "Falha de conexão no upload do arquivo.".to_string())?;

    if res.status().is_success() {
        #[derive(serde::Deserialize)]
        struct UploadRes {
            url: String,
        }
        let data = res
            .json::<UploadRes>()
            .await
            .map_err(|_| "Erro ao ler resposta do upload.".to_string())?;
        Ok(data.url)
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao fazer upload do documento.".into()
        } else {
            err
        })
    }
}

// -------------------------------------------------------------------------------------------------
// PUBLIC SIGNING PORTAL CLIENT FUNCTIONS
// -------------------------------------------------------------------------------------------------

pub async fn fetch_public_signing_document(
    signing_token: &str,
) -> Result<PublicSigningDocumentResponse, String> {
    let url = format!("{}/public/sign/{}", API_BASE, signing_token);

    let res = get_client()
        .get(&url)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao carregar documento de assinatura.".to_string())?;

    if res.status().is_success() {
        res.json::<PublicSigningDocumentResponse>()
            .await
            .map_err(|_| "Erro ao processar contrato de assinatura.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Documento de assinatura inválido ou expirado.".into()
        } else {
            err
        })
    }
}

pub async fn check_patient_signing(
    signing_token: &str,
    cpf: &str,
) -> Result<PatientCheckResponse, String> {
    let url = format!("{}/public/sign/{}/check-patient", API_BASE, signing_token);
    let req = PatientCheckRequest { cpf: cpf.to_string() };

    let res = get_client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão com portal de assinatura.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientCheckResponse>()
            .await
            .map_err(|_| "Erro ao processar verificação do paciente.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&err) {
            if let Some(msg) = v.get("error").and_then(|e| e.as_str()) {
                return Err(msg.to_string());
            }
        }
        Err("CPF não localizado para este documento.".into())
    }
}

pub async fn register_patient_password(
    signing_token: &str,
    cpf: &str,
    password: &str,
) -> Result<SignAuthResponse, String> {
    let url = format!("{}/public/sign/{}/register-patient-password", API_BASE, signing_token);
    let req = PatientRegisterPasswordRequest { cpf: cpf.to_string(), password: password.to_string() };

    let res = get_client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao cadastrar senha.".to_string())?;

    if res.status().is_success() {
        res.json::<SignAuthResponse>()
            .await
            .map_err(|_| "Erro ao autenticar após criar senha.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&err) {
            if let Some(msg) = v.get("error").and_then(|e| e.as_str()) {
                return Err(msg.to_string());
            }
        }
        Err("Falha ao registrar senha de assinatura.".into())
    }
}

pub async fn auth_patient_signing(
    signing_token: &str,
    req: PatientSignAuthRequest,
) -> Result<SignAuthResponse, String> {
    let url = format!("{}/public/sign/{}/auth-patient", API_BASE, signing_token);

    let res = get_client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão com portal de assinatura.".to_string())?;

    if res.status().is_success() {
        res.json::<SignAuthResponse>()
            .await
            .map_err(|_| "Erro ao autenticar paciente.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "CPF ou senha inválidos para este contrato.".into()
        } else {
            err
        })
    }
}

pub async fn auth_doctor_signing(
    signing_token: &str,
    req: DoctorSignAuthRequest,
) -> Result<SignAuthResponse, String> {
    let url = format!("{}/public/sign/{}/auth-doctor", API_BASE, signing_token);

    let res = get_client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão com portal de assinatura.".to_string())?;

    if res.status().is_success() {
        res.json::<SignAuthResponse>()
            .await
            .map_err(|_| "Erro ao autenticar profissional.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Usuário ou senha incorretos.".into()
        } else {
            err
        })
    }
}

pub async fn request_signing_otp(signing_token: &str, channel: &str) -> Result<String, String> {
    let url = format!("{}/public/sign/{}/request-otp", API_BASE, signing_token);

    let payload = shared::documents::RequestOtpRequest {
        channel: Some(channel.to_string()),
    };

    let res = get_client()
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|_| "Falha ao solicitar código de validação.".to_string())?;

    if res.status().is_success() {
        let json_body: serde_json::Value = res.json().await.unwrap_or_default();
        let msg = json_body.get("message").and_then(|v| v.as_str()).unwrap_or("Código enviado com sucesso.");
        Ok(msg.to_string())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao disparar código de validação.".into()
        } else {
            err
        })
    }
}

pub async fn submit_digital_signature(
    signing_token: &str,
    req: SubmitSignatureRequest,
) -> Result<PatientDocument, String> {
    let url = format!(
        "{}/public/sign/{}/submit-signature",
        API_BASE, signing_token
    );

    let res = get_client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de conexão ao submeter assinatura.".to_string())?;

    if res.status().is_success() {
        res.json::<PatientDocument>()
            .await
            .map_err(|_| "Erro ao processar confirmação de assinatura.".into())
    } else {
        let err = res.text().await.unwrap_or_default();
        Err(if err.is_empty() {
            "Erro ao registrar assinatura digital.".into()
        } else {
            err
        })
    }
}
