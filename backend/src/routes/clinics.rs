use actix_web::{HttpResponse, delete, get, post, put, web};
use serde::Deserialize;
use shared::clinics::{ClinicAddress, ClinicResponse, UpdateClinicRequest};
use shared::files::FileUploadRequest;
use std::sync::Arc;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use crate::storage::StorageProvider;

#[derive(Deserialize, Debug, SurrealValue)]
struct DbClinic {
    id: RecordId,
    corporate_name: String,
    trading_name: String,
    document_cnpj: String,
    theme_color: String,
    logo_url: Option<String>,
    whatsapp_instance: Option<String>,
    street: String,
    number: String,
    complement: Option<String>,
    neighborhood: String,
    city: String,
    state: String,
    zip_code: String,
    auto_reminders: Option<bool>,
    require_esign: Option<bool>,
}

fn record_key(id: &RecordId) -> String {
    id.key.to_sql()
}

#[get("/clinics/{clinic_id}")]
pub async fn get_clinic(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();

    if !check_permission(&db, &auth.id, &clinic_id, "clinics:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Você não tem permissão para ler dados desta unidade.".into(),
        ));
    }

    let mut response = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", clinic_id.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao buscar clínica no banco.".into()))?;

    let clinic: Option<DbClinic> = response.take(0).unwrap_or(None);

    if let Some(c) = clinic {
        return Ok(HttpResponse::Ok().json(ClinicResponse {
            id: record_key(&c.id),
            corporate_name: c.corporate_name,
            trading_name: c.trading_name,
            document_cnpj: c.document_cnpj,
            theme_color: c.theme_color,
            logo_url: c.logo_url,
            whatsapp_instance: c.whatsapp_instance,
            auto_reminders: c.auto_reminders.unwrap_or(true),
            require_esign: c.require_esign.unwrap_or(true),
            address: ClinicAddress {
                street: c.street,
                number: c.number,
                complement: c.complement,
                neighborhood: c.neighborhood,
                city: c.city,
                state: c.state,
                zip_code: c.zip_code,
            },
        }));
    }

    Err(ApiError::BadRequest("Clínica não encontrada.".into()))
}

#[put("/clinics/{clinic_id}")]
pub async fn update_clinic(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<UpdateClinicRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let data = req.into_inner();

    if !check_permission(&db, &auth.id, &clinic_id, "clinics:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para editar os dados da unidade.".into(),
        ));
    }

    let mut patch = serde_json::Map::new();
    if let Some(v) = data.trading_name {
        patch.insert("trading_name".into(), serde_json::Value::String(v));
    }
    if let Some(v) = data.corporate_name {
        patch.insert("corporate_name".into(), serde_json::Value::String(v));
    }
    if let Some(v) = data.document_cnpj {
        patch.insert("document_cnpj".into(), serde_json::Value::String(v));
    }
    if let Some(v) = data.theme_color {
        patch.insert("theme_color".into(), serde_json::Value::String(v));
    }
    if let Some(v) = data.auto_reminders {
        patch.insert("auto_reminders".into(), serde_json::Value::Bool(v));
    }
    if let Some(v) = data.require_esign {
        patch.insert("require_esign".into(), serde_json::Value::Bool(v));
    }

    if patch.is_empty() {
        return Ok(HttpResponse::Ok().json("Nenhum campo para atualizar."));
    }

    db.query("UPDATE type::record($id) MERGE $patch")
        .bind(("id", clinic_id))
        .bind(("patch", serde_json::Value::Object(patch)))
        .await
        .map_err(|_| ApiError::Database("Erro ao atualizar clínica.".into()))?;

    Ok(HttpResponse::Ok().json("Atualizado com sucesso."))
}

#[delete("/clinics/{clinic_id}")]
pub async fn delete_clinic(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();

    if !check_permission(&db, &auth.id, &clinic_id, "clinics:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Apenas administradores podem excluir uma clínica.".into(),
        ));
    }

    db.query("DELETE type::record($id)")
        .bind(("id", clinic_id))
        .await
        .map_err(|_| ApiError::Database("Erro ao excluir clínica.".into()))?;

    Ok(HttpResponse::Ok().json("Clínica excluída com sucesso."))
}

#[post("/clinics/{clinic_id}/logo")]
pub async fn upload_logo(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<FileUploadRequest>,
    db: web::Data<Db>,
    storage: web::Data<Arc<dyn StorageProvider>>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let data = req.into_inner();

    if !check_permission(&db, &auth.id, &clinic_id, "clinics:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para editar a logo da unidade.".into(),
        ));
    }

    let ext = data.filename.rsplit('.').next().unwrap_or("png");
    let file_url = storage
        .upload_file("clinics/logos", ext, &data.base64_content)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro no upload: {}", e)))?;

    db.query("UPDATE type::record($id) SET logo_url = $logo_url")
        .bind(("id", clinic_id))
        .bind(("logo_url", file_url.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao salvar URL da logo.".into()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "logo_url": file_url })))
}

#[get("/clinics/{clinic_id}/whatsapp/qr")]
pub async fn get_whatsapp_qr(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();

    if !check_permission(&db, &auth.id, &clinic_id, "whatsapp:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para acessar integração de WhatsApp.".into(),
        ));
    }

    let qr_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    Ok(HttpResponse::Ok().json(serde_json::json!({ "qrcode": qr_base64 })))
}
