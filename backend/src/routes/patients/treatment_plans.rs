//! # Planos de Tratamento / Orçamentos (Backend)
//!
//! Controla a criação, consulta, atualização, aprovação e baixa de pagamentos
//! de orçamentos de tratamento por paciente, com conversão automática de itens
//! em procedimentos clínicos no prontuário e lançamento no módulo financeiro.

use super::{clinic_record_id, parse_record_id};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::finance::{RegisterPaymentRequest, TransactionPaymentEntry};
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
    paid_amount_cents: i64,
    description: String,
    category: String,
    status: String,
    due_date: DateTime<Utc>,
    paid_date: Option<DateTime<Utc>>,
    payment_method: Option<String>,
    payments: Vec<serde_json::Value>,
    installment_current: i32,
    installment_total: i32,
}

#[derive(Deserialize, SurrealValue)]
struct TxMinimal {
    id: RecordId,
}

#[derive(Deserialize, Serialize, Debug, SurrealValue)]
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
    pub paid_amount_cents: Option<i64>,
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
                        if let Some(s) = v.as_str() { Some(s.to_string()) }
                        else if let Some(obj2) = v.as_object() {
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

    let paid = row.paid_amount_cents.unwrap_or(0);
    let remaining = (total - paid).max(0);

    let fin_status = if paid >= total && total > 0 {
        "paid".to_string()
    } else if paid > 0 {
        "partial".to_string()
    } else {
        "unpaid".to_string()
    };

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
        paid_amount_cents: paid,
        remaining_amount_cents: remaining,
        financial_status: Some(fin_status),
        payments: vec![],
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

/// Sincroniza um plano aprovado: cria a pendência financeira e adiciona os procedimentos ao prontuário.
async fn sync_approved_plan(
    db: &Db,
    clinic_id: &str,
    patient_id: &str,
    plan_rec: &RecordId,
    plan_title: &str,
    total_cents: i64,
    dentist_rec: &Option<RecordId>,
    items: &[TreatmentPlanItem],
) {
    let clinic_rec = parse_record_id("clinic", clinic_id);
    let pat_rec = parse_record_id("patient", patient_id);

    // 1. Cria ou garante a existência da transação financeira como pendência
    let mut tx_res = db
        .query("SELECT * FROM transaction WHERE treatment_plan_id = type::record($pid) LIMIT 1;")
        .bind(("pid", plan_rec.clone()))
        .await;

    let existing_tx: Option<serde_json::Value> = tx_res
        .as_mut()
        .ok()
        .and_then(|r| r.take::<Vec<serde_json::Value>>(0).ok())
        .and_then(|mut v| v.pop());

    if let Some(tx_obj) = existing_tx {
        if let Some(tx_id_str) = tx_obj.get("id").and_then(|v| v.as_str()) {
            let tx_rec = parse_record_id("transaction", tx_id_str);
            let _ = db
                .query("UPDATE type::record($pid) SET transaction_id = $tid, updated_at = time::now();")
                .bind(("pid", plan_rec.clone()))
                .bind(("tid", tx_rec))
                .await;
        }
    } else if total_cents > 0 {
        let tx_data = InsertPlanTransactionDb {
            clinic_id: clinic_rec.clone(),
            appointment_id: None,
            patient_id: Some(pat_rec.clone()),
            user_id: dentist_rec.clone(),
            treatment_plan_id: Some(plan_rec.clone()),
            direction: "income".to_string(),
            amount_cents: total_cents,
            paid_amount_cents: 0,
            description: format!("Orçamento: {}", plan_title),
            category: "Tratamento Odontológico".to_string(),
            status: "pending".to_string(),
            due_date: Utc::now(),
            paid_date: None,
            payment_method: None,
            payments: vec![],
            installment_current: 1,
            installment_total: 1,
        };

        if let Ok(Some(tx)) = db.create::<Option<TxMinimal>>("transaction").content(tx_data).await {
            let _ = db
                .query("UPDATE type::record($pid) SET transaction_id = $tid, updated_at = time::now();")
                .bind(("pid", plan_rec.clone()))
                .bind(("tid", tx.id))
                .await;
        }
    }

    // 2. Adiciona individualmente cada tratamento do orçamento aprovado como Procedimento no Prontuário
    for item in items {
        let mut exist_proc_res = db
            .query(
                "SELECT * FROM patient_treatment
                WHERE treatment_plan_id = type::record($pid)
                AND treatment_plan_item_id = $item_id LIMIT 1;",
            )
            .bind(("pid", plan_rec.clone()))
            .bind(("item_id", item.id.clone()))
            .await;

        let has_existing = exist_proc_res
            .as_mut()
            .ok()
            .and_then(|r| r.take::<Vec<serde_json::Value>>(0).ok())
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        if !has_existing {
            let _ = db
                .query(
                    "CREATE patient_treatment CONTENT {
                    patient_id: $pat,
                    clinic_id: $cid,
                    dentist_user_id: $uid,
                    treatment_plan_id: $pid,
                    treatment_plan_item_id: $item_id,
                    procedure_name: $pname,
                    procedure_category: $pcat,
                    tooth_number: $tooth,
                    surfaces: $surfaces,
                    cost_cents: $cost,
                    clinical_notes: $notes,
                    status: 'pending',
                    created_at: time::now()
                };",
                )
                .bind(("pat", pat_rec.clone()))
                .bind(("cid", clinic_rec.clone()))
                .bind(("uid", dentist_rec.clone()))
                .bind(("pid", plan_rec.clone()))
                .bind(("item_id", item.id.clone()))
                .bind(("pname", item.procedure_name.clone()))
                .bind(("pcat", item.category.clone()))
                .bind(("tooth", item.tooth_number.clone()))
                .bind(("surfaces", item.surfaces.clone()))
                .bind(("cost", item.price_cents))
                .bind(("notes", item.clinical_notes.clone()))
                .await;
        }
    }
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
            "SELECT * FROM patient_treatment_plan
            WHERE patient_id = type::record($pid)
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

/// Cria um novo plano de tratamento. Se `approve_immediately = true`, gera transação e procedimentos.
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
            paid_amount_cents: 0,
            notes: $notes,
            planned_start_date: $start_date,
            planned_end_date: $end_date,
            created_at: time::now(),
            updated_at: time::now()
        };",
        )
        .bind(("pid", pat_rec.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("uid", dentist_rec.clone()))
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
    let Some(plan_row) = created else {
        return Err(ApiError::Database("Erro ao salvar plano no banco de dados.".into()));
    };

    let mapped = map_plan(plan_row, None);

    if data.approve_immediately {
        let plan_rec = parse_record_id("patient_treatment_plan", &mapped.id);
        sync_approved_plan(
            &db,
            &data.clinic_id,
            &patient_id,
            &plan_rec,
            &mapped.title,
            total_cents,
            &Some(dentist_rec),
            &mapped.items,
        )
        .await;
    }

    Ok(HttpResponse::Created().json(mapped))
}

/// Atualiza um plano de tratamento.
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

/// Atualiza o status de um plano. Se Aprovado, gera transação e procedimentos no prontuário.
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

    let mapped = map_plan(row, None);

    if data.status == TreatmentPlanStatus::Approved {
        sync_approved_plan(
            &db,
            &data.clinic_id,
            &patient_id,
            &plan_rec,
            &mapped.title,
            mapped.total_price_cents,
            &mapped.dentist_user_id.as_ref().map(|d| parse_record_id("user", d)),
            &mapped.items,
        )
        .await;
    }

    // Re-fetch
    let mut res2 = db
        .query("SELECT * FROM type::record($pid);")
        .bind(("pid", RecordId::new("patient_treatment_plan", plan_key)))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let final_row: Option<DbTreatmentPlanRow> =
        res2.take(0).map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(map_plan(
        final_row.unwrap_or_else(|| {
            DbTreatmentPlanRow {
                id: plan_rec,
                patient_id: parse_record_id("patient", &patient_id),
                clinic_id: parse_record_id("clinic", &data.clinic_id),
                dentist_user_id: None,
                transaction_id: None,
                title: mapped.title,
                status: Some(new_status.into()),
                items: None,
                total_price_cents: Some(mapped.total_price_cents),
                paid_amount_cents: Some(mapped.paid_amount_cents),
                notes: mapped.notes,
                planned_start_date: mapped.planned_start_date,
                planned_end_date: mapped.planned_end_date,
                created_at: None,
                updated_at: None,
            }
        }),
        None,
    )))
}

/// Registra pagamento (parcial ou total) diretamente pelo Orçamento.
#[post("/patients/{id}/treatment-plans/{plan_id}/pay")]
pub async fn pay_treatment_plan(
    auth: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<RegisterPaymentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let (patient_id, plan_id) = path.into_inner();
    let data = body.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "finance:write")
        .await
        .unwrap_or(false)
        && !check_permission(&db, &auth.id, &clinic_str, "treatments:write")
            .await
            .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para registrar pagamentos.".into(),
        ));
    }

    let method = data.payment_method.trim().to_string();
    if method.is_empty() {
        return Err(ApiError::BadRequest(
            "O método de pagamento é obrigatório.".into(),
        ));
    }

    if data.amount_cents <= 0 {
        return Err(ApiError::BadRequest(
            "O valor do pagamento deve ser maior que zero.".into(),
        ));
    }

    let plan_key = plan_id.trim_start_matches("patient_treatment_plan:");
    let plan_rec = RecordId::new("patient_treatment_plan", plan_key);

    let mut p_res = db
        .query("SELECT * FROM type::record($pid);")
        .bind(("pid", plan_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let p_row: Option<DbTreatmentPlanRow> =
        p_res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(plan) = p_row else {
        return Err(ApiError::NotFound("Plano de tratamento não encontrado.".into()));
    };

    let total = plan.total_price_cents.unwrap_or(0);
    let prev_paid = plan.paid_amount_cents.unwrap_or(0);
    let new_paid = prev_paid + data.amount_cents;

    let payment_entry = TransactionPaymentEntry {
        id: uuid::Uuid::new_v4().to_string(),
        paid_at: data.paid_date.unwrap_or_else(|| Utc::now().to_rfc3339()),
        amount_cents: data.amount_cents,
        payment_method: method.clone(),
        notes: data.notes.clone(),
        registered_by_user_id: Some(auth.id.clone()),
        registered_by_user_name: None,
    };
    let payment_json = serde_json::to_value(&payment_entry).unwrap_or_default();

    let fin_status = if new_paid >= total && total > 0 {
        "paid"
    } else {
        "partial"
    };

    if let Some(ref tx_rec) = plan.transaction_id {
        let _ = db
            .query(
                "UPDATE type::record($tid) SET
                paid_amount_cents = $paid,
                status = $status,
                payment_method = $method,
                paid_date = time::now(),
                payments = array::concat(IF payments != NONE THEN payments ELSE [] END, [$entry]),
                updated_at = time::now();",
            )
            .bind(("tid", tx_rec.clone()))
            .bind(("paid", new_paid))
            .bind(("status", fin_status.to_string()))
            .bind(("method", method.clone()))
            .bind(("entry", payment_json.clone()))
            .await;
    } else {
        let clinic_rec = parse_record_id("clinic", &data.clinic_id);
        let pat_rec = parse_record_id("patient", &patient_id);
        let tx_data = InsertPlanTransactionDb {
            clinic_id: clinic_rec,
            appointment_id: None,
            patient_id: Some(pat_rec),
            user_id: plan.dentist_user_id.clone(),
            treatment_plan_id: Some(plan_rec.clone()),
            direction: "income".to_string(),
            amount_cents: total,
            paid_amount_cents: new_paid,
            description: format!("Orçamento: {}", plan.title),
            category: "Tratamento Odontológico".to_string(),
            status: fin_status.to_string(),
            due_date: Utc::now(),
            paid_date: Some(Utc::now()),
            payment_method: Some(method),
            payments: vec![payment_json],
            installment_current: 1,
            installment_total: 1,
        };

        if let Ok(Some(tx)) = db.create::<Option<TxMinimal>>("transaction").content(tx_data).await {
            let _ = db
                .query("UPDATE type::record($pid) SET transaction_id = $tid, updated_at = time::now();")
                .bind(("pid", plan_rec.clone()))
                .bind(("tid", tx.id))
                .await;
        }
    }

    let _ = db
        .query(
            "UPDATE type::record($pid) SET
            paid_amount_cents = $paid,
            financial_status = $fin_status,
            status = IF status == 'draft' THEN 'approved' ELSE status END,
            updated_at = time::now();
            UPDATE patient_treatment SET
            financial_status = $fin_status,
            updated_at = time::now()
            WHERE treatment_plan_id = type::record($pid);",
        )
        .bind(("pid", plan_rec.clone()))
        .bind(("paid", new_paid))
        .bind(("fin_status", fin_status.to_string()))
        .await;

    // Sincroniza procedimentos no prontuário caso o plano tenha virado aprovado
    let mapped_plan = map_plan(plan, None);
    sync_approved_plan(
        &db,
        &data.clinic_id,
        &patient_id,
        &plan_rec,
        &mapped_plan.title,
        mapped_plan.total_price_cents,
        &mapped_plan.dentist_user_id.as_ref().map(|d| parse_record_id("user", d)),
        &mapped_plan.items,
    )
    .await;

    let mut refetch = db
        .query("SELECT * FROM type::record($pid);")
        .bind(("pid", plan_rec))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let updated_plan: Option<DbTreatmentPlanRow> =
        refetch.take(0).map_err(|e| ApiError::Database(e.to_string()))?;

    let mut final_mapped = map_plan(updated_plan.unwrap_or(DbTreatmentPlanRow {
        id: parse_record_id("patient_treatment_plan", &plan_id),
        patient_id: parse_record_id("patient", &patient_id),
        clinic_id: parse_record_id("clinic", &data.clinic_id),
        dentist_user_id: None,
        transaction_id: None,
        title: mapped_plan.title,
        status: Some("approved".into()),
        items: None,
        total_price_cents: Some(total),
        paid_amount_cents: Some(new_paid),
        notes: None,
        planned_start_date: None,
        planned_end_date: None,
        created_at: None,
        updated_at: None,
    }), None);

    final_mapped.payments.push(payment_entry);
    Ok(HttpResponse::Ok().json(final_mapped))
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
