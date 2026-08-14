use super::API_BASE;
use reqwest::Client;
use shared::auth::{LoginRequest, LoginResponse};

fn get_client() -> Client {
    Client::new()
}

pub async fn authenticate(req: LoginRequest) -> Result<LoginResponse, String> {
    let url = format!("{}/login", API_BASE);

    let res = get_client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o servidor. Verifique sua conexão.".to_string())?;

    if res.status().is_success() {
        res.json::<LoginResponse>()
            .await
            .map_err(|_| "Erro ao processar resposta do servidor.".into())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&err_text) {
            if let Some(err) = val.get("error").and_then(|e| e.as_str()) {
                return Err(err.to_string());
            }
        }
        Err("Usuário ou senha inválidos.".to_string())
    }
}
