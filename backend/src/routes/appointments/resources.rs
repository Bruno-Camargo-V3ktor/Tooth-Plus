//! # Recursos e Opções do Calendário de Atendimento (Backend)
//!
//! Fornece membros da equipe, pacientes cadastrados e insumos odontológicos
//! para preenchimento ágil de formulários e filtros de agenda.

use super::{clinic_record_id, ClinicQuery, DbResourceRecord};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{get, web, HttpResponse};
use shared::appointments::{AgendaResourceOption, AgendaResourcesResponse};
use surrealdb::types::ToSql;

/// Retorna listas de profissionais, pacientes e itens de estoque para alimentar os seletores da agenda.
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

    Ok(HttpResponse::Ok().json(AgendaResourcesResponse {
        team_members,
        patients,
        inventory_items,
        equipment_items,
    }))
}
