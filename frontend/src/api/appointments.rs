use super::API_BASE;
use reqwest::Client;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, CreateAppointmentRequest,
    UpdateAppointmentRequest, UpdateAppointmentStatusRequest,
};

fn get_client() -> Client {
    Client::new()
}

pub async fn fetch_appointments(
    token: &str,
    clinic_id: &str,
    date: Option<&str>,
    user_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<AppointmentResponse>, String> {
    let mut url = format!("{}/appointments?clinic_id={}", API_BASE, clinic_id);

    if let Some(d) = date {
        if !d.is_empty() {
            url.push_str(&format!("&date={}", d));
        }
    }

    if let Some(u) = user_id {
        if !u.is_empty() && u != "all" {
            url.push_str(&format!("&user_id={}", u));
        }
    }

    if let Some(s) = status {
        if !s.is_empty() && s != "all" {
            url.push_str(&format!("&status={}", s));
        }
    }

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o servidor ao buscar agendamentos.".to_string())?;

    if res.status().is_success() {
        res.json::<Vec<AppointmentResponse>>()
            .await
            .map_err(|_| "Erro ao processar listagem de agendamentos.".into())
    } else {
        Err("Não foi possível carregar a agenda desta unidade.".to_string())
    }
}

pub async fn fetch_agenda_resources(
    token: &str,
    clinic_id: &str,
) -> Result<AgendaResourcesResponse, String> {
    let url = format!(
        "{}/appointments/resources?clinic_id={}",
        API_BASE, clinic_id
    );

    let res = get_client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao buscar recursos da agenda.".to_string())?;

    if res.status().is_success() {
        res.json::<AgendaResourcesResponse>()
            .await
            .map_err(|_| "Erro ao processar recursos da agenda.".into())
    } else {
        Err("Não foi possível carregar profissionais e pacientes.".to_string())
    }
}

pub async fn create_appointment(token: &str, req: CreateAppointmentRequest) -> Result<(), String> {
    let url = format!("{}/appointments", API_BASE);

    let res = get_client()
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação com o servidor ao criar agendamento.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&err_text) {
            if let Some(err) = val.get("error").and_then(|e| e.as_str()) {
                return Err(err.to_string());
            }
        }
        Err("Não foi possível criar o agendamento. Verifique os dados informados.".to_string())
    }
}

pub async fn update_appointment(
    token: &str,
    id: &str,
    clinic_id: &str,
    req: UpdateAppointmentRequest,
) -> Result<(), String> {
    let url = format!("{}/appointments/{}?clinic_id={}", API_BASE, id, clinic_id);

    let res = get_client()
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao atualizar agendamento.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível atualizar o agendamento.".to_string())
    }
}

pub async fn update_appointment_status(
    token: &str,
    id: &str,
    clinic_id: &str,
    req: UpdateAppointmentStatusRequest,
) -> Result<(), String> {
    let url = format!(
        "{}/appointments/{}/status?clinic_id={}",
        API_BASE, id, clinic_id
    );

    let res = get_client()
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao alterar status do agendamento.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível alterar o status do agendamento.".to_string())
    }
}

pub async fn delete_appointment(token: &str, id: &str, clinic_id: &str) -> Result<(), String> {
    let url = format!("{}/appointments/{}?clinic_id={}", API_BASE, id, clinic_id);

    let res = get_client()
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|_| "Falha de comunicação ao excluir agendamento.".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err("Não foi possível excluir o agendamento.".to_string())
    }
}
