//! # Exames e Galeria de Fotos / Radiografias (Backend)
//!
//! Controla o anexo de exames radiológicos, tomografias e fotografias intraorais
//! ao prontuário do paciente.

use super::{clinic_record_id, parse_record_id, DbExamRow};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, post, put, web, HttpResponse};
use chrono::Utc;
use shared::patients::{CreatePatientExamRequest, PatientExam, UpdatePatientExamRequest};
use surrealdb::types::ToSql;

/// Query para exclusão de exame
#[derive(serde::Deserialize)]
pub struct DeleteExamQuery {
    pub clinic_id: String,
}

/// Registra um exame ou fotos no prontuário do paciente com URLs dos arquivos anexados.
#[post("/patients/{id}/exams")]
pub async fn create_exam(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<CreatePatientExamRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "exams:upload")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para adicionar exames ao paciente.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    let auth_rec = parse_record_id("user", &auth.id);

    let mut res = db
        .query(
            "CREATE patient_exam CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            title: $title,
            exam_type: $etype,
            requested_by_user_id: $uid,
            status: 'received',
            file_urls: $urls,
            clinical_interpretation: $notes,
            requested_date: time::now(),
            result_date: time::now(),
            created_at: time::now()
        };",
        )
        .bind(("pid", pat_rec))
        .bind(("cid", clinic_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("etype", data.exam_type))
        .bind(("uid", auth_rec))
        .bind(("urls", data.file_urls.clone()))
        .bind(("notes", data.clinical_interpretation.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao registrar exame: {}", e)))?;

    let created: Option<DbExamRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(e) = created else {
        return Err(ApiError::Database("Erro ao salvar exame.".into()));
    };

    Ok(HttpResponse::Created().json(PatientExam {
        id: e.id.to_sql(),
        patient_id: e.patient_id.to_sql(),
        clinic_id: e.clinic_id.to_sql(),
        title: e.title,
        exam_type: e.exam_type,
        requested_by_user_id: e.requested_by_user_id.map(|u| u.to_sql()),
        requested_by_user_name: None,
        status: e.status.unwrap_or_else(|| "received".into()),
        requested_date: e
            .requested_date
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        result_date: e.result_date.map(|d| d.to_rfc3339()),
        file_urls: e.file_urls.unwrap_or(data.file_urls),
        clinical_interpretation: e.clinical_interpretation.or(data.clinical_interpretation),
        created_at: e.created_at.to_rfc3339(),
    }))
}

/// Exclui um exame anexado ao prontuário.
#[delete("/patients/{id}/exams/{exam_id}")]
pub async fn delete_exam(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    query: web::Query<DeleteExamQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (_pat_id, exam_id) = path.into_inner();
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "exams:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para remover exames do paciente.".into(),
        ));
    }

    let exam_rec = parse_record_id("patient_exam", &exam_id);

    db.query("DELETE type::record($eid);")
        .bind(("eid", exam_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir exame: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Exame excluído com sucesso."
    })))
}

/// Atualiza os dados ou laudo de um exame já anexado.
#[put("/patients/{id}/exams/{exam_id}")]
pub async fn update_exam(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    req: web::Json<UpdatePatientExamRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (_pat_id, exam_id) = path.into_inner();
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "exams:upload")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para editar exames do paciente.".into(),
        ));
    }

    let exam_rec = parse_record_id("patient_exam", &exam_id);

    let mut res = db
        .query(
            "UPDATE type::record($eid) SET
            title = $title,
            exam_type = $etype,
            status = $status,
            file_urls = $urls,
            clinical_interpretation = $notes,
            updated_at = time::now();",
        )
        .bind(("eid", exam_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("etype", data.exam_type))
        .bind(("status", data.status.unwrap_or_else(|| "received".into())))
        .bind(("urls", data.file_urls.clone()))
        .bind(("notes", data.clinical_interpretation.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar exame: {}", e)))?;

    let updated: Option<DbExamRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(e) = updated else {
        return Err(ApiError::NotFound("Exame não encontrado para atualização.".into()));
    };

    Ok(HttpResponse::Ok().json(PatientExam {
        id: e.id.to_sql(),
        patient_id: e.patient_id.to_sql(),
        clinic_id: e.clinic_id.to_sql(),
        title: e.title,
        exam_type: e.exam_type,
        requested_by_user_id: e.requested_by_user_id.map(|u| u.to_sql()),
        requested_by_user_name: None,
        status: e.status.unwrap_or_else(|| "received".into()),
        requested_date: e
            .requested_date
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        result_date: e.result_date.map(|d| d.to_rfc3339()),
        file_urls: e.file_urls.unwrap_or(data.file_urls),
        clinical_interpretation: e.clinical_interpretation.or(data.clinical_interpretation),
        created_at: e.created_at.to_rfc3339(),
    }))
}


