use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub document_cpf: String,
    pub professional_registry: Option<String>,
    pub is_active: bool,
    pub role: String,
    pub permissions: Vec<String>,
    pub clinic_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateUserRequest {
    pub username: String,
    pub password_plain: String,
    pub full_name: String,
    pub document_cpf: String,
    pub professional_registry: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub clinic_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub document_cpf: Option<String>,
    pub professional_registry: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub clinic_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToggleStatusRequest {
    pub is_active: bool,
}
