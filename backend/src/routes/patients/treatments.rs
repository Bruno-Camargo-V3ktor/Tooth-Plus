//! # Procedimentos Odontológicos e Tratamentos (Backend)
//!
//! Controla o registro de procedimentos clínicos, dente tratado e status da evolução.

use super::{clinic_record_id, parse_record_id, DbTreatmentRow};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, post, put, web, HttpResponse};
use shared::patients::{
    CreatePatientTreatmentRequest, PatientTreatment, UpdatePatientTreatmentRequest,
};
use surrealdb::types::ToSql;

/// Query para exclusão de procedimento
#[derive(serde::Deserialize)]
pub struct DeleteTreatmentQuery {
    pub clinic_id: String,
}

/// Registra um novo procedimento odontológico vinculado ao paciente e dentista responsável.
#[post("/patients/{id}/treatments")]
pub async fn create_treatment(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<CreatePatientTreatmentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para adicionar procedimentos/tratamentos.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);

    let dentist_rec = if let Some(ref d_id) = data.dentist_user_id {
        parse_record_id("user", d_id)
    } else {
        parse_record_id("user", &auth.id)
    };

    let appt_rec = data
        .appointment_id
        .as_deref()
        .map(|a| parse_record_id("appointment", a));

    let doc_rec = data
        .document_id
        .as_deref()
        .map(|d| parse_record_id("patient_document", d));

    let exam_rec = data
        .exam_id
        .as_deref()
        .map(|e| parse_record_id("patient_exam", e));

    let plan_rec = data
        .treatment_plan_id
        .as_deref()
        .map(|p| parse_record_id("patient_treatment_plan", p));

    let tx_rec = data
        .transaction_id
        .as_deref()
        .map(|t| parse_record_id("transaction", t));

    let surfaces_list = data.surfaces.unwrap_or_default();
    let materials_list = data.materials_used.unwrap_or_default();

    let mut res = db
        .query(
            "CREATE patient_treatment CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            dentist_user_id: $uid,
            appointment_id: $aid,
            document_id: $did,
            exam_id: $eid,
            treatment_plan_id: $plan_id,
            transaction_id: $tx_id,
            procedure_category: $pcat,
            procedure_name: $pname,
            tooth_number: $tooth,
            surfaces: $surfaces,
            materials_used: $materials,
            status: $status,
            cost_cents: $cost,
            post_care_instructions: $post_care,
            clinical_notes: $notes,
            performed_at: time::now(),
            created_at: time::now()
        };",
        )
        .bind(("pid", pat_rec))
        .bind(("cid", clinic_rec))
        .bind(("uid", dentist_rec))
        .bind(("aid", appt_rec))
        .bind(("did", doc_rec))
        .bind(("eid", exam_rec))
        .bind(("plan_id", plan_rec))
        .bind(("tx_id", tx_rec))
        .bind(("pcat", data.procedure_category.clone()))
        .bind(("pname", data.procedure_name.trim().to_string()))
        .bind(("tooth", data.tooth_number.map(|s| s.trim().to_string())))
        .bind(("surfaces", surfaces_list.clone()))
        .bind(("materials", materials_list.clone()))
        .bind(("status", data.status.clone()))
        .bind(("cost", data.cost_cents))
        .bind(("post_care", data.post_care_instructions.map(|s| s.trim().to_string())))
        .bind(("notes", data.clinical_notes.map(|s| s.trim().to_string())))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar tratamento: {}", e)))?;

    let created: Option<DbTreatmentRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(t) = created else {
        return Err(ApiError::Database("Erro ao salvar procedimento.".into()));
    };

    Ok(HttpResponse::Created().json(PatientTreatment {
        id: t.id.to_sql(),
        patient_id: t.patient_id.to_sql(),
        clinic_id: t.clinic_id.to_sql(),
        dentist_user_id: t.dentist_user_id.map(|u| u.to_sql()),
        dentist_user_name: None,
        appointment_id: t.appointment_id.map(|a| a.to_sql()),
        document_id: t.document_id.map(|d| d.to_sql()),
        exam_id: t.exam_id.map(|e| e.to_sql()),
        treatment_plan_id: t.treatment_plan_id.map(|p| p.to_sql()),
        transaction_id: t.transaction_id.map(|x| x.to_sql()),
        procedure_category: t.procedure_category,
        procedure_name: t.procedure_name,
        tooth_number: t.tooth_number,
        surfaces: t.surfaces.or(Some(surfaces_list)),
        materials_used: t.materials_used.or(Some(materials_list)),
        status: t.status.unwrap_or(data.status),
        cost_cents: t.cost_cents.unwrap_or(data.cost_cents),
        post_care_instructions: t.post_care_instructions,
        clinical_notes: t.clinical_notes,
        performed_at: t.performed_at.map(|d| d.to_rfc3339()),
        created_at: t.created_at.to_rfc3339(),
    }))
}

/// Exclui um registro de procedimento/tratamento do prontuário.
#[delete("/patients/{id}/treatments/{treatment_id}")]
pub async fn delete_treatment(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    query: web::Query<DeleteTreatmentQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (_pat_id, treat_id) = path.into_inner();
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para remover procedimentos/tratamentos.".into(),
        ));
    }


    let treat_rec = parse_record_id("patient_treatment", &treat_id);

    db.query("DELETE type::record($tid);")
        .bind(("tid", treat_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir procedimento: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Procedimento removido com sucesso."
    })))
}

/// Atualiza um procedimento odontológico / evolução clínica.
#[put("/patients/{id}/treatments/{treatment_id}")]
pub async fn update_treatment(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    req: web::Json<UpdatePatientTreatmentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (_pat_id, treat_id) = path.into_inner();
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para editar procedimentos/tratamentos.".into(),
        ));
    }

    let treat_rec = parse_record_id("patient_treatment", &treat_id);

    let dentist_rec = if let Some(ref d_id) = data.dentist_user_id {
        Some(parse_record_id("user", d_id))
    } else {
        None
    };

    let plan_rec = data
        .treatment_plan_id
        .as_deref()
        .map(|p| parse_record_id("patient_treatment_plan", p));

    let tx_rec = data
        .transaction_id
        .as_deref()
        .map(|t| parse_record_id("transaction", t));

    let surfaces_list = data.surfaces.unwrap_or_default();
    let materials_list = data.materials_used.unwrap_or_default();

    let mut res = db
        .query(
            "UPDATE type::record($tid) SET
            procedure_category = $pcat,
            procedure_name = $pname,
            tooth_number = $tooth,
            surfaces = $surfaces,
            materials_used = $materials,
            status = $status,
            cost_cents = $cost,
            post_care_instructions = $post_care,
            clinical_notes = $notes,
            dentist_user_id = IF $uid != NONE THEN $uid ELSE dentist_user_id END,
            treatment_plan_id = IF $plan_id != NONE THEN $plan_id ELSE treatment_plan_id END,
            transaction_id = IF $tx_id != NONE THEN $tx_id ELSE transaction_id END,
            updated_at = time::now();",
        )
        .bind(("tid", treat_rec))
        .bind(("pcat", data.procedure_category.clone()))
        .bind(("pname", data.procedure_name.trim().to_string()))
        .bind(("tooth", data.tooth_number.map(|s| s.trim().to_string())))
        .bind(("surfaces", surfaces_list.clone()))
        .bind(("materials", materials_list.clone()))
        .bind(("status", data.status.clone()))
        .bind(("cost", data.cost_cents))
        .bind(("post_care", data.post_care_instructions.map(|s| s.trim().to_string())))
        .bind(("notes", data.clinical_notes.map(|s| s.trim().to_string())))
        .bind(("uid", dentist_rec))
        .bind(("plan_id", plan_rec))
        .bind(("tx_id", tx_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar tratamento: {}", e)))?;

    let updated: Option<DbTreatmentRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(t) = updated else {
        return Err(ApiError::NotFound("Procedimento não encontrado para atualização.".into()));
    };

    Ok(HttpResponse::Ok().json(PatientTreatment {
        id: t.id.to_sql(),
        patient_id: t.patient_id.to_sql(),
        clinic_id: t.clinic_id.to_sql(),
        dentist_user_id: t.dentist_user_id.map(|u| u.to_sql()),
        dentist_user_name: None,
        appointment_id: t.appointment_id.map(|a| a.to_sql()),
        document_id: t.document_id.map(|d| d.to_sql()),
        exam_id: t.exam_id.map(|e| e.to_sql()),
        treatment_plan_id: t.treatment_plan_id.map(|p| p.to_sql()),
        transaction_id: t.transaction_id.map(|x| x.to_sql()),
        procedure_category: t.procedure_category,
        procedure_name: t.procedure_name,
        tooth_number: t.tooth_number,
        surfaces: t.surfaces.or(Some(surfaces_list)),
        materials_used: t.materials_used.or(Some(materials_list)),
        status: t.status.unwrap_or(data.status),
        cost_cents: t.cost_cents.unwrap_or(data.cost_cents),
        post_care_instructions: t.post_care_instructions,
        clinical_notes: t.clinical_notes,
        performed_at: t.performed_at.map(|d| d.to_rfc3339()),
        created_at: t.created_at.to_rfc3339(),
    }))
}


