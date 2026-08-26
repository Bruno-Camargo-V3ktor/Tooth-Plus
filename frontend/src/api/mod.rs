//! # Camada de Integração de API e Serviços
//!
//! Centraliza o gerenciamento de sessão, autenticação, armazenamento local
//! e exporta todos os serviços de entidades com interface fortemente tipada.

pub mod appointments;
pub mod auth;
pub mod clinics;
pub mod finance;
pub mod mock_db;
pub mod patients;
pub mod stock;
pub mod treatments;
pub mod users;

pub use appointments::AppointmentsApi;
pub use auth::{AuthApi, SessionState};
pub use clinics::ClinicsApi;
pub use finance::FinanceApi;
pub use patients::PatientsApi;
pub use stock::StockApi;
pub use treatments::TreatmentsApi;
pub use users::UsersApi;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActiveClinicState {
    pub clinic_id: String,
    pub trading_name: String,
    pub theme_color: String,
    pub logo_url: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
}

const SESSION_STORAGE_KEY: &str = "toothplus_v2_session";
const ACTIVE_CLINIC_STORAGE_KEY: &str = "toothplus_v2_active_clinic";

/// Salva o estado da sessão no LocalStorage do navegador.
pub fn save_session(session: &SessionState) {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let val = serde_json::json!({
                "token": session.token,
                "user_id": session.user_id,
                "full_name": session.full_name,
                "clinics": session.clinics,
            });
            let _ = storage.set_item(SESSION_STORAGE_KEY, &val.to_string());
        }
    }
}

/// Carrega a sessão salva no LocalStorage.
pub fn load_session() -> Option<SessionState> {
    let win = web_sys::window()?;
    let storage = win.local_storage().ok()??;
    let raw = storage.get_item(SESSION_STORAGE_KEY).ok()??;
    let val: serde_json::Value = serde_json::from_str(&raw).ok()?;

    Some(SessionState {
        token: val.get("token")?.as_str()?.to_string(),
        user_id: val.get("user_id")?.as_str()?.to_string(),
        full_name: val.get("full_name")?.as_str()?.to_string(),
        clinics: serde_json::from_value(val.get("clinics")?.clone()).ok()?,
    })
}

/// Limpa os dados de sessão e desconecta o usuário.
pub fn clear_session() {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.remove_item(SESSION_STORAGE_KEY);
            let _ = storage.remove_item(ACTIVE_CLINIC_STORAGE_KEY);
        }
    }
}

/// Salva a clínica ativa selecionada.
pub fn save_active_clinic(clinic: &ActiveClinicState) {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let val = serde_json::json!({
                "clinic_id": clinic.clinic_id,
                "trading_name": clinic.trading_name,
                "theme_color": clinic.theme_color,
                "logo_url": clinic.logo_url,
                "role": clinic.role,
                "permissions": clinic.permissions,
            });
            let _ = storage.set_item(ACTIVE_CLINIC_STORAGE_KEY, &val.to_string());
        }
    }
}

/// Carrega a clínica ativa salva.
pub fn load_active_clinic() -> Option<ActiveClinicState> {
    let win = web_sys::window()?;
    let storage = win.local_storage().ok()??;
    let raw = storage.get_item(ACTIVE_CLINIC_STORAGE_KEY).ok()??;
    let val: serde_json::Value = serde_json::from_str(&raw).ok()?;

    Some(ActiveClinicState {
        clinic_id: val.get("clinic_id")?.as_str()?.to_string(),
        trading_name: val.get("trading_name")?.as_str()?.to_string(),
        theme_color: val.get("theme_color")?.as_str()?.to_string(),
        logo_url: val.get("logo_url").and_then(|v| v.as_str()).map(String::from),
        role: val.get("role")?.as_str()?.to_string(),
        permissions: serde_json::from_value(val.get("permissions")?.clone()).ok()?,
    })
}
