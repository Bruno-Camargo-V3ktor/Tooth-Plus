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

        let alt_perm = if required_permission.starts_with("appointments:") {
            Some(required_permission.replace("appointments:", "agenda:"))
        } else if required_permission.starts_with("agenda:") {
            Some(required_permission.replace("agenda:", "appointments:"))
        } else if required_permission.starts_with("treatments:") {
            Some(required_permission.replace("treatments:", "patients:"))
        } else if required_permission.starts_with("anamnese:") {
            Some(required_permission.replace("anamnese:", "patients:"))
        } else if required_permission.starts_with("exams:") {
            Some(required_permission.replace("exams:", "patients:"))
        } else {
            None
        };

        if let Some(perms) = r.get("permissions").and_then(|p| p.as_array()) {
            for p in perms {
                if let Some(s) = p.as_str() {
                    if s == required_permission
                        || s == "admin:all"
                        || alt_perm.as_deref() == Some(s)
                    {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}
