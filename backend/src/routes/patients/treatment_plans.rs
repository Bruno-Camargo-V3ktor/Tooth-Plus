//! # Planos de Tratamento / Orçamentos (Backend)
//!
//! Controla a criação, consulta, atualização e exclusão de orçamentos de tratamento
//! por paciente, com sincronização automática com o módulo financeiro ao aprovar.

use super::{clinic_record_id, parse_record_id};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::treatments::{
    CreateTreatmentPlanRequest, PatientTreatmentPlan, TreatmentItemStatus, TreatmentPlanItem,
    TreatmentPlanStatus, UpdateTreatmentPlanRequest, UpdateTreatmentPlanStatusRequest,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

#[derive(Serialize, SurrealValue)]
struct InsertPlanTransactionDb {
    clinic_id: RecordId,
    appointment_id: Option<RecordId>,
    patient_id: Option<RecordId>,
    user_id: Option<RecordId>,
    treatment_plan_id: Option<RecordId>,
    direction: String,
    amount_cents: i64,
    description: String,
    category: String,
    status: String,
    due_date: DateTime<Utc>,
    paid_date: Option<DateTime<Utc>>,
    payment_method: Option<String>,
    installment_current: i32,
    installment_total: i32,
}

#[derive(Deserialize, SurrealValue)]
struct TxMinimal {
    id: RecordId,
}

#[derive(Deserialize, Debug, SurrealValue)]
pub(crate) struct DbTreatmentPlanRow {
    pub id: RecordId,
    pub patient_id: RecordId,
    pub clinic_id: RecordId,
    pub dentist_user_id: Option<RecordId>,
    pub transaction_id: Option<RecordId>,
    pub title: String,
    pub status: Option<String>,
    pub items: Option<serde_json::Value>,
    pub total_price_cents: Option<i64>,
    pub notes: Option<String>,
    pub planned_start_date: Option<String>,
    pub planned_end_date: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

fn parse_plan_status(s: &str) -> TreatmentPlanStatus {
    match s {
        "approved" => TreatmentPlanStatus::Approved,
        "in_progress" => TreatmentPlanStatus::InProgress,
        "completed" => TreatmentPlanStatus::Completed,
        "canceled" => TreatmentPlanStatus::Canceled,
        _ => TreatmentPlanStatus::Draft,
    }
}

fn plan_status_str(s: TreatmentPlanStatus) -> &'static str {
    match s {
        TreatmentPlanStatus::Draft => "draft",
        TreatmentPlanStatus::Approved => "approved",
        TreatmentPlanStatus::InProgress => "in_progress",
        TreatmentPlanStatus::Completed => "completed",
        TreatmentPlanStatus::Canceled => "canceled",
    }
}

fn parse_item_status(s: &str) -> TreatmentItemStatus {
    match s {
        "in_progress" => TreatmentItemStatus::InProgress,
        "done" => TreatmentItemStatus::Done,
        "canceled" => TreatmentItemStatus::Canceled,
        _ => TreatmentItemStatus::Pending,
    }
}

pub(crate) fn map_plan(row: DbTreatmentPlanRow, dentist_name: Option<String>) -> PatientTreatmentPlan {
    let items: Vec<TreatmentPlanItem> = row
        .items
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let obj = item.as_object()?;
                    let id = obj.get("id").and_then(|v| {
                        // SurrealDB can serialize RecordId as object or string
                        if let Some(s) = v.as_str() { Some(s.to_string()) }
                        else if let Some(obj2) = v.as_object() {
                            // {"tb":"...","id":{"String":"..."}} pattern
                            let tb = obj2.get("tb").and_then(|x| x.as_str()).unwrap_or("item");
                            let key = obj2.get("id")
                                .and_then(|x| x.as_object())
                                .and_then(|x| x.get("String"))
                                .and_then(|x| x.as_str())
                                .unwrap_or("");
                            Some(format!("{}:{}", tb, key))
                        }
                        else { None }
                    }).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                    let status_str = obj.get("status").and_then(|v| v.as_str()).unwrap_or("pending");
                    Some(TreatmentPlanItem {
                        id,
                        template_id: obj.get("template_id").and_then(|v| v.as_str()).map(String::from),
                        procedure_name: obj.get("procedure_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        category: obj.get("category").and_then(|v| v.as_str()).map(String::from),
                        tooth_number: obj.get("tooth_number").and_then(|v| v.as_str()).map(String::from),
                        dental_region: obj.get("dental_region").and_then(|v| v.as_str()).map(String::from),
                        surfaces: obj.get("surfaces").and_then(|v| v.as_array())
                            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                        price_cents: obj.get("price_cents").and_then(|v| v.as_i64()).unwrap_or(0),
                        status: parse_item_status(status_str),
                        appointment_id: obj.get("appointment_id").and_then(|v| v.as_str()).map(String::from),
                        clinical_notes: obj.get("clinical_notes").and_then(|v| v.as_str()).map(String::from),
                        sort_order: obj.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let total = row
        .total_price_cents
        .unwrap_or_else(|| items.iter().map(|i| i.price_cents).sum());

    PatientTreatmentPlan {
        id: row.id.to_sql(),
        patient_id: row.patient_id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        dentist_user_id: row.dentist_user_id.as_ref().map(|u| u.to_sql()),
        dentist_user_name: dentist_name,
        transaction_id: row.transaction_id.map(|t| t.to_sql()),
        title: row.title,
        status: parse_plan_status(row.status.as_deref().unwrap_or("draft")),
        items,
        total_price_cents: total,
        notes: row.notes,
        planned_start_date: row.planned_start_date,
        planned_end_date: row.planned_end_date,
        created_at: row
            .created_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        updated_at: row
            .updated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    }
}

#[derive(Deserialize)]
pub struct PlanClinicQuery {
    pub clinic_id: String,
}

/// Lista todos os planos de tratamento de um paciente.
#[get("/patients/{id}/treatment-plans")]
pub async fn list_patient_plans(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<PlanClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let patient_id = path.into_inner();
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para visualizar planos de tratamento.".into(),
        ));
    }

    let pat_rec = parse_record_id("patient", &patient_id);

    let mut res = db
        .query(
            "SELECT *, (SELECT name FROM type::record(dentist_user_id))[0].name AS dentist_name
             FROM patient_treatment_plan
             WHERE patient_id = $pid
             ORDER BY created_at DESC;",
        )
        .bind(("pid", pat_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao listar planos: {}", e)))?;

    let rows: Vec<DbTreatmentPlanRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;

    let plans: Vec<PatientTreatmentPlan> = rows.into_iter().map(|r| map_plan(r, None)).collect();
    Ok(HttpResponse::Ok().json(plans))
}

/// Cria um novo plano de tratamento. Se `approve_immediately = true`, gera transação financeira.
#[post("/patients/{id}/treatment-plans")]
pub async fn create_treatment_plan(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<CreateTreatmentPlanRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let patient_id = path.into_inner();
    let data = body.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para criar planos de tratamento.".into(),
        ));
    }

    let pat_rec = parse_record_id("patient", &patient_id);
    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    let dentist_rec = data
        .dentist_user_id
        .as_deref()
        .map(|d| parse_record_id("user", d))
        .unwrap_or_else(|| parse_record_id("user", &auth.id));

    let total_cents: i64 = data.items.iter().map(|i| i.price_cents).sum();

    let items_json = serde_json::to_value(
        data.items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                serde_json::json!({
                    "id": uuid::Uuid::new_v4().to_string(),
                    "template_id": item.template_id,
                    "procedure_name": item.procedure_name.trim(),
                    "category": item.category,
                    "tooth_number": item.tooth_number,
                    "dental_region": item.dental_region,
                    "surfaces": item.surfaces,
                    "price_cents": item.price_cents,
                    "status": "pending",
                    "appointment_id": serde_json::Value::Null,
                    "clinical_notes": item.clinical_notes,
                    "sort_order": item.sort_order.unwrap_or(idx as i32),
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or(serde_json::Value::Array(vec![]));

    let initial_status = if data.approve_immediately {
        "approved"
    } else {
        "draft"
    };

    let mut res = db
        .query(
            "CREATE patient_treatment_plan CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            dentist_user_id: $uid,
            title: $title,
            status: $status,
            items: $items,
            total_price_cents: $total,
            notes: $notes,
            planned_start_date: $start_date,
            planned_end_date: $end_date,
            created_at: time::now(),
            updated_at: time::now()
        };",
        )
        .bind(("pid", pat_rec.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("uid", dentist_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("status", initial_status.to_string()))
        .bind(("items", items_json))
        .bind(("total", total_cents))
        .bind(("notes", data.notes.clone()))
        .bind(("start_date", data.planned_start_date.clone()))
        .bind(("end_date", data.planned_end_date.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar plano: {}", e)))?;

    let created: Option<DbTreatmentPlanRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(mut plan_row) = created else {
        return Err(ApiError::Database("Erro ao salvar plano.".into()));
    };

    // Se aprovado imediatamente, criar transação financeira
    if data.approve_immediately && total_cents > 0 {
        // Fetch patient name for transaction description
        let mut patient_res = db
            .query("SELECT full_name FROM type::record($pid);")
            .bind(("pid", pat_rec.clone()))
            .await
            .ok();

        let patient_name: Option<String> = patient_res
            .as_mut()
            .and_then(|r| {
                r.take::<Vec<serde_json::Value>>(0).ok()
            })
            .and_then(|rows| rows.into_iter().next())
            .and_then(|row| row.get("full_name").and_then(|v| v.as_str()).map(String::from));

        let patient_name_str = patient_name.unwrap_or_else(|| "Paciente".to_string());
        let description = format!("Orçamento: {}", data.title.trim());

        let plan_id_str = plan_row.id.to_sql();
        let plan_rec = {
            let key = plan_id_str.trim_start_matches("patient_treatment_plan:");
            RecordId::new("patient_treatment_plan", key)
        };

        let tx_data = InsertPlanTransactionDb {
            clinic_id: clinic_rec,
            appointment_id: None,
            patient_id: Some(pat_rec),
            user_id: None,
            treatment_plan_id: Some(plan_rec.clone()),
            direction: "income".to_string(),
            amount_cents: total_cents,
            description,
            category: "Tratamento".to_string(),
            status: "pending".to_string(),
            due_date: Utc::now(),
            paid_date: None,
            payment_method: None,
            installment_current: 1,
            installment_total: 1,
        };

        if let Ok(Some(tx)) = db.create::<Option<TxMinimal>>("transaction").content(tx_data).await {
            let _ = db
                .query("UPDATE type::record($pid) SET transaction_id = $tid, updated_at = time::now();")
                .bind(("pid", plan_rec))
                .bind(("tid", tx.id.clone()))
                .await;
            plan_row.transaction_id = Some(tx.id);
        }
    }

    Ok(HttpResponse::Created().json(map_plan(plan_row, None)))
}

/// Atualiza um plano de tratamento existente (rascunho).
#[put("/patients/{id}/treatment-plans/{plan_id}")]
pub async fn update_treatment_plan(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateTreatmentPlanRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (_patient_id, plan_id) = path.into_inner();
    let data = body.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para editar planos de tratamento.".into(),
        ));
    }

    let plan_key = plan_id.trim_start_matches("patient_treatment_plan:");
    let plan_rec = RecordId::new("patient_treatment_plan", plan_key);
    let total_cents: i64 = data.items.iter().map(|i| i.price_cents).sum();

    let items_json = serde_json::to_value(
        data.items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                serde_json::json!({
                    "id": uuid::Uuid::new_v4().to_string(),
                    "template_id": item.template_id,
                    "procedure_name": item.procedure_name.trim(),
                    "category": item.category,
                    "tooth_number": item.tooth_number,
                    "dental_region": item.dental_region,
                    "surfaces": item.surfaces,
                    "price_cents": item.price_cents,
                    "status": "pending",
                    "appointment_id": serde_json::Value::Null,
                    "clinical_notes": item.clinical_notes,
                    "sort_order": item.sort_order.unwrap_or(idx as i32),
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or(serde_json::Value::Array(vec![]));

    let dentist_rec = data
        .dentist_user_id
        .as_deref()
        .map(|d| parse_record_id("user", d));

    let mut res = db
        .query(
            "UPDATE type::record($pid) SET
            title = $title,
            dentist_user_id = IF $uid != NONE THEN $uid ELSE dentist_user_id END,
            items = $items,
            total_price_cents = $total,
            notes = $notes,
            planned_start_date = $start_date,
            planned_end_date = $end_date,
            updated_at = time::now();",
        )
        .bind(("pid", plan_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("uid", dentist_rec))
        .bind(("items", items_json))
        .bind(("total", total_cents))
        .bind(("notes", data.notes))
        .bind(("start_date", data.planned_start_date))
        .bind(("end_date", data.planned_end_date))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar plano: {}", e)))?;

    let updated: Option<DbTreatmentPlanRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = updated else {
        return Err(ApiError::NotFound("Plano de tratamento não encontrado.".into()));
    };

    Ok(HttpResponse::Ok().json(map_plan(row, None)))
}

/// Atualiza apenas o status de um plano. Se aprovado, sincroniza o valor com a transação financeira.
#[patch("/patients/{id}/treatment-plans/{plan_id}/status")]
pub async fn update_treatment_plan_status(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateTreatmentPlanStatusRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (patient_id, plan_id) = path.into_inner();
    let data = body.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para alterar status do plano.".into(),
        ));
    }

    let plan_key = plan_id.trim_start_matches("patient_treatment_plan:");
    let plan_rec = RecordId::new("patient_treatment_plan", plan_key);
    let new_status = plan_status_str(data.status);

    let mut res = db
        .query(
            "UPDATE type::record($pid) SET
            status = $status,
            updated_at = time::now();",
        )
        .bind(("pid", plan_rec.clone()))
        .bind(("status", new_status.to_string()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar status: {}", e)))?;

    let updated: Option<DbTreatmentPlanRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = updated else {
        return Err(ApiError::NotFound("Plano não encontrado.".into()));
    };

    // Se aprovado e há transação vinculada, atualizar valor da transação
    if data.status == TreatmentPlanStatus::Approved {
        if let Some(ref tx_rec) = row.transaction_id {
            let _ = db
                .query("UPDATE type::record($tid) SET amount_cents = $amount, updated_at = time::now();")
                .bind(("tid", tx_rec.clone()))
                .bind(("amount", row.total_price_cents.unwrap_or(0)))
                .await;
        } else {
            // Gerar transação se não existir ainda
            let clinic_rec = parse_record_id("clinic", &data.clinic_id);
            let pat_rec = parse_record_id("patient", &patient_id);
            let total = row.total_price_cents.unwrap_or(0);
            let title = row.title.clone();

            let tx_data = InsertPlanTransactionDb {
                clinic_id: clinic_rec,
                appointment_id: None,
                patient_id: Some(pat_rec),
                user_id: None,
                treatment_plan_id: Some(plan_rec.clone()),
                direction: "income".to_string(),
                amount_cents: total,
                description: format!("Orçamento: {}", title),
                category: "Tratamento".to_string(),
                status: "pending".to_string(),
                due_date: Utc::now(),
                paid_date: None,
                payment_method: None,
                installment_current: 1,
                installment_total: 1,
            };

            if let Ok(Some(tx)) = db.create::<Option<TxMinimal>>("transaction").content(tx_data).await {
                let _ = db
                    .query("UPDATE type::record($pid) SET transaction_id = $tid, updated_at = time::now();")
                    .bind(("pid", plan_rec))
                    .bind(("tid", tx.id))
                    .await;
            }
        }
    }

    // Re-fetch with updated transaction_id
    let mut res2 = db
        .query("SELECT * FROM type::record($pid);")
        .bind(("pid", RecordId::new("patient_treatment_plan", plan_key)))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let final_row: Option<DbTreatmentPlanRow> =
        res2.take(0).map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(map_plan(
        final_row.unwrap_or(row),
        None,
    )))
}

/// Exclui um plano de tratamento (somente rascunhos).
#[delete("/patients/{id}/treatment-plans/{plan_id}")]
pub async fn delete_treatment_plan(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    query: web::Query<PlanClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (_patient_id, plan_id) = path.into_inner();
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para excluir planos de tratamento.".into(),
        ));
    }

    let plan_key = plan_id.trim_start_matches("patient_treatment_plan:");
    let plan_rec = RecordId::new("patient_treatment_plan", plan_key);

    db.query("DELETE type::record($pid);")
        .bind(("pid", plan_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir plano: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Plano de tratamento excluído com sucesso."
    })))
}
