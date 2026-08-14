use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use crate::security::crypto::{decrypt_deterministic, encrypt_deterministic, hash_password};
use actix_web::{HttpResponse, delete, get, patch, post, put, web};
use serde::Deserialize;
use shared::users::{CreateUserRequest, ToggleStatusRequest, UpdateUserRequest, UserResponse};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

#[derive(Deserialize)]
pub struct ClinicQuery {
    clinic_id: String,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbUserRecord {
    id: RecordId,
    username: String,
    full_name: String,
    document_cpf: String,
    professional_registry: Option<String>,
    is_active: bool,
    role: String,
    permissions: Vec<String>,
    #[serde(default)]
    clinic_ids: Vec<RecordId>,
}

fn record_key(id: &RecordId) -> String {
    id.key.to_sql()
}

fn user_record_id(id: &str) -> String {
    if id.starts_with("user:") {
        id.to_string()
    } else {
        format!("user:{}", id)
    }
}

fn clinic_record_id(id: &str) -> String {
    if id.starts_with("clinic:") {
        id.to_string()
    } else {
        format!("clinic:{}", id)
    }
}

fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
}

#[post("/users")]
pub async fn create_user(
    auth: AuthenticatedUser,
    req: web::Json<CreateUserRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();

    if data.clinic_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "Selecione ao menos uma clínica.".into(),
        ));
    }

    let check_clinic = clinic_record_id(&data.clinic_ids[0]);
    if !check_permission(&db, &auth.id, &check_clinic, "users:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para criar novos membros.".into(),
        ));
    }

    let hashed_password =
        hash_password(&data.password_plain).map_err(|e| ApiError::Internal(e.to_string()))?;
    let encrypted_cpf = encrypt_deterministic(&data.document_cpf)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut create_resp = db
        .query(
            "CREATE user SET
                username              = $username,
                password_hash         = $password_hash,
                full_name             = $full_name,
                document_cpf          = $document_cpf,
                professional_registry = $professional_registry,
                is_active             = true
            RETURN id",
        )
        .bind(("username", data.username.clone()))
        .bind(("password_hash", hashed_password))
        .bind(("full_name", data.full_name.clone()))
        .bind(("document_cpf", encrypted_cpf))
        .bind(("professional_registry", data.professional_registry.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao criar usuário.".into()))?;

    #[derive(Deserialize, SurrealValue)]
    struct CreatedId {
        id: RecordId,
    }

    let created: Option<CreatedId> = create_resp.take(0).unwrap_or(None);
    let new_user_id = created
        .ok_or_else(|| ApiError::Database("Usuário não retornou ID após criação.".into()))?
        .id;

    for clinic_id in &data.clinic_ids {
        let clinic_rec = parse_record_id("clinic", clinic_id);
        db.query(
            "RELATE $user->works_at->$clinic SET
                role        = $role,
                permissions = $permissions",
        )
        .bind(("user", new_user_id.clone()))
        .bind(("clinic", clinic_rec))
        .bind(("role", data.role.clone()))
        .bind(("permissions", data.permissions.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao vincular usuário à clínica.".into()))?;
    }

    Ok(HttpResponse::Created().json("Usuário criado com sucesso."))
}

#[get("/users")]
pub async fn list_users(
    auth: AuthenticatedUser,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_rec = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "users:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para listar usuários desta unidade.".into(),
        ));
    }

    let mut response = db
        .query(
            "SELECT
                in.id                    AS id,
                in.username              AS username,
                in.full_name             AS full_name,
                in.document_cpf          AS document_cpf,
                in.professional_registry AS professional_registry,
                in.is_active             AS is_active,
                role,
                permissions,
                (SELECT VALUE out FROM works_at WHERE in = $parent.in) AS clinic_ids
            FROM works_at
            WHERE out = type::record($clinic_id)",
        )
        .bind(("clinic_id", clinic_rec))
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar usuários.".into()))?;

    let users: Vec<DbUserRecord> = response.take(0).unwrap_or_default();

    let result: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: record_key(&u.id),
            username: u.username,
            full_name: u.full_name,
            document_cpf: decrypt_deterministic(&u.document_cpf).unwrap_or_default(),
            professional_registry: u.professional_registry,
            is_active: u.is_active,
            role: u.role,
            permissions: u.permissions,
            clinic_ids: u.clinic_ids.iter().map(|c| c.to_sql()).collect(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[put("/users/{target_id}")]
pub async fn update_user(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<UpdateUserRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let target_id = path.into_inner();
    let data = req.into_inner();
    let clinic_id = query.clinic_id.clone();
    let target_rec = user_record_id(&target_id);
    let clinic_rec = clinic_record_id(&clinic_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "users:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para editar usuários.".into(),
        ));
    }

    let mut patch = serde_json::Map::new();

    if let Some(v) = data.full_name {
        patch.insert("full_name".into(), serde_json::Value::String(v));
    }
    if let Some(ref cpf) = data.document_cpf {
        let encrypted = encrypt_deterministic(cpf)
            .map_err(|_| ApiError::Internal("Falha ao criptografar CPF.".into()))?;
        patch.insert("document_cpf".into(), serde_json::Value::String(encrypted));
    }
    if let Some(v) = data.professional_registry {
        patch.insert(
            "professional_registry".into(),
            serde_json::Value::String(v),
        );
    }

    if !patch.is_empty() {
        db.query("UPDATE type::record($target_id) MERGE $patch")
            .bind(("target_id", target_rec.clone()))
            .bind(("patch", serde_json::Value::Object(patch)))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar dados do usuário.".into()))?;
    }

    if let Some(role) = data.role.clone() {
        db.query(
            "UPDATE works_at SET role = $role
            WHERE in = type::record($target_id) AND out = type::record($clinic_id)",
        )
        .bind(("target_id", target_rec.clone()))
        .bind(("clinic_id", clinic_rec.clone()))
        .bind(("role", role))
        .await
        .map_err(|_| ApiError::Database("Falha ao atualizar cargo.".into()))?;
    }

    if let Some(perms) = data.permissions.clone() {
        db.query(
            "UPDATE works_at SET permissions = $perms
            WHERE in = type::record($target_id) AND out = type::record($clinic_id)",
        )
        .bind(("target_id", target_rec.clone()))
        .bind(("clinic_id", clinic_rec.clone()))
        .bind(("perms", perms))
        .await
        .map_err(|_| ApiError::Database("Falha ao atualizar permissões.".into()))?;
    }

    if let Some(ref new_clinic_ids) = data.clinic_ids {
        if new_clinic_ids.is_empty() {
            return Err(ApiError::BadRequest(
                "O usuário deve pertencer a pelo menos uma clínica.".into(),
            ));
        }

        #[derive(Deserialize, Debug, SurrealValue)]
        struct ExistingWork {
            out: RecordId,
        }

        let mut current_works_resp = db
            .query("SELECT out FROM works_at WHERE in = type::record($target_id)")
            .bind(("target_id", target_rec.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao buscar clínicas do usuário.".into()))?;

        let existing_works: Vec<ExistingWork> = current_works_resp.take(0).unwrap_or_default();
        let existing_cids: Vec<String> = existing_works
            .into_iter()
            .map(|w| clinic_record_id(&w.out.to_sql()))
            .collect();

        let normalized_new_cids: Vec<String> = new_clinic_ids
            .iter()
            .map(|cid| clinic_record_id(cid))
            .collect();

        for existing_cid in &existing_cids {
            if !normalized_new_cids.contains(existing_cid) {
                let target_rec_id = parse_record_id("user", &target_id);
                let old_cid_id = parse_record_id("clinic", existing_cid);
                db.query(
                    "DELETE works_at WHERE in = $target_id AND out = $clinic",
                )
                .bind(("target_id", target_rec_id))
                .bind(("clinic", old_cid_id))
                .await
                .map_err(|_| ApiError::Database("Falha ao remover vínculo com clínica.".into()))?;
            }
        }

        let user_role = data.role.clone().unwrap_or_else(|| "dentist".to_string());
        let user_perms = data.permissions.clone().unwrap_or_default();

        for new_cid in &normalized_new_cids {
            if !existing_cids.contains(new_cid) {
                let target_rec_id = parse_record_id("user", &target_id);
                let new_cid_id = parse_record_id("clinic", new_cid);
                db.query(
                    "RELATE $target_id->works_at->$clinic SET
                        role        = $role,
                        permissions = $permissions",
                )
                .bind(("target_id", target_rec_id))
                .bind(("clinic", new_cid_id))
                .bind(("role", user_role.clone()))
                .bind(("permissions", user_perms.clone()))
                .await
                .map_err(|_| ApiError::Database("Falha ao vincular usuário à nova clínica.".into()))?;
            }
        }
    }

    Ok(HttpResponse::Ok().json("Usuário atualizado com sucesso."))
}

#[patch("/users/{target_id}/status")]
pub async fn toggle_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<ToggleStatusRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_rec = clinic_record_id(&query.clinic_id);
    let target_rec = user_record_id(&path.into_inner());

    if !check_permission(&db, &auth.id, &clinic_rec, "users:manage_status")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para alterar status de usuários.".into(),
        ));
    }

    let is_active = req.into_inner().is_active;

    db.query("UPDATE type::record($target_id) SET is_active = $is_active")
        .bind(("target_id", target_rec))
        .bind(("is_active", is_active))
        .await
        .map_err(|_| ApiError::Database("Falha ao atualizar status.".into()))?;

    Ok(HttpResponse::Ok().json("Status atualizado."))
}

#[delete("/users/{target_id}")]
pub async fn delete_user(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_rec = clinic_record_id(&query.clinic_id);
    let target_rec = user_record_id(&path.into_inner());

    if !check_permission(&db, &auth.id, &clinic_rec, "users:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para remover usuários.".into(),
        ));
    }

    db.query(
        "DELETE works_at
        WHERE in = type::record($target_id) AND out = type::record($clinic_id)",
    )
    .bind(("target_id", target_rec))
    .bind(("clinic_id", clinic_rec))
    .await
    .map_err(|_| ApiError::Database("Falha ao remover acesso do usuário.".into()))?;

    Ok(HttpResponse::Ok().json("Acesso do usuário removido com sucesso."))
}
