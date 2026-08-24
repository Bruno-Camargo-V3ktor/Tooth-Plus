//! # Modelos de Domínio - Usuários e Membros da Equipe
//!
//! Este módulo define os modelos para criação, listagem e controle de acesso baseado em
//! permissões (PBAC) para membros da equipe clínica (dentistas, recepcionistas, administradores).

use serde::{Deserialize, Serialize};

/// Representação de um usuário do sistema em uma ou mais clínicas.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub document_cpf: String,
    pub professional_registry: Option<String>,
    pub is_active: bool,
    pub role: String,
    pub permissions: Vec<String>,
    pub clinic_ids: Vec<String>,
}

/// Requisição para criação de novo membro da clínica.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateUserRequest {
    pub username: String,
    pub password_plain: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub document_cpf: String,
    pub professional_registry: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub clinic_ids: Vec<String>,
}

/// Requisição para atualização de dados ou permissões do usuário.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub new_password: Option<String>,
    pub document_cpf: Option<String>,
    pub professional_registry: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub clinic_ids: Option<Vec<String>>,
}

/// Requisição para ativar ou desativar o acesso de um usuário.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToggleStatusRequest {
    pub is_active: bool,
}
