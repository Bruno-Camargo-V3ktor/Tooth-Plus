//! # Ficha de Anamnese Odontológica (Backend)
//!
//! Controla o histórico de saúde sistêmica, alergias, medicações de uso contínuo,
//! hábitos e queixa principal do paciente.

use super::{clinic_record_id, parse_record_id, DbAnamnesisRow};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use shared::patients::{PatientAnamnesis, SaveAnamnesisRequest};
use surrealdb::types::ToSql;

/// Salva ou atualiza a ficha de anamnese do paciente com histórico médico e hábitos.
#[post("/patients/{id}/anamnesis")]
pub async fn save_anamnesis(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<SaveAnamnesisRequest>,
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
            "Sem permissão para atualizar a ficha médica/anamnese.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);

    let mut res = db
        .query(
            "UPSERT patient_anamnesis SET
            patient_id = $pid,
            clinic_id = $cid,
            allergies = $allergies,
            continuous_medications = $meds,
            systemic_diseases = $diseases,
            is_pregnant = $preg,
            has_bleeding_disorder = $bleed,
            smoker = $smoker,
            bruxism = $brux,
            chief_complaint = $complaint,
            clinical_notes = $notes,
            updated_at = time::now()
            WHERE patient_id = $pid;",
        )
        .bind(("pid", pat_rec.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("allergies", data.allergies.clone()))
        .bind(("meds", data.continuous_medications.clone()))
        .bind(("diseases", data.systemic_diseases.clone()))
        .bind(("preg", data.is_pregnant))
        .bind(("bleed", data.has_bleeding_disorder))
        .bind(("smoker", data.smoker))
        .bind(("brux", data.bruxism))
        .bind(("complaint", data.chief_complaint.clone()))
        .bind(("notes", data.clinical_notes.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao salvar anamnese: {}", e)))?;

    let saved: Option<DbAnamnesisRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(a) = saved else {
        return Err(ApiError::Database(
            "Erro ao recuperar registro de anamnese.".into(),
        ));
    };

    Ok(HttpResponse::Ok().json(PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
        allergies: a.allergies.unwrap_or(data.allergies),
        continuous_medications: a.continuous_medications.or(data.continuous_medications),
        systemic_diseases: a.systemic_diseases.unwrap_or(data.systemic_diseases),
        is_pregnant: a.is_pregnant.unwrap_or(data.is_pregnant),
        has_bleeding_disorder: a
            .has_bleeding_disorder
            .unwrap_or(data.has_bleeding_disorder),
        smoker: a.smoker.unwrap_or(data.smoker),
        bruxism: a.bruxism.unwrap_or(data.bruxism),
        chief_complaint: a.chief_complaint.or(data.chief_complaint),
        clinical_notes: a.clinical_notes.or(data.clinical_notes),
        updated_at: a
            .updated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    }))
}
