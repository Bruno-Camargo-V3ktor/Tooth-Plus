use actix_web::{HttpResponse, Responder, post, web};
use serde::Deserialize;
use shared::auth::{LoginRequest, LoginResponse};
use shared::models::ClinicAccess;
use surrealdb::types::{SurrealValue, ToSql};

use crate::db::Db;
use crate::security::crypto::{generate_jwt, verify_password};

#[derive(Deserialize, Debug, SurrealValue)]
struct UserRecord {
    id: surrealdb::types::RecordId,
    password_hash: String,
    full_name: String,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct ClinicAccessRecord {
    clinic_id: surrealdb::types::RecordId,
    trading_name: String,
    theme_color: String,
    logo_url: Option<String>,
    role: String,
}

#[post("/login")]
pub async fn login(req: web::Json<LoginRequest>, db: web::Data<Db>) -> impl Responder {
    let credentials = req.into_inner();

    let mut response = match db
        .query("SELECT * FROM user WHERE username = $user")
        .bind(("user", credentials.username))
        .await
    {
        Ok(res) => res,
        Err(_) => return HttpResponse::InternalServerError().json("Database connection error"),
    };

    let user: Option<UserRecord> = response.take(0).unwrap_or(None);

    if let Some(u) = user {
        if verify_password(&u.password_hash, &credentials.password_plain) {
            let token = match generate_jwt(&u.id.key.to_sql()) {
                Ok(t) => t,
                Err(_) => return HttpResponse::InternalServerError().json("Failed to sign token"),
            };

            let mut access_response = match db
                .query("SELECT out.id AS clinic_id, out.trading_name AS trading_name, out.theme_color AS theme_color, role FROM works_at WHERE in = $user_id")
                .bind(("user_id", u.id.clone()))
                .await
            {
                Ok(res) => res,
                Err(_) => return HttpResponse::InternalServerError().json("Failed to fetch clinic access"),
            };

            let access_records: Vec<ClinicAccessRecord> =
                access_response.take(0).unwrap_or_default();

            let clinics = access_records
                .into_iter()
                .map(|record| ClinicAccess {
                    clinic_id: record.clinic_id.key.to_sql(),
                    trading_name: record.trading_name,
                    theme_color: record.theme_color,
                    logo_url: record.logo_url,
                    role: record.role,
                })
                .collect();

            let response_data = LoginResponse {
                token,
                user_id: u.id.key.to_sql(),
                full_name: u.full_name,
                clinics,
            };

            return HttpResponse::Ok().json(response_data);
        }
    }

    HttpResponse::Unauthorized().json("Invalid credentials")
}
