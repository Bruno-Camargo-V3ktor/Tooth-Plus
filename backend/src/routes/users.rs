use crate::auth_guard::{AuthenticatedUser, check_permission};
use crate::crypto::{decrypt_deterministic, encrypt_deterministic, hash_password};
use crate::db::Db;
use actix_web::{HttpResponse, Responder, delete, get, patch, post, put, web};
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
    document_cpf: String,
    professional_registry: Option<String>,
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

    if !check_permission(&db, &auth.id, &data.clinic_ids[0], "users:write")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    let hashed_password = match hash_password(&data.password_plain) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to secure password"),
    };

    let encrypted_cpf = match encrypt_deterministic(&data.document_cpf) {
        Ok(enc) => enc,
        Err(_) => return HttpResponse::InternalServerError().json("Failed to encrypt document"),
    };

    let mut response = match db
        .query(
            "
            BEGIN TRANSACTION;

            LET $new_user = (CREATE user SET
                username = $username,
                password_hash = $password_hash,
                full_name = $full_name,
                document_cpf = $document_cpf,
                professional_registry = $professional_registry,
                is_active = true
            );

            LET $clinics = (SELECT id FROM type::thing($clinic_ids));

            RELATE ($new_user[0].id)->works_at->$clinics SET
                role = $role,
                permissions = $permissions;

            COMMIT TRANSACTION;
        ",
        )
        .bind(("username", data.username))
        .bind(("password_hash", hashed_password))
        .bind(("full_name", data.full_name))
        .bind(("document_cpf", encrypted_cpf))
        .bind(("professional_registry", data.professional_registry))
        .bind(("clinic_ids", data.clinic_ids))
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
                in.document_cpf AS document_cpf,
                in.professional_registry AS professional_registry,
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
            document_cpf: decrypt_deterministic(&u.document_cpf).unwrap_or_default(),
            professional_registry: u.professional_registry,
            is_active: u.is_active,
            role: u.role,
            permissions: u.permissions,
            clinic_ids: vec![query.clinic_id.clone()],
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
    let target_id = path.into_inner();
    let data = req.into_inner();
    let clinic_id = query.clinic_id.clone();

    if !check_permission(&db, &auth.id, &clinic_id, "users:write")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    if data.full_name.is_some()
        || data.document_cpf.is_some()
        || data.professional_registry.is_some()
    {
        let mut q = String::from("UPDATE type::thing($target_id) MERGE {");

        if let Some(ref name) = data.full_name {
            q.push_str(&format!("full_name: '{}', ", name));
        }

        if let Some(ref cpf) = data.document_cpf {
            if let Ok(encrypted_cpf) = encrypt_deterministic(cpf) {
                q.push_str(&format!("document_cpf: '{}', ", encrypted_cpf));
            }
        }

        if let Some(ref reg) = data.professional_registry {
            q.push_str(&format!("professional_registry: '{}', ", reg));
        }

        q.pop();
        q.pop();
        q.push_str("}");

        let _ = db.query(q).bind(("target_id", target_id.clone())).await;
    }

    if data.role.is_some() || data.permissions.is_some() {
        if let Some(ref role) = data.role {
            let _ = db.query("UPDATE works_at SET role = $role WHERE in = type::thing($target_id) AND out = type::thing($clinic_id)")
                .bind(("target_id", target_id.clone()))
                .bind(("clinic_id", clinic_id.clone()))
                .bind(("role", role.clone()))
                .await;
        }

        if let Some(perms) = data.permissions {
            let _ = db.query("UPDATE works_at SET permissions = $perms WHERE in = type::thing($target_id) AND out = type::thing($clinic_id)")
                .bind(("target_id", target_id.clone()))
                .bind(("clinic_id", clinic_id.clone()))
                .bind(("perms", perms))
                .await;
        }
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

#[delete("/users/{target_id}")]
pub async fn delete_user(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> impl Responder {
    if !check_permission(&db, &auth.id, &query.clinic_id, "users:write")
        .await
        .unwrap_or(false)
    {
        return HttpResponse::Forbidden().json("Insufficient permissions");
    }

    let target_id = path.into_inner();

    match db
        .query(
            "DELETE works_at WHERE in = type::thing($target_id) AND out = type::thing($clinic_id)",
        )
        .bind(("target_id", target_id))
        .bind(("clinic_id", query.clinic_id.clone()))
        .await
    {
        Ok(_) => HttpResponse::Ok().json("User access removed successfully"),
        Err(_) => HttpResponse::InternalServerError().json("Failed to remove user access"),
    }
}
