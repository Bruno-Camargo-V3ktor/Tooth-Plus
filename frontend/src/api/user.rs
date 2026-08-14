use super::API_BASE;
use reqwest::Client;
use shared::users::{CreateUserRequest, ToggleStatusRequest, UpdateUserRequest, UserResponse};

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_users(token: &str, clinic_id: &str) -> Result<Vec<UserResponse>, String> {
    let url = format!("{}/users?clinic_id={}", API_BASE, clinic_id);

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o servidor ao buscar usuários.".to_string())?;

    if res.status().is_success() {
        res.json::<Vec<UserResponse>>()
            .await
            .map_err(|_| "Erro ao processar listagem de usuários.".into())
    } else {
        Err("Não foi possível carregar os usuários desta unidade.".to_string())
    }
}

pub async fn create_user(token: &str, req: CreateUserRequest) -> Result<(), String> {
    let url = format!("{}/users", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o servidor ao criar usuário.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&err_text) {
            if let Some(err) = val.get("error").and_then(|e| e.as_str()) {
                return Err(err.to_string());
            }
        }
        Err("Não foi possível criar o usuário. Verifique as permissões e dados informados.".to_string())
    }
}

pub async fn update_user(
    token: &str,
    target_id: &str,
    clinic_id: &str,
    req: UpdateUserRequest,
) -> Result<(), String> {
    let url = format!("{}/users/{}?clinic_id={}", API_BASE, target_id, clinic_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao atualizar dados do usuário.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível atualizar o usuário. Verifique suas permissões.".to_string())
    }
}

pub async fn toggle_user_status(
    token: &str,
    target_id: &str,
    clinic_id: &str,
    req: ToggleStatusRequest,
) -> Result<(), String> {
    let url = format!("{}/users/{}/status?clinic_id={}", API_BASE, target_id, clinic_id);

    let res = get_client()
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao alterar status do usuário.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível alterar o status do usuário.".to_string())
    }
}

pub async fn delete_user(token: &str, target_id: &str, clinic_id: &str) -> Result<(), String> {
    let url = format!("{}/users/{}?clinic_id={}", API_BASE, target_id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao remover usuário.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível remover o acesso do usuário nesta unidade.".to_string())
    }
}
