//! # Agendamento e Gestão de Consultas (Backend)
//!
//! Controla a listagem, criação, edição e exclusão de agendamentos odontológicos,
//! vinculação de profissionais responsáveis e plano de consumo de materiais.

use super::{
    appointment_record_id, clinic_record_id, parse_record_id, parse_status, parse_type,
    patient_record_id, type_to_str, ClinicQuery, DbAppointmentRecord, DbAssignedRecord,
    DbConsumedRecord,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, get, post, put, web, HttpResponse};
use serde::Deserialize;
use shared::appointments::{
    AppointmentResponse, AssignedUserDto, ConsumedItemDto, CreateAppointmentRequest,
    UpdateAppointmentRequest,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// Query string para filtragem de consultas por clínica, data, profissional ou status.
#[derive(Deserialize)]
pub struct AppointmentQuery {
    pub clinic_id: String,
    pub date: Option<String>,
    pub user_id: Option<String>,
    pub status: Option<String>,
}

/// Lista os agendamentos da clínica com filtros opcionais por dia, profissional ou status.
#[get("/appointments")]
pub async fn list_appointments(
    auth: AuthenticatedUser,
    query: web::Query<AppointmentQuery>,
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
        return Err(ApiError::Forbidden(
            "Sem privilégios para visualizar a agenda desta unidade.".into(),
        ));
    }

    let mut sql = String::from(
        "SELECT
            id,
            clinic_id,
            patient_id,
            patient_name,
            title,
            scheduled_for,
            duration_minutes,
            status,
            appointment_type,
            financial_amount_cents,
            financial_type,
            room,
            notes,
            cancellation_reason
        FROM appointment
        WHERE clinic_id = type::record($clinic_id)",
    );

    if let Some(ref d) = query.date {
        if !d.trim().is_empty() {
            sql.push_str(" AND type::string(scheduled_for) CONTAINS $filter_date");
        }
    }

    if let Some(ref st) = query.status {
        if !st.trim().is_empty() && st != "all" {
            sql.push_str(" AND status = $filter_status");
        }
    }

    sql.push_str(" ORDER BY scheduled_for ASC");

    let mut q = db.query(&sql).bind(("clinic_id", clinic_rec.clone()));

    if let Some(ref d) = query.date {
        if !d.trim().is_empty() {
            q = q.bind(("filter_date", d.clone()));
        }
    }

    if let Some(ref st) = query.status {
        if !st.trim().is_empty() && st != "all" {
            q = q.bind(("filter_status", st.clone()));
        }
    }

    let mut response = q
        .await
        .map_err(|_| ApiError::Database("Falha ao buscar agendamentos.".into()))?;

    let db_appointments: Vec<DbAppointmentRecord> = response.take(0).unwrap_or_default();
    let mut results: Vec<AppointmentResponse> = Vec::new();

    let can_finance = check_permission(&db, &auth.id, &clinic_rec, "appointments:finance")
        .await
        .unwrap_or(false)
        || check_permission(&db, &auth.id, &clinic_rec, "agenda:finance")
            .await
            .unwrap_or(false);

    for app in db_appointments {
        let app_rec_str = app.id.to_sql();

        let mut assigned_resp = db
            .query(
                "SELECT
                    out             AS user_id,
                    out.full_name   AS user_name,
                    role_in_appointment,
                    split_percentage
                FROM assigned_to
                WHERE in = type::record($app_id)",
            )
            .bind(("app_id", app_rec_str.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao buscar membros do agendamento.".into()))?;

        let db_assigned: Vec<DbAssignedRecord> = assigned_resp.take(0).unwrap_or_default();

        if let Some(ref u_filter) = query.user_id {
            if !u_filter.trim().is_empty()
                && u_filter != "all"
                && !db_assigned.iter().any(|a| {
                    a.user_id.to_sql().contains(u_filter)
                        || a.user_id.key.to_sql().contains(u_filter)
                })
            {
                continue;
            }
        }

        let mut consumed_resp = db
            .query(
                "SELECT
                    out          AS item_id,
                    out.name     AS item_name,
                    quantity_planned,
                    quantity_used
                FROM consumes
                WHERE in = type::record($app_id)",
            )
            .bind(("app_id", app_rec_str.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao buscar itens consumidos.".into()))?;

        let db_consumed: Vec<DbConsumedRecord> = consumed_resp.take(0).unwrap_or_default();

        #[derive(Deserialize, Debug, SurrealValue)]
        struct EquipRow {
            name: Option<String>,
        }

        let mut equip_resp = db
            .query(
                "SELECT out.name AS name FROM uses_equipment WHERE in = type::record($app_id)",
            )
            .bind(("app_id", app_rec_str.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao buscar equipamentos do agendamento.".into()))?;

        let db_equip: Vec<EquipRow> = equip_resp.take(0).unwrap_or_default();
        let assigned_equipment: Vec<String> = db_equip.into_iter().filter_map(|e| e.name).collect();

        results.push(AppointmentResponse {
            id: app.id.key.to_sql(),
            clinic_id: app.clinic_id.to_sql(),
            patient_id: app.patient_id.map(|p| p.to_sql()),
            patient_name: app.patient_name,
            treatment_id: None,
            treatment_plan_id: None,
            title: app.title,
            scheduled_for: app.scheduled_for.to_rfc3339(),
            duration_minutes: app.duration_minutes,
            status: parse_status(&app.status),
            appointment_type: parse_type(&app.appointment_type),
            financial_amount_cents: if can_finance { app.financial_amount_cents } else { None },
            financial_type: if can_finance { app.financial_type } else { None },
            notes: app.notes,
            cancellation_reason: app.cancellation_reason,
            assigned_users: db_assigned
                .into_iter()
                .map(|a| AssignedUserDto {
                    user_id: a.user_id.to_sql(),
                    user_name: a.user_name,
                    role_in_appointment: a.role_in_appointment,
                    split_percentage: if can_finance { a.split_percentage } else { 0 },
                })
                .collect(),
            consumed_items: db_consumed
                .into_iter()
                .map(|c| ConsumedItemDto {
                    item_id: c.item_id.to_sql(),
                    item_name: c.item_name,
                    quantity_planned: c.quantity_planned,
                    quantity_used: c.quantity_used,
                })
                .collect(),
            assigned_equipment,
        });
    }


    Ok(HttpResponse::Ok().json(results))
}

/// Cria um novo agendamento com os profissionais e materiais associados.
#[post("/appointments")]
pub async fn create_appointment(
    auth: AuthenticatedUser,
    req: web::Json<CreateAppointmentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let clinic_rec = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "appointments:write")
        .await
        .unwrap_or(false)
        && !check_permission(&db, &auth.id, &clinic_rec, "agenda:write")
            .await
            .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para criar agendamentos nesta unidade.".into(),
        ));
    }

    let can_finance = check_permission(&db, &auth.id, &clinic_rec, "appointments:finance")
        .await
        .unwrap_or(false)
        || check_permission(&db, &auth.id, &clinic_rec, "agenda:finance")
            .await
            .unwrap_or(false);

    let (fin_amount, fin_type) = if can_finance {
        (data.financial_amount_cents, data.financial_type)
    } else {
        (None, None)
    };

    if data.assigned_users.is_empty() {
        return Err(ApiError::BadRequest(
            "Vincule ao menos um profissional responsável ao agendamento.".into(),
        ));
    }

    let parsed_dt = chrono::DateTime::parse_from_rfc3339(&data.scheduled_for)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| ApiError::BadRequest("Formato de data e hora inválido.".into()))?;

    let patient_rec = data.patient_id.as_ref().map(|p| patient_record_id(p));

    let mut create_resp = db
        .query(
            "CREATE appointment SET
                clinic_id              = type::record($clinic_id),
                patient_id             = IF $patient_id != NONE THEN type::record($patient_id) ELSE NONE END,
                patient_name           = $patient_name,
                title                  = $title,
                scheduled_for          = $scheduled_for,
                duration_minutes       = $duration_minutes,
                status                 = 'pending',
                appointment_type       = $appointment_type,
                financial_amount_cents = $financial_amount,
                financial_type         = $financial_type,
                notes                  = $notes
            RETURN id",
        )
        .bind(("clinic_id", clinic_rec.clone()))
        .bind(("patient_id", patient_rec))
        .bind(("patient_name", data.patient_name))
        .bind(("title", data.title))
        .bind(("scheduled_for", parsed_dt))
        .bind(("duration_minutes", data.duration_minutes))
        .bind(("appointment_type", type_to_str(&data.appointment_type)))
        .bind(("financial_amount", fin_amount))
        .bind(("financial_type", fin_type))
        .bind(("notes", data.notes))
        .await
        .map_err(|e| ApiError::Database(format!("Falha ao criar agendamento: {}", e)))?;

    #[derive(Deserialize, SurrealValue)]
    struct CreatedId {
        id: RecordId,
    }

    let created: Vec<CreatedId> = create_resp.take(0).unwrap_or_default();
    let new_app_id = created
        .into_iter()
        .next()
        .map(|c| c.id)
        .ok_or_else(|| ApiError::Database("Agendamento não retornou ID.".into()))?;

    for u in &data.assigned_users {
        let u_rec = parse_record_id("user", &u.user_id);
        let split_val = if can_finance { u.split_percentage } else { 100 };
        db.query(
            "RELATE $app_id->assigned_to->$user_id SET
                role_in_appointment = $role,
                split_percentage    = $split",
        )
        .bind(("app_id", new_app_id.clone()))
        .bind(("user_id", u_rec))
        .bind(("role", u.role_in_appointment.clone()))
        .bind(("split", split_val))
        .await
        .map_err(|_| ApiError::Database("Falha ao vincular profissional ao agendamento.".into()))?;
    }

    for item in &data.consumed_items {
        let item_rec = parse_record_id("inventory_item", &item.item_id);
        db.query(
            "RELATE $app_id->consumes->$item_id SET
                quantity_planned = $qty,
                quantity_used    = NONE",
        )
        .bind(("app_id", new_app_id.clone()))
        .bind(("item_id", item_rec))
        .bind(("qty", item.quantity_planned))
        .await
        .map_err(|_| ApiError::Database("Falha ao associar item de estoque.".into()))?;
    }

    if let Some(ref equip_list) = data.assigned_equipment {
        for equip_id in equip_list {
            let equip_rec = parse_record_id("inventory_item", equip_id);
            db.query("RELATE $app_id->uses_equipment->$equip_id")
                .bind(("app_id", new_app_id.clone()))
                .bind(("equip_id", equip_rec))
                .await
                .map_err(|_| ApiError::Database("Falha ao alocar equipamento.".into()))?;
        }
    }

    // Se vinculado a um procedimento do prontuário, atualiza status para agendado e associa ID da consulta
    if let Some(ref treat_id) = data.treatment_id {
        let treat_rec = parse_record_id("patient_treatment", treat_id);
        let _ = db
            .query(
                "UPDATE type::record($tid) SET
                appointment_id = $aid,
                status = 'scheduled',
                performed_at = $perf,
                updated_at = time::now();",
            )
            .bind(("tid", treat_rec))
            .bind(("aid", new_app_id.clone()))
            .bind(("perf", parsed_dt))
            .await;
    }

    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "id": new_app_id.key.to_sql(),
        "message": "Agendamento criado com sucesso."
    })))
}

/// Atualiza as informações de uma consulta existente na agenda.
#[put("/appointments/{id}")]
pub async fn update_appointment(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<UpdateAppointmentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let app_id = path.into_inner();
    let data = req.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);
    let app_rec = appointment_record_id(&app_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "appointments:write")
        .await
        .unwrap_or(false)
        && !check_permission(&db, &auth.id, &clinic_rec, "agenda:write")
            .await
            .unwrap_or(false)
    {
        return Err(ApiError::Forbidden("Sem privilégios de edição.".into()));
    }

    let can_finance = check_permission(&db, &auth.id, &clinic_rec, "appointments:finance")
        .await
        .unwrap_or(false)
        || check_permission(&db, &auth.id, &clinic_rec, "agenda:finance")
            .await
            .unwrap_or(false);

    let mut patch = serde_json::Map::new();

    if let Some(ref t) = data.title {
        patch.insert("title".into(), serde_json::Value::String(t.clone()));
    }

    if let Some(ref sf) = data.scheduled_for {
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(sf) {
            let utc_dt = parsed.with_timezone(&chrono::Utc);
            patch.insert(
                "scheduled_for".into(),
                serde_json::Value::String(utc_dt.to_rfc3339()),
            );
        }
    }

    if let Some(dur) = data.duration_minutes {
        patch.insert(
            "duration_minutes".into(),
            serde_json::Value::Number(dur.into()),
        );
    }

    if let Some(ref at) = data.appointment_type {
        patch.insert(
            "appointment_type".into(),
            serde_json::Value::String(type_to_str(at).into()),
        );
    }

    if let Some(ref p_name) = data.patient_name {
        patch.insert(
            "patient_name".into(),
            serde_json::Value::String(p_name.clone()),
        );
    }

    if let Some(ref p_id) = data.patient_id {
        patch.insert(
            "patient_id".into(),
            serde_json::Value::String(patient_record_id(p_id)),
        );
    }

    if can_finance {
        if let Some(amt) = data.financial_amount_cents {
            patch.insert(
                "financial_amount_cents".into(),
                serde_json::Value::Number(amt.into()),
            );
        }

        if let Some(ref ft) = data.financial_type {
            patch.insert(
                "financial_type".into(),
                serde_json::Value::String(ft.clone()),
            );
        }
    }

    if let Some(ref nt) = data.notes {
        patch.insert("notes".into(), serde_json::Value::String(nt.clone()));
    }

    if !patch.is_empty() {
        db.query("UPDATE type::record($app_id) MERGE $patch")
            .bind(("app_id", app_rec.clone()))
            .bind(("patch", serde_json::Value::Object(patch)))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar dados do agendamento.".into()))?;
    }

    let app_rec_id = parse_record_id("appointment", &app_id);

    if let Some(ref assigned_users) = data.assigned_users {
        db.query("DELETE assigned_to WHERE in = $app_id")
            .bind(("app_id", app_rec_id.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar membros.".into()))?;

        for u in assigned_users {
            let u_rec = parse_record_id("user", &u.user_id);
            let split_val = if can_finance { u.split_percentage } else { 100 };
            db.query(
                "RELATE $app_id->assigned_to->$user_id SET
                    role_in_appointment = $role,
                    split_percentage    = $split",
            )
            .bind(("app_id", app_rec_id.clone()))
            .bind(("user_id", u_rec))
            .bind(("role", u.role_in_appointment.clone()))
            .bind(("split", split_val))
            .await
            .map_err(|_| ApiError::Database("Falha ao vincular membros.".into()))?;
        }
    }

    if let Some(ref consumed_items) = data.consumed_items {
        db.query("DELETE consumes WHERE in = $app_id")
            .bind(("app_id", app_rec_id.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar itens consumidos.".into()))?;

        for item in consumed_items {
            let item_rec = parse_record_id("inventory_item", &item.item_id);
            db.query(
                "RELATE $app_id->consumes->$item_id SET
                    quantity_planned = $qty,
                    quantity_used    = $used",
            )
            .bind(("app_id", app_rec_id.clone()))
            .bind(("item_id", item_rec))
            .bind(("qty", item.quantity_planned))
            .bind(("used", item.quantity_used))
            .await
            .map_err(|_| ApiError::Database("Falha ao associar itens.".into()))?;
        }
    }

    if let Some(ref equip_list) = data.assigned_equipment {
        db.query("DELETE uses_equipment WHERE in = $app_id")
            .bind(("app_id", app_rec_id.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar equipamentos.".into()))?;

        for equip_id in equip_list {
            let equip_rec = parse_record_id("inventory_item", equip_id);
            db.query("RELATE $app_id->uses_equipment->$equip_id")
                .bind(("app_id", app_rec_id.clone()))
                .bind(("equip_id", equip_rec))
                .await
                .map_err(|_| ApiError::Database("Falha ao associar equipamento.".into()))?;
        }
    }

    Ok(HttpResponse::Ok().json("Agendamento atualizado com sucesso."))
}

/// Exclui o agendamento e remove seus relacionamentos no banco de dados.
#[delete("/appointments/{id}")]
pub async fn delete_appointment(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let app_id = path.into_inner();
    let clinic_rec = clinic_record_id(&query.clinic_id);
    let app_rec = appointment_record_id(&app_id);

    if !check_permission(&db, &auth.id, &clinic_rec, "appointments:delete")
        .await
        .unwrap_or(false)
        && !check_permission(&db, &auth.id, &clinic_rec, "agenda:delete")
            .await
            .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para excluir agendamentos.".into(),
        ));
    }


    db.query("DELETE assigned_to WHERE in = type::record($app_id)")
        .bind(("app_id", app_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao remover vínculos de profissionais.".into()))?;

    db.query("DELETE consumes WHERE in = type::record($app_id)")
        .bind(("app_id", app_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao remover vínculos de estoque.".into()))?;

    db.query("DELETE uses_equipment WHERE in = type::record($app_id)")
        .bind(("app_id", app_rec.clone()))
        .await
        .map_err(|_| ApiError::Database("Falha ao remover vínculos de equipamentos.".into()))?;

    db.query("DELETE type::record($app_id)")
        .bind(("app_id", app_rec))
        .await
        .map_err(|_| ApiError::Database("Falha ao excluir agendamento.".into()))?;

    Ok(HttpResponse::Ok().json("Agendamento excluído com sucesso."))
}
