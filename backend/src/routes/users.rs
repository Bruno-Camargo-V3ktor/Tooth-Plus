use crate::auth_guard::check_permission;
use crate::db::Db;
use crate::{auth_guard::AuthenticatedUser, crypto::hash_password};
use actix_web::{HttpResponse, Responder, get, patch, post, put, web};
use serde::Deserialize;
use shared::users::{CreateUserRequest, ToggleStatusRequest, UpdateUserRequest, UserResponse};
use surrealdb::types::{SurrealValue, ToSql};

#[derive(Deserialize)]
pub struct ClinicQuery {
    clinic_id: String,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbUserRecord {
    id: surrealdb::types::RecordId,
    username: String,
    full_name: String,
    is_active: bool,
    role: String,
    permissions: Vec<String>,
}

#[post("/users")]
pub async fn create_user(
    auth: AuthenticatedUser,
    req: web::Json<CreateUserRequest>,
    db: web::Data<Db>,
) -> impl Responder {
    let data = req.into_inner();

    if !check_permission(&db, &auth.id, &data.clinic_id, "users:write")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    let hashed_password = match hash_password(&data.password_plain) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to secure password"),
    };

    let mut response = match db
        .query(
            "
            BEGIN TRANSACTION;

            LET $new_user = (CREATE user SET
                username = $username,
                password_hash = $password_hash,
                full_name = $full_name,
                is_active = true
            );

            RELATE ($new_user[0].id)->works_at->(type::thing($clinic_id)) SET
                role = $role,
                permissions = $permissions;

            COMMIT TRANSACTION;
        ",
        )
        .bind(("username", data.username))
        .bind(("password_hash", hashed_password))
        .bind(("full_name", data.full_name))
        .bind(("clinic_id", data.clinic_id))
        .bind(("role", data.role))
        .bind(("permissions", data.permissions))
        .await
    {
        Ok(res) => res,
        Err(_) => return HttpResponse::InternalServerError().json("Database transaction failed"),
    };

    if response.take_errors().is_empty() {
        HttpResponse::Created().json("User created successfully")
    } else {
        HttpResponse::BadRequest().json("Failed to create user")
    }
}

#[get("/users")]
pub async fn list_users(
    auth: AuthenticatedUser,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> impl Responder {
    if !check_permission(&db, &auth.id, &query.clinic_id, "users:read")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    let mut response = match db
        .query(
            "
            SELECT
                in.id AS id,
                in.username AS username,
                in.full_name AS full_name,
                in.is_active AS is_active,
                role,
                permissions
            FROM works_at
            WHERE out = type::thing($clinic_id)
        ",
        )
        .bind(("clinic_id", query.clinic_id.clone()))
        .await
    {
        Ok(res) => res,
        Err(_) => return HttpResponse::InternalServerError().json("Database query failed"),
    };

    let users: Vec<DbUserRecord> = response.take(0).unwrap_or_default();

    let result: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id.key.to_sql(),
            username: u.username,
            full_name: u.full_name,
            is_active: u.is_active,
            role: u.role,
            permissions: u.permissions,
        })
        .collect();

    HttpResponse::Ok().json(result)
}

#[put("/users/{target_id}")]
pub async fn update_user(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<UpdateUserRequest>,
    db: web::Data<Db>,
) -> impl Responder {
    if !check_permission(&db, &auth.id, &query.clinic_id, "users:write")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    let target_id = path.into_inner();
    let data = req.into_inner();

    if let Some(name) = data.full_name {
        let _ = db
            .query("UPDATE type::thing($target_id) SET full_name = $name")
            .bind(("target_id", target_id.clone()))
            .bind(("name", name))
            .await;
    }

    if data.role.is_some() || data.permissions.is_some() {
        let mut q = String::from("UPDATE works_at SET ");
        let mut bindings = Vec::new();

        if let Some(r) = data.role {
            q.push_str("role = $role, ");
            bindings.push(("role", r));
        }

        if let Some(p) = data.permissions {
            q.push_str("permissions = $permissions, ");
            bindings.push(("permissions", serde_json::to_string(&p).unwrap_or_default()));
        }

        q.pop();
        q.pop();

        q.push_str(" WHERE in = type::thing($target_id) AND out = type::thing($clinic_id)");

        let mut stmt = db
            .query(q)
            .bind(("target_id", target_id.clone()))
            .bind(("clinic_id", query.clinic_id.clone()));

        for (k, v) in bindings {
            stmt = stmt.bind((k, v));
        }

        let _ = stmt.await;
    }

    HttpResponse::Ok().json("User updated successfully")
}

#[patch("/users/{target_id}/status")]
pub async fn toggle_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<ToggleStatusRequest>,
    db: web::Data<Db>,
) -> impl Responder {
    if !check_permission(&db, &auth.id, &query.clinic_id, "users:manage_status")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    let target_id = path.into_inner();
    let is_active = req.into_inner().is_active;

    match db
        .query("UPDATE type::thing($target_id) SET is_active = $is_active")
        .bind(("target_id", target_id))
        .bind(("is_active", is_active))
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Status updated"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to update status"),
    }
}
