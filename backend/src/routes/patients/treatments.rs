//! # Procedimentos Odontológicos e Tratamentos (Backend)
//!
//! Controla o registro de procedimentos clínicos, dente tratado e status da evolução.

use super::{clinic_record_id, parse_record_id, DbTreatmentRow};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{post, web, HttpResponse};
use shared::patients::{CreatePatientTreatmentRequest, PatientTreatment};
use surrealdb::types::ToSql;

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

    if !check_permission(&db, &auth.id, &clinic_str, "patients:write")
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

    let mut res = db
        .query(
            "CREATE patient_treatment CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            dentist_user_id: $uid,
            appointment_id: $aid,
            procedure_name: $pname,
            tooth_number: $tooth,
            status: $status,
            cost_cents: $cost,
            clinical_notes: $notes,
            created_at: time::now()
        };",
        )
        .bind(("pid", pat_rec))
        .bind(("cid", clinic_rec))
        .bind(("uid", dentist_rec))
        .bind(("aid", appt_rec))
        .bind(("pname", data.procedure_name.trim().to_string()))
        .bind(("tooth", data.tooth_number.map(|s| s.trim().to_string())))
        .bind(("status", data.status.clone()))
        .bind(("cost", data.cost_cents))
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
        procedure_name: t.procedure_name,
        tooth_number: t.tooth_number,
        status: t.status.unwrap_or(data.status),
        cost_cents: t.cost_cents.unwrap_or(data.cost_cents),
        clinical_notes: t.clinical_notes,
        created_at: t.created_at.to_rfc3339(),
    }))
}
