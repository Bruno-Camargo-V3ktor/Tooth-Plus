//! # Módulo de Integração e Serviço de Usuários e Equipe (UsersApi)

use super::mock_db::DB;
use shared::users::{CreateUserRequest, UpdateUserRequest, UserResponse};

pub struct UsersApi;

impl UsersApi {
    /// Lista membros da equipe clínica.
    pub async fn list_users(_clinic_id: &str) -> Result<Vec<UserResponse>, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;
        Ok(db.users.clone())
    }

    /// Cria um novo usuário no sistema.
    pub async fn create_user(req: CreateUserRequest) -> Result<UserResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let user = UserResponse {
            id: format!("user:{}", req.username.to_lowercase().replace(' ', "_")),
            username: req.username,
            full_name: req.full_name,
            email: req.email,
            phone: req.phone,
            document_cpf: req.document_cpf,
            professional_registry: req.professional_registry,
            is_active: true,
            role: req.role,
            permissions: req.permissions,
            clinic_ids: req.clinic_ids,
        };

        db.users.push(user.clone());
        Ok(user)
    }

    /// Atualiza dados de um usuário existente.
    pub async fn update_user(user_id: &str, req: UpdateUserRequest) -> Result<UserResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let user = db
            .users
            .iter_mut()
            .find(|u| u.id == user_id)
            .ok_or_else(|| format!("Usuário {} não encontrado.", user_id))?;

        if let Some(fn_name) = req.full_name { user.full_name = fn_name; }
        if let Some(em) = req.email { user.email = Some(em); }
        if let Some(ph) = req.phone { user.phone = Some(ph); }
        if let Some(cpf) = req.document_cpf { user.document_cpf = cpf; }
        if let Some(reg) = req.professional_registry { user.professional_registry = Some(reg); }
        if let Some(role) = req.role { user.role = role; }
        if let Some(perms) = req.permissions { user.permissions = perms; }
        if let Some(clinics) = req.clinic_ids { user.clinic_ids = clinics; }

        Ok(user.clone())
    }

    /// Remove um usuário.
    pub async fn delete_user(user_id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let initial = db.users.len();
        db.users.retain(|u| u.id != user_id);

        if db.users.len() == initial {
            return Err(format!("Usuário {} não encontrado.", user_id));
        }

        Ok(())
    }
}
