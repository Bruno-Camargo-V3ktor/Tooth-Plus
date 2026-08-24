//! # Endpoints de Autenticação (Frontend)

use super::mock::{mock_login_call, SessionState};
use shared::auth::{LoginRequest, LoginResponse};

pub async fn login_user(username: String, password_plain: String) -> Result<SessionState, String> {
    let req = LoginRequest {
        username,
        password_plain,
    };

    // Utiliza o mock interativo durante a fase de construção do frontend
    let res: LoginResponse = mock_login_call(req).await?;

    Ok(SessionState {
        token: res.token,
        user_id: res.user_id,
        full_name: res.full_name,
        clinics: res.clinics,
    })
}
