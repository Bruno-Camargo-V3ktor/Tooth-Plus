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

pub async fn fetch_whatsapp_qr_code(token: &str, clinic_id: &str) -> Result<String, String> {
    let url = format!("{}/clinics/{}/whatsapp/qr", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o serviço do WhatsApp.".to_string())?;

    if res.status().is_success() {
        let json_val: serde_json::Value = res
            .json()
            .await
            .map_err(|_| "Erro ao processar QR code.".to_string())?;

        if let Some(qr) = json_val.as_str() {
            Ok(qr.to_string())
        } else if let Some(qr) = json_val.get("qrcode").and_then(|q| q.as_str()) {
            Ok(qr.to_string())
        } else {
            Err("QR Code não disponível no momento.".to_string())
        }
    } else {
        Err("Não foi possível gerar a sessão do WhatsApp.".to_string())
    }
}
