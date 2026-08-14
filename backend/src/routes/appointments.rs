use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use actix_web::{HttpResponse, delete, get, patch, post, put, web};
use serde::Deserialize;
use shared::appointments::{
    AgendaResourceOption, AgendaResourcesResponse, AppointmentResponse, AppointmentStatus,
    AppointmentType, AssignedUserDto, ConsumedItemDto, CreateAppointmentRequest,
    UpdateAppointmentRequest, UpdateAppointmentStatusRequest,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

#[derive(Deserialize)]
pub struct AppointmentQuery {
    clinic_id: String,
    date: Option<String>,
    user_id: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct ClinicQuery {
    clinic_id: String,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbAppointmentRecord {
    id: RecordId,
    clinic_id: RecordId,
    patient_id: Option<RecordId>,
    patient_name: Option<String>,
    title: String,
    scheduled_for: chrono::DateTime<chrono::Utc>,
    duration_minutes: i32,
    status: String,
    appointment_type: String,
    financial_amount_cents: Option<i64>,
    financial_type: Option<String>,
    cancellation_reason: Option<String>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbAssignedRecord {
    user_id: RecordId,
    user_name: Option<String>,
    role_in_appointment: String,
    split_percentage: i32,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbConsumedRecord {
    item_id: RecordId,
    item_name: Option<String>,
    quantity_planned: i32,
    quantity_used: Option<i32>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbResourceRecord {
    id: RecordId,
    name: String,
    extra_info: Option<String>,
}

fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
}

fn clinic_record_id(id: &str) -> String {
    if id.starts_with("clinic:") {
        id.to_string()
    } else {
        format!("clinic:{}", id)
    }
}

fn user_record_id(id: &str) -> String {
    if id.starts_with("user:") {
        id.to_string()
    } else {
        format!("user:{}", id)
    }
}

fn appointment_record_id(id: &str) -> String {
    if id.starts_with("appointment:") {
        id.to_string()
    } else {
        format!("appointment:{}", id)
    }
}

fn patient_record_id(id: &str) -> String {
    if id.starts_with("patient:") {
        id.to_string()
    } else {
        format!("patient:{}", id)
    }
}

fn inventory_record_id(id: &str) -> String {
    if id.starts_with("inventory_item:") {
        id.to_string()
    } else {
        format!("inventory_item:{}", id)
    }
}

fn parse_status(s: &str) -> AppointmentStatus {
    match s {
        "confirmed" => AppointmentStatus::Confirmed,
        "in_progress" => AppointmentStatus::InProgress,
        "completed" => AppointmentStatus::Completed,
        "canceled" => AppointmentStatus::Canceled,
        "no_show" => AppointmentStatus::NoShow,
        _ => AppointmentStatus::Pending,
    }
}

fn parse_type(s: &str) -> AppointmentType {
    match s {
        "treatment" => AppointmentType::Treatment,
        "surgery" => AppointmentType::Surgery,
        "return" => AppointmentType::Return,
        "meeting" => AppointmentType::Meeting,
        "other" => AppointmentType::Other,
        _ => AppointmentType::Consultation,
    }
}

fn status_to_str(s: &AppointmentStatus) -> &'static str {
    match s {
        AppointmentStatus::Pending => "pending",
        AppointmentStatus::Confirmed => "confirmed",
        AppointmentStatus::InProgress => "in_progress",
        AppointmentStatus::Completed => "completed",
        AppointmentStatus::Canceled => "canceled",
        AppointmentStatus::NoShow => "no_show",
    }
}

fn type_to_str(t: &AppointmentType) -> &'static str {
    match t {
        AppointmentType::Consultation => "consultation",
        AppointmentType::Treatment => "treatment",
        AppointmentType::Surgery => "surgery",
        AppointmentType::Return => "return",
        AppointmentType::Meeting => "meeting",
        AppointmentType::Other => "other",
    }
}

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

        results.push(AppointmentResponse {
            id: app.id.key.to_sql(),
            clinic_id: app.clinic_id.to_sql(),
            patient_id: app.patient_id.map(|p| p.to_sql()),
            patient_name: app.patient_name,
            title: app.title,
            scheduled_for: app.scheduled_for.to_rfc3339(),
            duration_minutes: app.duration_minutes,
            status: parse_status(&app.status),
            appointment_type: parse_type(&app.appointment_type),
            financial_amount_cents: app.financial_amount_cents,
            financial_type: app.financial_type,
            cancellation_reason: app.cancellation_reason,
            assigned_users: db_assigned
                .into_iter()
                .map(|a| AssignedUserDto {
                    user_id: a.user_id.to_sql(),
                    user_name: a.user_name,
                    role_in_appointment: a.role_in_appointment,
                    split_percentage: a.split_percentage,
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
        });
    }

    Ok(HttpResponse::Ok().json(results))
}

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
            WHERE clinic_id = type::record($clinic_id)",
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

    Ok(HttpResponse::Ok().json(AgendaResourcesResponse {
        team_members,
        patients,
        inventory_items,
    }))
}

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
                financial_type         = $financial_type
            RETURN id",
        )
        .bind(("clinic_id", clinic_rec))
        .bind(("patient_id", patient_rec))
        .bind(("patient_name", data.patient_name))
        .bind(("title", data.title))
        .bind(("scheduled_for", parsed_dt))
        .bind(("duration_minutes", data.duration_minutes))
        .bind(("appointment_type", type_to_str(&data.appointment_type)))
        .bind(("financial_amount", data.financial_amount_cents))
        .bind(("financial_type", data.financial_type))
        .await
        .map_err(|_| ApiError::Database("Falha ao criar agendamento.".into()))?;

    #[derive(Deserialize, SurrealValue)]
    struct CreatedId {
        id: RecordId,
    }

    let created: Option<CreatedId> = create_resp.take(0).unwrap_or(None);
    let new_app_id = created
        .ok_or_else(|| ApiError::Database("Agendamento não retornou ID.".into()))?
        .id;

    for u in &data.assigned_users {
        let u_rec = parse_record_id("user", &u.user_id);
        db.query(
            "RELATE $app_id->assigned_to->$user_id SET
                role_in_appointment = $role,
                split_percentage    = $split",
        )
        .bind(("app_id", new_app_id.clone()))
        .bind(("user_id", u_rec))
        .bind(("role", u.role_in_appointment.clone()))
        .bind(("split", u.split_percentage))
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

    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": new_app_id.key.to_sql(),
        "message": "Agendamento criado com sucesso."
    })))
}

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

    if !patch.is_empty() {
        db.query("UPDATE type::record($app_id) MERGE $patch")
            .bind(("app_id", app_rec.clone()))
            .bind(("patch", serde_json::Value::Object(patch)))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar dados do agendamento.".into()))?;
    }

    if let Some(ref assigned_users) = data.assigned_users {
        let app_rec_id = parse_record_id("appointment", &app_id);
        db.query("DELETE assigned_to WHERE in = $app_id")
            .bind(("app_id", app_rec_id.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao atualizar membros.".into()))?;

        for u in assigned_users {
            let u_rec = parse_record_id("user", &u.user_id);
            db.query(
                "RELATE $app_id->assigned_to->$user_id SET
                    role_in_appointment = $role,
                    split_percentage    = $split",
            )
            .bind(("app_id", app_rec_id.clone()))
            .bind(("user_id", u_rec))
            .bind(("role", u.role_in_appointment.clone()))
            .bind(("split", u.split_percentage))
            .await
            .map_err(|_| ApiError::Database("Falha ao vincular membros.".into()))?;
        }
    }

    if let Some(ref consumed_items) = data.consumed_items {
        let app_rec_id = parse_record_id("appointment", &app_id);
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

    Ok(HttpResponse::Ok().json("Agendamento atualizado com sucesso."))
}

#[patch("/appointments/{id}/status")]
pub async fn update_appointment_status(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    req: web::Json<UpdateAppointmentStatusRequest>,
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
        return Err(ApiError::Forbidden(
            "Sem privilégios para alterar status de agendamentos.".into(),
        ));
    }

    let status_str = status_to_str(&data.status);

    db.query(
        "UPDATE type::record($app_id) SET
            status              = $status,
            cancellation_reason = $reason",
    )
    .bind(("app_id", app_rec.clone()))
    .bind(("status", status_str))
    .bind(("reason", data.cancellation_reason.clone()))
    .await
    .map_err(|_| ApiError::Database("Falha ao atualizar status do agendamento.".into()))?;

    if data.status == AppointmentStatus::Completed {
        if let Some(ref items) = data.consumed_items {
            for item in items {
                let item_rec = inventory_record_id(&item.item_id);
                let qty_to_deduct = item.quantity_used.unwrap_or(item.quantity_planned);

                db.query(
                    "UPDATE consumes SET quantity_used = $qty
                    WHERE in = type::record($app_id) AND out = type::record($item_id)",
                )
                .bind(("app_id", app_rec.clone()))
                .bind(("item_id", item_rec.clone()))
                .bind(("qty", qty_to_deduct))
                .await
                .ok();

                if qty_to_deduct > 0 {
                    db.query(
                        "UPDATE type::record($item_id) SET current_stock = current_stock - $qty",
                    )
                    .bind(("item_id", item_rec.clone()))
                    .bind(("qty", qty_to_deduct))
                    .await
                    .ok();

                    db.query(
                        "CREATE stock_movement SET
                            item_id         = type::record($item_id),
                            quantity_change = -$qty,
                            movement_type   = 'appointment_consumed'",
                    )
                    .bind(("item_id", item_rec))
                    .bind(("qty", qty_to_deduct))
                    .await
                    .ok();
                }
            }
        }

        #[derive(Deserialize, SurrealValue)]
        struct AppFinanceInfo {
            financial_amount_cents: Option<i64>,
            financial_type: Option<String>,
            title: String,
        }

        let mut fin_resp = db
            .query(
                "SELECT financial_amount_cents, financial_type, title FROM type::record($app_id)",
            )
            .bind(("app_id", app_rec.clone()))
            .await
            .map_err(|_| ApiError::Database("Falha ao verificar dados financeiros.".into()))?;

        let fin_info: Option<AppFinanceInfo> = fin_resp.take(0).unwrap_or(None);

        if let Some(fi) = fin_info {
            if let (Some(amount), Some(ft)) = (fi.financial_amount_cents, fi.financial_type) {
                if amount > 0 && ft == "income" {
                    db.query(
                        "CREATE financial_transaction SET
                            clinic_id      = type::record($clinic_id),
                            appointment_id = type::record($app_id),
                            amount_cents   = $amount,
                            direction      = 'income',
                            status         = 'completed',
                            category       = 'appointment_revenue',
                            description    = $desc",
                    )
                    .bind(("clinic_id", clinic_rec.clone()))
                    .bind(("app_id", app_rec.clone()))
                    .bind(("amount", amount))
                    .bind(("desc", format!("Receita de Atendimento: {}", fi.title)))
                    .await
                    .ok();

                    let mut assigned_resp = db
                        .query(
                            "SELECT
                                out              AS user_id,
                                role_in_appointment,
                                split_percentage
                            FROM assigned_to
                            WHERE in = type::record($app_id) AND split_percentage > 0",
                        )
                        .bind(("app_id", app_rec.clone()))
                        .await
                        .unwrap();

                    let assigned: Vec<DbAssignedRecord> = assigned_resp.take(0).unwrap_or_default();

                    for a in assigned {
                        let commission_cents = (amount * a.split_percentage as i64) / 100;
                        if commission_cents > 0 {
                            db.query(
                                "CREATE financial_transaction SET
                                    clinic_id      = type::record($clinic_id),
                                    appointment_id = type::record($app_id),
                                    user_id        = type::record($user_id),
                                    amount_cents   = $amount,
                                    direction      = 'expense',
                                    status         = 'pending',
                                    category       = 'commission',
                                    description    = $desc",
                            )
                            .bind(("clinic_id", clinic_rec.clone()))
                            .bind(("app_id", app_rec.clone()))
                            .bind(("user_id", a.user_id.to_sql()))
                            .bind(("amount", commission_cents))
                            .bind((
                                "desc",
                                format!("Comissão ({}%): {}", a.split_percentage, fi.title),
                            ))
                            .await
                            .ok();
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json("Status atualizado com sucesso."))
}

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
        && !check_permission(&db, &auth.id, &clinic_rec, "appointments:write")
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

    db.query("DELETE type::record($app_id)")
        .bind(("app_id", app_rec))
        .await
        .map_err(|_| ApiError::Database("Falha ao excluir agendamento.".into()))?;

    Ok(HttpResponse::Ok().json("Agendamento excluído com sucesso."))
}
