use crate::models::ClinicAccess;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LoginRequest {
    pub username: String,
    pub password_plain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub full_name: String,
    pub clinics: Vec<ClinicAccess>,
}
