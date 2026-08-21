//! # Recursos e Opções do Calendário de Atendimento (Backend)
//!
//! Fornece membros da equipe, pacientes cadastrados, insumos odontológicos
//! e procedimentos clínicos pendentes para vinculação direta nos agendamentos.

use super::{clinic_record_id, ClinicQuery, DbResourceRecord};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use shared::appointments::{
    AgendaResourceOption, AgendaResourcesResponse, AgendaTreatmentOption,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

#[derive(Deserialize, SurrealValue)]
struct DbPendingTreatRow {
    id: RecordId,
    patient_id: RecordId,
    patient_name: Option<String>,
    procedure_name: String,
    procedure_category: Option<String>,
    tooth_number: Option<String>,
    cost_cents: Option<i64>,
    treatment_plan_id: Option<RecordId>,
}

/// Retorna listas de profissionais, pacientes, itens de estoque e procedimentos pendentes.
#[get("/appointments/resources")]
pub async fn get_agenda_resources(
    auth: AuthenticatedUser,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_rec = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "appointments:read")
        .await
        .unwrap_or(false)
        && !check_permission(&db, &auth.id, &clinic_rec, "agenda:read")
            .await
            .unwrap_or(false)
    {
        return Err(ApiError::Forbidden("Sem privilégios de acesso.".into()));
    }

    let mut team_resp = db
        .query(
            "SELECT
                in           AS id,
                in.full_name AS name,
                role         AS extra_info
            FROM works_at
            WHERE out = type::record($clinic_id)",
        )
        .bind(("clinic_id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar profissionais.".into()))?;

    let team_raw: Vec<DbResourceRecord> = team_resp.take(0).unwrap_or_default();
    let team_members = team_raw
        .into_iter()
        .map(|r| AgendaResourceOption {
            id: r.id.to_sql(),
            name: r.name,
            extra_info: r.extra_info,
        })
        .collect();

    let mut patients_resp = db
        .query(
            "SELECT
                id,
                full_name AS name,
                phone     AS extra_info
            FROM patient
            WHERE clinic_id = type::record($clinic_id)",
        )
        .bind(("clinic_id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar pacientes.".into()))?;

    let patients_raw: Vec<DbResourceRecord> = patients_resp.take(0).unwrap_or_default();
    let patients = patients_raw
        .into_iter()
        .map(|r| AgendaResourceOption {
            id: r.id.to_sql(),
            name: r.name,
            extra_info: r.extra_info,
        })
        .collect();

    let mut items_resp = db
        .query(
            "SELECT
                id,
                name,
                unit AS extra_info
            FROM inventory_item
            WHERE clinic_id = type::record($clinic_id) AND item_type != 'equipment'",
        )
        .bind(("clinic_id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar itens de estoque.".into()))?;

    let items_raw: Vec<DbResourceRecord> = items_resp.take(0).unwrap_or_default();
    let inventory_items = items_raw
        .into_iter()
        .map(|r| AgendaResourceOption {
            id: r.id.to_sql(),
            name: r.name,
            extra_info: r.extra_info,
        })
        .collect();

    let mut equip_resp = db
        .query(
            "SELECT
                id,
                name,
                unit AS extra_info
            FROM inventory_item
            WHERE clinic_id = type::record($clinic_id) AND item_type = 'equipment'",
        )
        .bind(("clinic_id", clinic_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar equipamentos odontológicos.".into()))?;

    let equip_raw: Vec<DbResourceRecord> = equip_resp.take(0).unwrap_or_default();
    let equipment_items = equip_raw
        .into_iter()
        .map(|r| AgendaResourceOption {
            id: r.id.to_sql(),
            name: r.name,
            extra_info: r.extra_info,
        })
        .collect();

    let mut treats_resp = db
        .query(
            "SELECT
                id,
                patient_id,
                patient_id.full_name AS patient_name,
                procedure_name,
                procedure_category,
                tooth_number,
                cost_cents,
                treatment_plan_id
            FROM patient_treatment
            WHERE clinic_id = type::record($clinic_id)
            AND (status = 'pending' OR status = 'planned')
            ORDER BY created_at DESC",
        )
        .bind(("clinic_id", clinic_rec.clone()))
        .await;

    let treats_raw: Vec<DbPendingTreatRow> = treats_resp
        .as_mut()
        .ok()
        .and_then(|r| r.take::<Vec<DbPendingTreatRow>>(0).ok())
        .unwrap_or_default();

    let pending_treatments = treats_raw
        .into_iter()
        .map(|t| AgendaTreatmentOption {
            id: t.id.to_sql(),
            patient_id: t.patient_id.to_sql(),
            patient_name: t.patient_name.unwrap_or_else(|| "Paciente".into()),
            procedure_name: t.procedure_name,
            category: t.procedure_category,
            tooth_number: t.tooth_number,
            cost_cents: t.cost_cents.unwrap_or(0),
            treatment_plan_id: t.treatment_plan_id.map(|p| p.to_sql()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(AgendaResourcesResponse {
        team_members,
        patients,
        inventory_items,
        equipment_items,
        pending_treatments,
    }))
}
