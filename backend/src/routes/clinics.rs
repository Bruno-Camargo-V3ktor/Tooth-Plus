use actix_web::{HttpResponse, delete, get, post, put, web};
use serde::Deserialize;
use shared::clinics::{ClinicAddress, ClinicResponse, UpdateClinicRequest};
use shared::files::FileUploadRequest;
use std::sync::Arc;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

use crate::db::Db;
use crate::error::ApiError;
use crate::evolution::EvolutionClient;
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
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_user: Option<String>,
    smtp_from: Option<String>,
    smtp_tls: Option<bool>,
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
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);

    if !check_permission(&db, &auth.id, &clinic_rec, "clinics:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Você não tem permissão para ler dados desta unidade.".into(),
        ));
    }

    let mut response = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", clinic_rec.clone()))
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
            smtp_host: c.smtp_host,
            smtp_port: c.smtp_port,
            smtp_user: c.smtp_user,
            smtp_from: c.smtp_from,
            smtp_tls: c.smtp_tls,
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
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);
    let data = req.into_inner();

    if !check_permission(&db, &auth.id, &clinic_rec, "clinics:write")
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
    if let Some(v) = data.smtp_host {
        if v.trim().is_empty() {
            patch.insert("smtp_host".into(), serde_json::Value::Null);
        } else {
            patch.insert("smtp_host".into(), serde_json::Value::String(v.trim().to_string()));
        }
    }
    if let Some(v) = data.smtp_port {
        patch.insert("smtp_port".into(), serde_json::Value::Number(v.into()));
    }
    if let Some(v) = data.smtp_user {
        if v.trim().is_empty() {
            patch.insert("smtp_user".into(), serde_json::Value::Null);
        } else {
            patch.insert("smtp_user".into(), serde_json::Value::String(v.trim().to_string()));
        }
    }
    if let Some(v) = data.smtp_pass {
        if v.trim().is_empty() {
            patch.insert("smtp_pass".into(), serde_json::Value::Null);
        } else {
            patch.insert("smtp_pass".into(), serde_json::Value::String(v.trim().to_string()));
        }
    }
    if let Some(v) = data.smtp_from {
        if v.trim().is_empty() {
            patch.insert("smtp_from".into(), serde_json::Value::Null);
        } else {
            patch.insert("smtp_from".into(), serde_json::Value::String(v.trim().to_string()));
        }
    }
    if let Some(v) = data.smtp_tls {
        patch.insert("smtp_tls".into(), serde_json::Value::Bool(v));
    }

    if patch.is_empty() {
        return Ok(HttpResponse::Ok().json("Nenhum campo para atualizar."));
    }

    db.query("UPDATE type::record($id) MERGE $patch")
        .bind(("id", clinic_rec))
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
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);

    if !check_permission(&db, &auth.id, &clinic_rec, "clinics:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Apenas administradores podem excluir uma clínica.".into(),
        ));
    }

    db.query("DELETE type::record($id)")
        .bind(("id", clinic_rec))
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
    let clinic_rec = if clinic_id.contains(':') {
        clinic_id.clone()
    } else {
        format!("clinics:{}", clinic_id)
    };
    let data = req.into_inner();

    if !check_permission(&db, &auth.id, &clinic_rec, "clinics:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para editar a logo da unidade.".into(),
        ));
    }

    let ext = data.filename.rsplit('.').next().unwrap_or("png");
    let file_url = storage
        .upload_file(&format!("clinics/{}/logos", clinic_id.replace("clinic:", "").replace("clinics:", "")), ext, &data.base64_content)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro no upload: {}", e)))?;

    db.query("UPDATE type::record($id) SET logo_url = $logo_url")
        .bind(("id", clinic_rec))
        .bind(("logo_url", file_url.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao salvar URL da logo no banco.".into()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "logo_url": file_url })))
}

#[get("/clinics/{clinic_id}/whatsapp/status")]
pub async fn get_whatsapp_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
    evolution: web::Data<EvolutionClient>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);

    if !check_permission(&db, &auth.id, &clinic_rec, "whatsapp:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para acessar integração de WhatsApp.".into(),
        ));
    }

    let mut response = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao buscar clínica no banco.".into()))?;

    let clinic: Option<DbClinic> = response.take(0).unwrap_or(None);
    let instance_name = clinic
        .as_ref()
        .and_then(|c| c.whatsapp_instance.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("clinic_{}", raw_key));

    let api_key = std::env::var("EVOLUTION_API_KEY").unwrap_or_default();
    let state = evolution
        .get_connection_state(&instance_name, &api_key)
        .await
        .unwrap_or_else(|_| "disconnected".to_string());

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "instance": instance_name,
        "state": state
    })))
}

#[get("/clinics/{clinic_id}/whatsapp/qr")]
pub async fn get_whatsapp_qr(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
    evolution: web::Data<EvolutionClient>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);

    if !check_permission(&db, &auth.id, &clinic_rec, "whatsapp:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para acessar integração de WhatsApp.".into(),
        ));
    }

    let mut response = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao buscar clínica no banco.".into()))?;

    let clinic: Option<DbClinic> = response.take(0).unwrap_or(None);
    let instance_name = clinic
        .as_ref()
        .and_then(|c| c.whatsapp_instance.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("clinic_{}", raw_key));

    let api_key = std::env::var("EVOLUTION_API_KEY").unwrap_or_default();

    // Salva o nome da instância no registro da clínica se ainda não estiver definido
    let _ = db
        .query("UPDATE type::record($id) SET whatsapp_instance = $inst WHERE whatsapp_instance = NONE OR whatsapp_instance = NULL;")
        .bind(("id", clinic_rec.clone()))
        .bind(("inst", instance_name.clone()))
        .await;

    // 1. Tenta criar a instância (se não existir)
    let _ = evolution.create_instance(&instance_name, &api_key).await;

    // 2. Conecta e obtém o QR Code
    let qr_result = evolution.connect_instance(&instance_name, &api_key).await;

    let state = evolution
        .get_connection_state(&instance_name, &api_key)
        .await
        .unwrap_or_else(|_| "connecting".to_string());

    match qr_result {
        Ok(qr) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "qrcode": qr,
                "instance": instance_name,
                "state": state
            })))
        }
        Err(e) => {
            if state == "open" {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "qrcode": "ALREADY_CONNECTED",
                    "instance": instance_name,
                    "state": "open"
                })))
            } else {
                Err(ApiError::Internal(format!("Erro ao gerar QR Code: {}", e)))
            }
        }
    }
}

#[post("/clinics/{clinic_id}/whatsapp/disconnect")]
pub async fn disconnect_whatsapp(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
    evolution: web::Data<EvolutionClient>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);

    if !check_permission(&db, &auth.id, &clinic_rec, "whatsapp:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para desconectar o WhatsApp.".into(),
        ));
    }

    let mut response = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao buscar clínica no banco.".into()))?;

    let clinic: Option<DbClinic> = response.take(0).unwrap_or(None);
    let instance_name = clinic
        .as_ref()
        .and_then(|c| c.whatsapp_instance.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("clinic_{}", raw_key));

    let api_key = std::env::var("EVOLUTION_API_KEY").unwrap_or_default();
    evolution
        .disconnect_instance(&instance_name, &api_key)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao desconectar: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Sessão do WhatsApp desconectada com sucesso.",
        "state": "close"
    })))
}

#[derive(Deserialize)]
pub struct SendTestWhatsappRequest {
    pub phone: String,
    pub message: Option<String>,
}

#[post("/clinics/{clinic_id}/whatsapp/test")]
pub async fn send_test_whatsapp(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<SendTestWhatsappRequest>,
    db: web::Data<Db>,
    evolution: web::Data<EvolutionClient>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let raw_key = clinic_id
        .replace("clinics:", "")
        .replace("clinic:", "")
        .replace('⟨', "")
        .replace('⟩', "");
    let clinic_rec = format!("clinic:{}", raw_key);

    if !check_permission(&db, &auth.id, &clinic_rec, "whatsapp:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para enviar teste de WhatsApp.".into(),
        ));
    }

    let mut response = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Erro ao buscar clínica no banco.".into()))?;

    let clinic: Option<DbClinic> = response.take(0).unwrap_or(None);
    let instance_name = clinic
        .as_ref()
        .and_then(|c| c.whatsapp_instance.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("clinic_{}", raw_key));

    let api_key = std::env::var("EVOLUTION_API_KEY").unwrap_or_default();
    let default_msg = "🦷 *Tooth Plus — Teste de Integração*\n\nSeu WhatsApp está conectado com sucesso ao sistema Tooth-Plus!".to_string();
    let message = req.message.as_deref().unwrap_or(&default_msg);

    let message_id = evolution
        .send_whatsapp_text(&instance_name, &api_key, &req.phone, message)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro ao enviar mensagem de teste via WhatsApp: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Mensagem de teste enviada com sucesso!",
        "message_id": message_id
    })))
}

