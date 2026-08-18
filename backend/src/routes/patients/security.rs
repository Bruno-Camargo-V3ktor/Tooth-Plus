//! # Gestão de Segurança e Senha de Assinatura do Paciente (Backend)
//!
//! Controla o reset administrativo da senha de assinatura digital do paciente.

use super::{clinic_record_id, parse_record_id, crud::PatientPathQuery};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{post, web, HttpResponse};

/// Reseta a senha de assinatura digital do paciente para que ele possa cadastrar uma nova no portal.
#[post("/patients/{id}/reset-password")]
pub async fn reset_patient_password(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<PatientPathQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "patients:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para resetar senha do paciente.".into(),
        ));
    }

    db.query("UPDATE type::record($pid) SET password_hash = NONE, updated_at = time::now()")
        .bind(("pid", pat_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao resetar senha: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Senha de assinatura do paciente resetada com sucesso. O paciente poderá cadastrar uma nova senha ao acessar o portal de assinatura."
    })))
}
