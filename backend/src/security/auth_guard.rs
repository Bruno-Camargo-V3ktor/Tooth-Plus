use super::crypto::Claims;
use crate::db::Db;
use actix_web::{Error as ActixError, FromRequest, HttpRequest, dev::Payload};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::future::{Ready, ready};

pub struct AuthenticatedUser {
    pub id: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = match req.headers().get("Authorization") {
            Some(h) => h.to_str().unwrap_or(""),
            None => return ready(Err(actix_web::error::ErrorUnauthorized("Missing token"))),
        };

        if !auth_header.starts_with("Bearer ") {
            return ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid token format",
            )));
        }

        let token = &auth_header[7..];
        let secret = std::env::var("JWT_SECRET").unwrap_or_default();

        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(token_data) => ready(Ok(AuthenticatedUser {
                id: token_data.claims.sub,
            })),
            Err(_) => ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid or expired token",
            ))),
        }
    }
}

pub async fn check_permission(
    db: &Db,
    user_id: &str,
    clinic_id: &str,
    required_permission: &str,
) -> Result<bool, actix_web::Error> {
    let uid_rec = if user_id.contains(':') {
        user_id.to_string()
    } else {
        format!("users:{}", user_id)
    };
    let cid_rec = if clinic_id.contains(':') {
        clinic_id.to_string()
    } else {
        format!("clinics:{}", clinic_id)
    };

    let mut response = db
        .query(
            "SELECT role, permissions FROM works_at
            WHERE in = type::record($user_id) AND out = type::record($clinic_id)",
        )
        .bind(("user_id", uid_rec))
        .bind(("clinic_id", cid_rec))
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    let record: Option<serde_json::Value> = response.take(0).unwrap_or(None);

    if let Some(r) = record {
        if let Some(role) = r.get("role").and_then(|ro| ro.as_str()) {
            if role == "admin" {
                return Ok(true);
            }
        }

        if let Some(perms) = r.get("permissions").and_then(|p| p.as_array()) {
            for p in perms {
                if let Some(s) = p.as_str() {
                    if s == "admin:all" || s == required_permission {
                        return Ok(true);
                    }

                    // Alias Agenda / Appointments
                    if (required_permission.starts_with("appointments:") && s.replace("agenda:", "appointments:") == required_permission)
                        || (required_permission.starts_with("agenda:") && s.replace("appointments:", "agenda:") == required_permission)
                    {
                        return Ok(true);
                    }

                    // Tratamentos e Sub-módulos (Orçamentos, Catálogo, Prontuário)
                    if required_permission == "treatments:read"
                        && (s == "treatment_plans:read" || s == "treatment_templates:read" || s == "patients:read" || s == "patients:write")
                    {
                        return Ok(true);
                    }
                    if required_permission == "treatments:write"
                        && (s == "treatment_plans:write" || s == "treatment_templates:write" || s == "patients:write")
                    {
                        return Ok(true);
                    }
                    if required_permission == "treatments:delete"
                        && (s == "treatment_plans:delete" || s == "treatment_templates:delete" || s == "treatments:write" || s == "patients:delete")
                    {
                        return Ok(true);
                    }
                    if (required_permission.starts_with("treatment_plans:") || required_permission.starts_with("treatment_templates:"))
                        && (s == "treatments:write" || s == "patients:write")
                    {
                        return Ok(true);
                    }

                    // Anamnese & Exames herdando de pacientes
                    if required_permission.starts_with("anamnese:")
                        && (s == "patients:write" || (s == "patients:read" && required_permission.ends_with(":read")))
                    {
                        return Ok(true);
                    }
                    if required_permission.starts_with("exams:")
                        && (s == "patients:write" || (s == "patients:read" && required_permission.ends_with(":read")))
                    {
                        return Ok(true);
                    }

                    // Documentos
                    if required_permission.starts_with("documents:")
                        && (s == "patients:write" || (s == "patients:read" && required_permission.ends_with(":read")))
                    {
                        return Ok(true);
                    }

                    // Financeiro
                    if required_permission == "finance:read"
                        && (s == "finance:read_all" || s == "finance:read_income" || s == "finance:read_expense" || s == "finance:read_pending")
                    {
                        return Ok(true);
                    }
                    if required_permission == "finance:read_all" && s == "finance:read" {
                        return Ok(true);
                    }
                    if (required_permission == "finance:read_income" || required_permission == "finance:read_expense" || required_permission == "finance:read_pending")
                        && (s == "finance:read_all" || s == "finance:read")
                    {
                        return Ok(true);
                    }
                    if required_permission == "finance:write"
                        && (s == "finance:write_income" || s == "finance:write_expense")
                    {
                        return Ok(true);
                    }
                    if (required_permission == "finance:write_income" || required_permission == "finance:write_expense" || required_permission == "finance:update_status")
                        && s == "finance:write"
                    {
                        return Ok(true);
                    }

                    // Estoque
                    if (required_permission == "stock:movement" || required_permission == "stock:delete") && s == "stock:write" {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}
