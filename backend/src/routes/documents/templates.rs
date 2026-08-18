//! # Gerenciamento de Modelos de Contrato e Templates (Backend)
//!
//! Contém os endpoints para listagem, criação, edição e exclusão de modelos de
//! contratos clínicos reutilizáveis com campos de assinatura parametrizáveis.

use super::{
    clinic_record_id, map_template, parse_record_id, DbContractTemplateRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use crate::storage::StorageProvider;
use actix_web::{delete, get, post, put, web, HttpResponse};
use serde::Deserialize;
use shared::documents::{
    ContractTemplate, CreateContractTemplateRequest, UpdateContractTemplateRequest,
};
use shared::files::FileUploadRequest;
use std::sync::Arc;

/// Query string padrão com `clinic_id`.
#[derive(Deserialize)]
pub struct ClinicQuery {
    pub clinic_id: String,
}

/// Lista todos os modelos de contratos cadastrados para a clínica informada.
#[get("/documents/templates")]
pub async fn list_templates(
    auth: AuthenticatedUser,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para listar modelos.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &query.clinic_id);

    let mut res = db
        .query("SELECT * FROM contract_template WHERE clinic_id = $cid ORDER BY created_at DESC;")
        .bind(("cid", clinic_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao buscar modelos: {}", e)))?;

    let raw: Vec<DbContractTemplateRow> = res.take(0).unwrap_or_default();
    let list: Vec<ContractTemplate> = raw.into_iter().map(map_template).collect();

    Ok(HttpResponse::Ok().json(list))
}

/// Cria um novo modelo de contrato clínico com campos de assinatura.
#[post("/documents/templates")]
pub async fn create_template(
    auth: AuthenticatedUser,
    req: web::Json<CreateContractTemplateRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para criar modelos.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    let fields_json = serde_json::to_value(&data.signature_fields).unwrap_or_default();

    let mut res = db
        .query(
            "CREATE contract_template CONTENT {
            clinic_id: $cid,
            title: $title,
            category: $cat,
            description: $desc,
            pdf_url: $pdf_url,
            signature_fields: $fields,
            created_at: time::now(),
            updated_at: time::now()
        };",
        )
        .bind(("cid", clinic_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("cat", data.category))
        .bind(("desc", data.description.map(|s| s.trim().to_string())))
        .bind(("pdf_url", data.pdf_url))
        .bind(("fields", fields_json))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar modelo de contrato: {}", e)))?;

    let created: Option<DbContractTemplateRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = created else {
        return Err(ApiError::Database(
            "Falha ao retornar modelo criado.".into(),
        ));
    };

    Ok(HttpResponse::Created().json(map_template(row)))
}

/// Atualiza as configurações e campos de assinatura de um modelo existente.
#[put("/documents/templates/{id}")]
pub async fn update_template(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<UpdateContractTemplateRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let tpl_rec = parse_record_id("contract_template", &path.into_inner());
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para atualizar modelos.".into(),
        ));
    }

    let fields_json = serde_json::to_value(&data.signature_fields).unwrap_or_default();

    let mut res = db
        .query(
            "UPDATE type::record($id) SET
            title = $title,
            category = $cat,
            description = $desc,
            pdf_url = $pdf_url,
            signature_fields = $fields,
            updated_at = time::now();",
        )
        .bind(("id", tpl_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("cat", data.category))
        .bind(("desc", data.description.map(|s| s.trim().to_string())))
        .bind(("pdf_url", data.pdf_url))
        .bind(("fields", fields_json))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar modelo: {}", e)))?;

    let updated: Option<DbContractTemplateRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = updated else {
        return Err(ApiError::NotFound("Modelo não encontrado.".into()));
    };

    Ok(HttpResponse::Ok().json(map_template(row)))
}

/// Exclui um modelo de contrato clínico cadastrado.
#[delete("/documents/templates/{id}")]
pub async fn delete_template(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let tpl_rec = parse_record_id("contract_template", &path.into_inner());
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para excluir modelo.".into(),
        ));
    }

    db.query("DELETE type::record($id)")
        .bind(("id", tpl_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir modelo: {}", e)))?;

    Ok(HttpResponse::Ok().body("Modelo excluído com sucesso."))
}

/// Realiza o upload do PDF base de um modelo ou termo clínico para o Storage.
#[post("/documents/upload")]
pub async fn upload_document_pdf(
    _auth: AuthenticatedUser,
    req: web::Json<FileUploadRequest>,
    storage: web::Data<Arc<dyn StorageProvider>>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let ext = data.filename.rsplit('.').next().unwrap_or("pdf");
    let clean_clinic = data
        .clinic_id
        .as_deref()
        .map(|c| c.replace("clinic:", ""))
        .unwrap_or_else(|| "general".to_string());
    let module = data.module.as_deref().unwrap_or("documents");
    let prefix = format!("clinics/{}/{}", clean_clinic, module);

    let file_url = storage
        .upload_file(&prefix, ext, &data.base64_content)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro no upload de PDF: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "url": file_url,
        "filename": data.filename
    })))
}

/// Fornece o PDF de exemplo com guia de placeholders/tags suportadas pelo sistema.
#[get("/documents/sample-template")]
pub async fn get_sample_template_pdf() -> Result<HttpResponse, ApiError> {
    let bytes = crate::documents_pdf::generate_placeholder_guide_pdf_bytes();
    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            actix_web::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"modelo_placeholders_tooth_plus.pdf\"",
        ))
        .body(bytes))
}
