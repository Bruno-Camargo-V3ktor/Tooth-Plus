use shared::auth::{LoginRequest, LoginResponse};
use shared::models::ClinicAccess;
use std::time::Duration;

pub async fn authenticate(req: LoginRequest) -> Result<LoginResponse, String> {
    tokio::time::sleep(Duration::from_millis(800)).await;

    if req.username == "laura" && req.password_plain == "1234" {
        Ok(LoginResponse {
            token: "mock_jwt_token_12345".to_string(),
            user_id: "user:001".to_string(),
            full_name: "Dr. Laura Alves".to_string(),
            clinics: vec![
                ClinicAccess {
                    clinic_id: "clinic:001".to_string(),
                    trading_name: "Smile Plus".to_string(),
                    theme_color: "#0052cc".to_string(),
                    logo_url: Some(
                        "https://placehold.co/400x120/transparent/00a0e4?text=Smile+Plus"
                            .to_string(),
                    ),
                    role: "admin".to_string(),
                },
                ClinicAccess {
                    clinic_id: "clinic:002".to_string(),
                    trading_name: "Luria Odontologia".to_string(),
                    theme_color: "#263d25".to_string(),
                    logo_url: Some(
                        "https://placehold.co/400x120/transparent/1e293b?text=Luria+Odonto"
                            .to_string(),
                    ),
                    role: "dentist".to_string(),
                },
            ],
        })
    } else {
        Err("Invalid credentials".to_string())
    }
}
