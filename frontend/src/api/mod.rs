use shared::auth::{LoginRequest, LoginResponse};
use shared::models::ClinicAccess;
use std::time::Duration;

pub async fn authenticate(req: LoginRequest) -> Result<LoginResponse, String> {
    tokio::time::sleep(Duration::from_millis(800)).await;

    if req.username == "admin" && req.password_plain == "123" {
        Ok(LoginResponse {
            token: "mock_jwt_token_12345".to_string(),
            user_id: "user:001".to_string(),
            full_name: "Dr. Admin User".to_string(),
            clinics: vec![
                ClinicAccess {
                    clinic_id: "clinic:001".to_string(),
                    trading_name: "Tooth Plus - Matriz".to_string(),
                    theme_color: "#0052cc".to_string(),
                    role: "admin".to_string(),
                },
                ClinicAccess {
                    clinic_id: "clinic:002".to_string(),
                    trading_name: "Tooth Plus - Boutique".to_string(),
                    theme_color: "#E3D8C6".to_string(),
                    role: "dentist".to_string(),
                },
            ],
        })
    } else {
        Err("Invalid credentials".to_string())
    }
}
