//! # Módulo de Autenticação e Emissão de Tokens JWT (Backend)
//!
//! Controla o login de usuários no sistema, verificação criptográfica de senhas
//! e retorno de clínicas acessíveis com papéis e permissões do usuário.

use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use shared::auth::{LoginRequest, LoginResponse};
use shared::models::ClinicAccess;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

use crate::db::Db;
use crate::error::ApiError;
use crate::security::crypto::{generate_jwt, verify_password};

#[derive(Deserialize, Debug, SurrealValue)]
struct UserRecord {
    id: RecordId,
    password_hash: String,
    full_name: String,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct ClinicAccessRecord {
    clinic_id: RecordId,
    trading_name: String,
    theme_color: String,
    logo_url: Option<String>,
    role: String,
    permissions: Vec<String>,
}

fn record_id_string(id: &RecordId) -> String {
    id.to_sql()
}

#[post("/login")]
pub async fn login(
    req: web::Json<LoginRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let credentials = req.into_inner();

    let mut response = db
        .query("SELECT id, password_hash, full_name FROM user WHERE username = $username")
        .bind(("username", credentials.username))
        .await
        .map_err(|_| ApiError::Database("Erro de conexão com o banco de dados.".into()))?;

    let user: Option<UserRecord> = response.take(0).unwrap_or(None);

    if let Some(u) = user {
        if verify_password(&u.password_hash, &credentials.password_plain) {
            let full_user_id = record_id_string(&u.id);
            let token = generate_jwt(&full_user_id)
                .map_err(|_| ApiError::Internal("Falha ao assinar o token.".into()))?;

            let mut access_response = db
                .query(
                    "SELECT
                        out.id           AS clinic_id,
                        out.trading_name AS trading_name,
                        out.theme_color  AS theme_color,
                        out.logo_url     AS logo_url,
                        role,
                        permissions
                    FROM works_at
                    WHERE in = $user_id",
                )
                .bind(("user_id", u.id.clone()))
                .await
                .map_err(|_| ApiError::Database("Falha ao buscar acessos do usuário.".into()))?;

            let access_records: Vec<ClinicAccessRecord> =
                access_response.take(0).unwrap_or_default();

            let clinics = access_records
                .into_iter()
                .map(|r| ClinicAccess {
                    clinic_id: record_id_string(&r.clinic_id),
                    trading_name: r.trading_name,
                    theme_color: r.theme_color,
                    logo_url: r.logo_url,
                    role: r.role,
                    permissions: r.permissions,
                })
                .collect();

            return Ok(HttpResponse::Ok().json(LoginResponse {
                token,
                user_id: full_user_id,
                full_name: u.full_name,
                clinics,
            }));
        }
    }

    Err(ApiError::Unauthorized("Usuário ou senha inválidos.".into()))
}
