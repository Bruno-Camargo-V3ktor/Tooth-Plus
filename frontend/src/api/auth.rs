//! # Módulo de Integração e Serviço de Autenticação (AuthApi)

use super::mock_db::DB;
use shared::auth::ClinicAccess;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SessionState {
    pub token: String,
    pub user_id: String,
    pub full_name: String,
    pub clinics: Vec<ClinicAccess>,
}

pub struct AuthApi;

impl AuthApi {
    /// Realiza a autenticação do usuário.
    pub async fn login(username: String, password_plain: String) -> Result<SessionState, String> {
        gloo_timers::future::TimeoutFuture::new(250).await;

        let u = username.trim().to_lowercase();
        let p = password_plain.trim();

        if u.is_empty() || p.is_empty() {
            return Err("Informe o usuário e a senha.".to_string());
        }

        if u == "erro" {
            return Err("Credenciais inválidas. Verifique seu usuário e senha.".to_string());
        }

        let db = DB.lock().map_err(|e| e.to_string())?;

        let (user_id, full_name, role, permissions) = if let Some(found) = db.users.iter().find(|user| user.username.to_lowercase() == u) {
            (found.id.clone(), found.full_name.clone(), found.role.clone(), found.permissions.clone())
        } else {
            ("user:admin_principal".to_string(), "Dr. Roberto Alencar".to_string(), "admin".to_string(), vec![
                "patients:read".into(), "patients:write".into(),
                "agenda:read".into(), "agenda:write".into(),
                "finance:read".into(), "finance:write".into(),
                "stock:read".into(), "stock:write".into(),
                "treatments:read".into(), "treatments:write".into(),
                "settings:read".into(), "settings:write".into(),
            ])
        };

        let clinics = db.clinics.iter().map(|c| {
            ClinicAccess {
                clinic_id: c.id.clone(),
                trading_name: c.trading_name.clone(),
                theme_color: c.theme_color.clone(),
                logo_url: c.logo_url.clone(),
                role: role.clone(),
                permissions: permissions.clone(),
            }
        }).collect();

        Ok(SessionState {
            token: "jwt_mock_token_toothplus_v2".to_string(),
            user_id,
            full_name,
            clinics,
        })
    }
}
