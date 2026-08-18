//! # Upload de Documentos e Notas Fiscais de Estoque (Backend)
//!
//! Controla o upload de arquivos de notas fiscais, certificados e laudos de calibração
//! para o Storage Provider da clínica.

use super::clinic_record_id;
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use crate::storage::StorageProvider;
use actix_web::{post, web, HttpResponse};
use shared::files::FileUploadRequest;
use std::sync::Arc;

/// Realiza o upload de documentos de estoque (notas fiscais, manuais, certificados de garantia).
#[post("/stock/{clinic_id}/upload")]
pub async fn upload_stock_document(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<FileUploadRequest>,
    db: web::Data<Db>,
    storage: web::Data<Arc<dyn StorageProvider>>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let clinic_str = clinic_record_id(&clinic_id);

    let has_write = check_permission(&db, &auth.id, &clinic_str, "stock:write")
        .await
        .unwrap_or(false);
    let has_movement = check_permission(&db, &auth.id, &clinic_str, "stock:movement")
        .await
        .unwrap_or(false);

    if !has_write && !has_movement {
        return Err(ApiError::Forbidden(
            "Sem permissão para anexar documentos no estoque.".into(),
        ));
    }

    let data = req.into_inner();
    let ext = data.filename.rsplit('.').next().unwrap_or("pdf");
    let file_url = storage
        .upload_file(
            &format!("clinics/{}/stock", clinic_id.replace("clinic:", "")),
            ext,
            &data.base64_content,
        )
        .await
        .map_err(|e| ApiError::Internal(format!("Erro no upload: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "url": file_url,
        "filename": data.filename
    })))
}
