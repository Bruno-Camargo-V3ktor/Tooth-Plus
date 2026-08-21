use super::API_BASE;
use reqwest::Client;
use shared::clinics::{ClinicResponse, UpdateClinicRequest};
use shared::files::FileUploadRequest;

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_clinic(token: &str, clinic_id: &str) -> Result<ClinicResponse, String> {
    let url = format!("{}/clinics/{}", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| {
            "Falha de comunicação com o servidor. Tente novamente mais tarde.".to_string()
        })?;

    if res.status().is_success() {
        res.json::<ClinicResponse>()
            .await
            .map_err(|_| "Erro ao processar as informações da clínica.".into())
    } else {
        Err("Não foi possível acessar os dados no momento.".to_string())
    }
}

pub async fn update_clinic(
    token: &str,
    clinic_id: &str,
    req: UpdateClinicRequest,
) -> Result<(), String> {
    let url = format!("{}/clinics/{}", API_BASE, clinic_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor. Verifique sua internet.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Ação bloqueada ou dados inválidos.".into())
    }
}

pub async fn delete_clinic(token: &str, clinic_id: &str) -> Result<(), String> {
    let url = format!("{}/clinics/{}", API_BASE, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Sem conexão com o servidor. Verifique sua internet.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível excluir a clínica. Verifique suas permissões.".into())
    }
}

pub async fn upload_clinic_logo(
    token: &str,
    clinic_id: &str,
    req: FileUploadRequest,
) -> Result<String, String> {
    let url = format!("{}/clinics/{}/logo", API_BASE, clinic_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao enviar logo.".to_string())?;

    if res.status().is_success() {
        let json_val: serde_json::Value = res
            .json()
            .await
            .map_err(|_| "Erro ao processar resposta do servidor.".to_string())?;

        if let Some(logo_url) = json_val.as_str() {
            Ok(logo_url.to_string())
        } else if let Some(logo_url) = json_val.get("logo_url").and_then(|u| u.as_str()) {
            Ok(logo_url.to_string())
        } else {
            Ok("".to_string())
        }
    } else {
        Err("Não foi possível fazer upload do logo. Verifique suas permissões.".to_string())
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct WhatsappStatusResponse {
    pub instance: String,
    pub state: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct WhatsappQrResponse {
    pub qrcode: String,
    pub instance: String,
    pub state: String,
}

pub async fn fetch_whatsapp_status(token: &str, clinic_id: &str) -> Result<WhatsappStatusResponse, String> {
    let url = format!("{}/clinics/{}/whatsapp/status", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o serviço do WhatsApp.".to_string())?;

    if res.status().is_success() {
        res.json::<WhatsappStatusResponse>()
            .await
            .map_err(|_| "Erro ao processar status do WhatsApp.".to_string())
    } else {
        Err("Não foi possível obter o status da sessão do WhatsApp.".to_string())
    }
}

pub async fn fetch_whatsapp_qr_code(token: &str, clinic_id: &str) -> Result<WhatsappQrResponse, String> {
    let url = format!("{}/clinics/{}/whatsapp/qr", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o serviço do WhatsApp.".to_string())?;

    if res.status().is_success() {
        res.json::<WhatsappQrResponse>()
            .await
            .map_err(|_| "Erro ao processar QR code do WhatsApp.".to_string())
    } else {
        let err_txt = res.text().await.unwrap_or_default();
        Err(if err_txt.is_empty() {
            "Não foi possível gerar a sessão do WhatsApp.".to_string()
        } else {
            err_txt
        })
    }
}

pub async fn disconnect_whatsapp(token: &str, clinic_id: &str) -> Result<(), String> {
    let url = format!("{}/clinics/{}/whatsapp/disconnect", API_BASE, clinic_id);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao desconectar WhatsApp.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Erro ao desconectar sessão do WhatsApp.".to_string())
    }
}

pub async fn send_test_whatsapp_message(
    token: &str,
    clinic_id: &str,
    phone: &str,
    message: Option<&str>,
) -> Result<String, String> {
    let url = format!("{}/clinics/{}/whatsapp/test", API_BASE, clinic_id);

    let payload = serde_json::json!({
        "phone": phone,
        "message": message
    });

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|_| "Falha ao enviar mensagem de teste.".to_string())?;

    if res.status().is_success() {
        let json_val: serde_json::Value = res.json().await.map_err(|_| "Erro ao ler resposta".to_string())?;
        let msg = json_val.get("message").and_then(|m| m.as_str()).unwrap_or("Mensagem enviada com sucesso!").to_string();
        Ok(msg)
    } else {
        let err_txt = res.text().await.unwrap_or_default();
        Err(if err_txt.is_empty() {
            "Falha ao enviar mensagem de teste.".to_string()
        } else {
            err_txt
        })
    }
}