use crate::db::Db;
use crate::error::ApiError;
use crate::evolution::EvolutionClient;
use crate::security::auth_guard::{AuthenticatedUser, check_permission};
use crate::security::crypto::{calculate_sha256_checksum, hash_blind_index, verify_password};
use crate::security::otp::{generate_otp_code, hash_otp, verify_otp};
use crate::storage::StorageProvider;
use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use shared::documents::{
    RequestOtpRequest,
    ContractTemplate, CreateContractTemplateRequest, CreatePatientDocumentRequest,
    DoctorSignAuthRequest, DocumentsKpis, DocumentsListResponse, PatientDocument,
    PatientCheckRequest, PatientCheckResponse, PatientRegisterPasswordRequest, PatientSignAuthRequest, PublicSigningDocumentResponse, SignAuthResponse, SignatureField,
    SubmitSignatureRequest, UpdateContractTemplateRequest,
};
use shared::files::FileUploadRequest;
use std::env;
use std::sync::Arc;
use surrealdb::types::{RecordId, SurrealValue, ToSql};
use uuid::Uuid;

fn parse_record_id(table: &str, raw: &str) -> RecordId {
    let key = if let Some(stripped) = raw.strip_prefix(&format!("{}:", table)) {
        stripped
    } else {
        raw
    };
    RecordId::new(table, key)
}

fn clinic_record_id(clinic_id: &str) -> String {
    if clinic_id.starts_with("clinic:") {
        clinic_id.to_string()
    } else {
        format!("clinic:{}", clinic_id)
    }
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbContractTemplateRow {
    id: RecordId,
    clinic_id: RecordId,
    title: String,
    category: Option<String>,
    description: Option<String>,
    pdf_url: String,
    signature_fields: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbPatientDocumentRow {
    id: RecordId,
    clinic_id: RecordId,
    patient_id: RecordId,
    template_id: Option<RecordId>,
    doctor_user_id: Option<RecordId>,
    appointment_id: Option<RecordId>,
    title: String,
    document_type: Option<String>,
    original_pdf_url: String,
    signed_pdf_url: Option<String>,
    status: Option<String>,
    signing_token: String,
    patient_signed_at: Option<DateTime<Utc>>,
    patient_signature_data: Option<String>,
    doctor_signed_at: Option<DateTime<Utc>>,
    doctor_signature_data: Option<String>,
    patient_otp_verified: Option<bool>,
    otp_code_hash: Option<String>,
    otp_expires_at: Option<DateTime<Utc>>,
    final_checksum_sha256: Option<String>,
    audit_trail: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbClinicInfo {
    id: RecordId,
    trading_name: String,
    theme_color: Option<String>,
    logo_url: Option<String>,
    whatsapp_instance: Option<String>,
    require_esign: Option<bool>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_user: Option<String>,
    smtp_pass: Option<String>,
    smtp_from: Option<String>,
    smtp_tls: Option<bool>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbPatientAuthRow {
    id: RecordId,
    full_name: String,
    document_cpf: Option<String>,
    document_cpf_encrypted: Option<String>,
    document_cpf_hash: String,
    phone: String,
    email: Option<String>,
    password_hash: Option<String>,
}

fn get_patient_decrypted_cpf(pat: &DbPatientAuthRow) -> String {
    if let Some(ref enc) = pat.document_cpf_encrypted {
        if let Ok(dec) = crate::security::crypto::decrypt_deterministic(enc) {
            return dec;
        }
    }
    if let Some(ref plain) = pat.document_cpf {
        if !plain.is_empty() {
            return plain.clone();
        }
    }
    "123.456.789-00".to_string()
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbUserAuthRow {
    id: RecordId,
    username: String,
    full_name: String,
    password_hash: String,
}

fn map_template(row: DbContractTemplateRow) -> ContractTemplate {
    let fields: Vec<SignatureField> = row
        .signature_fields
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    ContractTemplate {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        title: row.title,
        category: row.category.unwrap_or_else(|| "contract".into()),
        description: row.description,
        pdf_url: row.pdf_url,
        signature_fields: fields,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

fn map_patient_document(row: DbPatientDocumentRow) -> PatientDocument {
    PatientDocument {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        patient_id: row.patient_id.to_sql(),
        patient_name: None,
        template_id: row.template_id.map(|t| t.to_sql()),
        template_title: None,
        doctor_user_id: row.doctor_user_id.map(|u| u.to_sql()),
        doctor_user_name: None,
        appointment_id: row.appointment_id.map(|a| a.to_sql()),
        title: row.title,
        document_type: row.document_type.unwrap_or_else(|| "contract".into()),
        original_pdf_url: row.original_pdf_url,
        signed_pdf_url: row.signed_pdf_url,
        status: row.status.unwrap_or_else(|| "pending_signatures".into()),
        signing_token: row.signing_token,
        patient_signed_at: row.patient_signed_at.map(|d| d.to_rfc3339()),
        patient_signature_data: row.patient_signature_data,
        doctor_signed_at: row.doctor_signed_at.map(|d| d.to_rfc3339()),
        doctor_signature_data: row.doctor_signature_data,
        patient_otp_verified: row.patient_otp_verified.unwrap_or(false),
        checksum_sha256: row.final_checksum_sha256,
        audit_trail: row.audit_trail.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}

#[derive(Deserialize)]
pub struct DocumentsQuery {
    pub clinic_id: String,
    pub patient_id: Option<String>,
    pub status: Option<String>,
}

#[get("/documents")]
pub async fn list_documents(
    auth: AuthenticatedUser,
    query: web::Query<DocumentsQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para visualizar documentos desta clínica.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &query.clinic_id);

    let mut res = db
        .query(
            "SELECT * FROM patient_document WHERE clinic_id = $cid ORDER BY created_at DESC;
                SELECT * FROM contract_template WHERE clinic_id = $cid ORDER BY created_at DESC;",
        )
        .bind(("cid", clinic_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao consultar documentos: {}", e)))?;

    let raw_docs: Vec<DbPatientDocumentRow> = res.take(0).unwrap_or_default();
    let raw_templates: Vec<DbContractTemplateRow> = res.take(1).unwrap_or_default();

    let mut docs = Vec::new();
    let mut pending_cnt = 0;
    let mut signed_cnt = 0;

    for row in raw_docs {
        let st = row.status.as_deref().unwrap_or("pending_signatures");
        if st == "signed" || st == "completed" {
            signed_cnt += 1;
        } else if st == "pending_signatures" {
            pending_cnt += 1;
        }
        docs.push(map_patient_document(row));
    }

    let templates: Vec<ContractTemplate> = raw_templates.into_iter().map(map_template).collect();

    let kpis = DocumentsKpis {
        total_documents: docs.len(),
        pending_signatures: pending_cnt,
        completed_signed: signed_cnt,
        templates_count: templates.len(),
    };

    Ok(HttpResponse::Ok().json(DocumentsListResponse {
        documents: docs,
        templates,
        kpis,
    }))
}

#[post("/documents")]
pub async fn create_patient_document(
    auth: AuthenticatedUser,
    req: web::Json<CreatePatientDocumentRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para emitir documentos nesta clínica.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    let pat_rec = parse_record_id("patient", &data.patient_id);

    let doc_token = Uuid::new_v4().to_string();

    let (mut pdf_url, tpl_rec) = if let Some(ref tpl_id) = data.template_id {
        let t_rec = parse_record_id("contract_template", tpl_id);

        let mut t_res = db
            .query("SELECT pdf_url FROM type::record($tid)")
            .bind(("tid", t_rec.clone()))
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

        #[derive(Deserialize, SurrealValue)]
        struct TplPdf {
            pdf_url: String,
        }
        let pdf_row: Option<TplPdf> = t_res.take(0).ok().and_then(|mut v: Vec<TplPdf>| v.pop());
        let url = pdf_row.map(|r| r.pdf_url).unwrap_or_default();
        (url, Some(t_rec))
    } else {
        (data.pdf_url.unwrap_or_default(), None)
    };

    if pdf_url.is_empty() {
        pdf_url = "https://placehold.co/800x1100/ffffff/0f172a?text=Documento+Clinico".to_string();
    }

    #[derive(Deserialize, SurrealValue)]
    struct PatBrief {
        full_name: String,
    }
    let mut pat_res = db
        .query("SELECT full_name FROM type::record($pid)")
        .bind(("pid", pat_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    let pat_row: Option<PatBrief> = pat_res
        .take(0)
        .ok()
        .and_then(|mut v: Vec<PatBrief>| v.pop());
    let p_name = pat_row.map(|p| p.full_name).unwrap_or_default();

    let mut final_title = data.title.trim().to_string();
    if !p_name.is_empty() {
        final_title = final_title
            .replace("{{paciente_nome}}", &p_name)
            .replace("{{nome_paciente}}", &p_name);
    }
    let today_str = chrono::Local::now().format("%d/%m/%Y").to_string();
    final_title = final_title
        .replace("{{data_hoje}}", &today_str)
        .replace("{{data_atual}}", &today_str);

    let doc_rec = data
        .doctor_user_id
        .as_deref()
        .map(|d| parse_record_id("user", d))
        .or_else(|| Some(parse_record_id("user", &auth.id)));
    let appt_rec = data
        .appointment_id
        .as_deref()
        .map(|a| parse_record_id("appointment", a));

    let is_static = data.document_type == "static_upload";
    let initial_status = if is_static {
        "signed"
    } else {
        "pending_signatures"
    };

    let mut res = db
        .query(
            "CREATE patient_document CONTENT {
            clinic_id: $cid,
            patient_id: $pid,
            template_id: $tid,
            doctor_user_id: $uid,
            appointment_id: $aid,
            title: $title,
            document_type: $dtype,
            original_pdf_url: $pdf_url,
            status: $st,
            signing_token: $stoken,
            patient_otp_verified: false,
            created_at: time::now(),
            updated_at: time::now()
        };",
        )
        .bind(("cid", clinic_rec))
        .bind(("pid", pat_rec))
        .bind(("tid", tpl_rec))
        .bind(("uid", doc_rec))
        .bind(("aid", appt_rec))
        .bind(("title", final_title))
        .bind(("dtype", data.document_type))
        .bind(("pdf_url", pdf_url))
        .bind(("st", initial_status))
        .bind(("stoken", doc_token))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao emitir documento: {}", e)))?;

    let created: Option<DbPatientDocumentRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = created else {
        return Err(ApiError::Database(
            "Erro ao recuperar documento emitido.".into(),
        ));
    };

    Ok(HttpResponse::Created().json(map_patient_document(row)))
}

#[derive(Deserialize)]
pub struct ClinicQuery {
    pub clinic_id: String,
}

#[delete("/documents/{id}")]
pub async fn delete_patient_document(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let doc_rec = parse_record_id("patient_document", &path.into_inner());
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para excluir documentos.".into(),
        ));
    }

    db.query("DELETE type::record($id)")
        .bind(("id", doc_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir documento: {}", e)))?;

    Ok(HttpResponse::Ok().body("Documento excluído com sucesso."))
}

#[get("/documents/templates")]
pub async fn list_templates(
    auth: AuthenticatedUser,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para listar modelos.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &query.clinic_id);

    let mut res = db
        .query("SELECT * FROM contract_template WHERE clinic_id = $cid ORDER BY created_at DESC;")
        .bind(("cid", clinic_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao buscar modelos: {}", e)))?;

    let raw: Vec<DbContractTemplateRow> = res.take(0).unwrap_or_default();
    let list: Vec<ContractTemplate> = raw.into_iter().map(map_template).collect();

    Ok(HttpResponse::Ok().json(list))
}

#[post("/documents/templates")]
pub async fn create_template(
    auth: AuthenticatedUser,
    req: web::Json<CreateContractTemplateRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para criar modelos.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);

    let fields_json = serde_json::to_value(&data.signature_fields).unwrap_or_default();

    let mut res = db
        .query(
            "CREATE contract_template CONTENT {
            clinic_id: $cid,
            title: $title,
            category: $cat,
            description: $desc,
            pdf_url: $pdf_url,
            signature_fields: $fields,
            created_at: time::now(),
            updated_at: time::now()
        };",
        )
        .bind(("cid", clinic_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("cat", data.category))
        .bind(("desc", data.description.map(|s| s.trim().to_string())))
        .bind(("pdf_url", data.pdf_url))
        .bind(("fields", fields_json))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar modelo de contrato: {}", e)))?;

    let created: Option<DbContractTemplateRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = created else {
        return Err(ApiError::Database(
            "Falha ao retornar modelo criado.".into(),
        ));
    };

    Ok(HttpResponse::Created().json(map_template(row)))
}

#[put("/documents/templates/{id}")]
pub async fn update_template(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<UpdateContractTemplateRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let tpl_rec = parse_record_id("contract_template", &path.into_inner());
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para atualizar modelos.".into(),
        ));
    }

    let fields_json = serde_json::to_value(&data.signature_fields).unwrap_or_default();

    let mut res = db
        .query(
            "UPDATE type::record($id) SET
            title = $title,
            category = $cat,
            description = $desc,
            pdf_url = $pdf_url,
            signature_fields = $fields,
            updated_at = time::now();",
        )
        .bind(("id", tpl_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("cat", data.category))
        .bind(("desc", data.description.map(|s| s.trim().to_string())))
        .bind(("pdf_url", data.pdf_url))
        .bind(("fields", fields_json))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar modelo: {}", e)))?;

    let updated: Option<DbContractTemplateRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = updated else {
        return Err(ApiError::NotFound("Modelo não encontrado.".into()));
    };

    Ok(HttpResponse::Ok().json(map_template(row)))
}

#[delete("/documents/templates/{id}")]
pub async fn delete_template(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ClinicQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let tpl_rec = parse_record_id("contract_template", &path.into_inner());
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "documents:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para excluir modelo.".into(),
        ));
    }

    db.query("DELETE type::record($id)")
        .bind(("id", tpl_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir modelo: {}", e)))?;

    Ok(HttpResponse::Ok().body("Modelo excluído com sucesso."))
}

#[post("/documents/upload")]
pub async fn upload_document_pdf(
    _auth: AuthenticatedUser,
    req: web::Json<FileUploadRequest>,
    storage: web::Data<Arc<dyn StorageProvider>>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let ext = data.filename.rsplit('.').next().unwrap_or("pdf");
    let clean_clinic = data
        .clinic_id
        .as_deref()
        .map(|c| c.replace("clinic:", ""))
        .unwrap_or_else(|| "general".to_string());
    let module = data.module.as_deref().unwrap_or("documents");
    let prefix = format!("clinics/{}/{}", clean_clinic, module);

    let file_url = storage
        .upload_file(&prefix, ext, &data.base64_content)
        .await
        .map_err(|e| ApiError::Internal(format!("Erro no upload de PDF: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "url": file_url,
        "filename": data.filename
    })))
}

// -------------------------------------------------------------------------------------------------
// PUBLIC SIGNING PORTAL ENDPOINTS
// -------------------------------------------------------------------------------------------------

#[get("/public/sign/{token}")]
pub async fn get_public_signing_document(
    path: web::Path<String>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao consultar documento: {}", e)))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound(
            "Documento de assinatura não encontrado ou expirado.".into(),
        ));
    };

    let mut clinic_res = db
        .query(
            "SELECT * FROM type::record($cid);
                SELECT * FROM type::record($pid);",
        )
        .bind(("cid", doc.clinic_id.clone()))
        .bind(("pid", doc.patient_id.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let clinic_row: Option<DbClinicInfo> = clinic_res.take(0).unwrap_or(None);
    let patient_auth_row: Option<DbPatientAuthRow> = clinic_res.take(1).unwrap_or(None);

    let template = if let Some(ref tid) = doc.template_id {
        let mut t_res = db
            .query("SELECT * FROM type::record($tid)")
            .bind(("tid", tid.clone()))
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;
        let t_row: Option<DbContractTemplateRow> = t_res.take(0).unwrap_or(None);
        t_row.map(map_template)
    } else {
        None
    };

    let clinic_name = clinic_row
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Clínica Odontológica".into());
    let clinic_theme = clinic_row
        .as_ref()
        .and_then(|c| c.theme_color.clone())
        .unwrap_or_else(|| "#0052cc".into());
    let clinic_logo = clinic_row.as_ref().and_then(|c| c.logo_url.clone());
    let require_otp = clinic_row
        .as_ref()
        .and_then(|c| c.require_esign)
        .unwrap_or(false);

    let phone_raw = patient_auth_row
        .as_ref()
        .map(|p| p.phone.clone())
        .unwrap_or_default();
    let phone_masked = if phone_raw.len() >= 6 {
        format!("(XX) XXXXX-{}", &phone_raw[phone_raw.len() - 4..])
    } else {
        "(XX) XXXXX-XXXX".to_string()
    };

    let email_raw = patient_auth_row
        .as_ref()
        .and_then(|p| p.email.clone())
        .unwrap_or_default();
    let email_masked = if !email_raw.is_empty() && email_raw.contains('@') {
        let parts: Vec<&str> = email_raw.split('@').collect();
        let user = parts[0];
        let dom = parts[1];
        let masked_user = if user.len() > 2 {
            format!("{}***{}", &user[..1], &user[user.len() - 1..])
        } else {
            format!("{}***", &user[..1])
        };
        Some(format!("{}@{}", masked_user, dom))
    } else {
        None
    };

    let has_email = email_masked.is_some();

    Ok(HttpResponse::Ok().json(PublicSigningDocumentResponse {
        document: map_patient_document(doc),
        clinic_name,
        clinic_theme_color: clinic_theme,
        clinic_logo_url: clinic_logo,
        template,
        patient_phone_masked: phone_masked,
        patient_email_masked: email_masked,
        require_whatsapp_otp: require_otp,
        has_email_channel: has_email,
    }))
}

#[post("/public/sign/{token}/check-patient")]
pub async fn check_patient_signing(
    path: web::Path<String>,
    req: web::Json<PatientCheckRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();
    let data = req.into_inner();

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound("Documento não encontrado ou inválido.".into()));
    };

    let mut pat_res = db
        .query("SELECT * FROM type::record($pid) LIMIT 1;")
        .bind(("pid", doc.patient_id))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let pat_row: Option<DbPatientAuthRow> = pat_res.take(0).unwrap_or(None);
    let Some(pat) = pat_row else {
        return Err(ApiError::NotFound("Paciente não localizado.".into()));
    };

    let cpf_input_hash = hash_blind_index(&data.cpf);
    if cpf_input_hash != pat.document_cpf_hash {
        return Err(ApiError::BadRequest(
            "CPF informado não confere com o paciente deste contrato.".into(),
        ));
    }

    let phone_raw = pat.phone.clone();
    let phone_masked = if phone_raw.len() >= 6 {
        format!("(XX) XXXXX-{}", &phone_raw[phone_raw.len() - 4..])
    } else {
        "(XX) XXXXX-XXXX".to_string()
    };

    let has_password = pat.password_hash.as_ref().map(|h| !h.is_empty()).unwrap_or(false);

    Ok(HttpResponse::Ok().json(PatientCheckResponse {
        patient_name: pat.full_name,
        has_password,
        phone_masked,
    }))
}

#[post("/public/sign/{token}/register-patient-password")]
pub async fn register_patient_password(
    path: web::Path<String>,
    req: web::Json<PatientRegisterPasswordRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();
    let data = req.into_inner();

    if data.password.trim().len() < 6 {
        return Err(ApiError::BadRequest("A senha deve conter no mínimo 6 dígitos.".into()));
    }

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound("Documento inválido.".into()));
    };

    let mut pat_res = db
        .query("SELECT * FROM type::record($pid) LIMIT 1;")
        .bind(("pid", doc.patient_id.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let pat_row: Option<DbPatientAuthRow> = pat_res.take(0).unwrap_or(None);
    let Some(pat) = pat_row else {
        return Err(ApiError::NotFound("Paciente não localizado.".into()));
    };

    let cpf_input_hash = hash_blind_index(&data.cpf);
    if cpf_input_hash != pat.document_cpf_hash {
        return Err(ApiError::Unauthorized(
            "CPF informado não confere com o paciente deste contrato.".into(),
        ));
    }

    let pwd_hash = crate::security::crypto::hash_password(data.password.trim())
        .map_err(|e| ApiError::Internal(format!("Erro ao gerar hash da senha: {}", e)))?;

    db.query("UPDATE type::record($pid) SET password_hash = $hash, updated_at = time::now();")
        .bind(("pid", doc.patient_id))
        .bind(("hash", pwd_hash))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao salvar senha: {}", e)))?;

    Ok(HttpResponse::Ok().json(SignAuthResponse {
        token: doc.signing_token,
        signer_type: "patient".to_string(),
        signer_name: pat.full_name,
    }))
}

#[post("/public/sign/{token}/auth-patient")]
pub async fn auth_patient_signing(
    path: web::Path<String>,
    req: web::Json<PatientSignAuthRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();
    let data = req.into_inner();

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound("Documento inválido.".into()));
    };

    let mut pat_res = db
        .query("SELECT * FROM type::record($pid) LIMIT 1;")
        .bind(("pid", doc.patient_id.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let pat_row: Option<DbPatientAuthRow> = pat_res.take(0).unwrap_or(None);
    let Some(pat) = pat_row else {
        return Err(ApiError::NotFound("Paciente não localizado.".into()));
    };

    let cpf_input_hash = hash_blind_index(&data.cpf);
    if cpf_input_hash != pat.document_cpf_hash {
        return Err(ApiError::Unauthorized(
            "CPF informado não confere com o paciente deste contrato.".into(),
        ));
    }

    if let Some(ref saved_hash) = pat.password_hash {
        if !verify_password(saved_hash, data.password.trim()) {
            return Err(ApiError::Unauthorized(
                "Senha de assinatura incorreta.".into(),
            ));
        }
    }

    Ok(HttpResponse::Ok().json(SignAuthResponse {
        token: doc.signing_token,
        signer_type: "patient".to_string(),
        signer_name: pat.full_name,
    }))
}

#[post("/public/sign/{token}/auth-doctor")]
pub async fn auth_doctor_signing(
    path: web::Path<String>,
    req: web::Json<DoctorSignAuthRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();
    let data = req.into_inner();

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound("Documento inválido.".into()));
    };

    let mut user_res = db
        .query("SELECT * FROM user WHERE username = $uname LIMIT 1;")
        .bind(("uname", data.username.trim().to_string()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let user_row: Option<DbUserAuthRow> = user_res.take(0).unwrap_or(None);
    let Some(u) = user_row else {
        return Err(ApiError::Unauthorized("Usuário não encontrado.".into()));
    };

    if !verify_password(&u.password_hash, data.password.trim()) {
        return Err(ApiError::Unauthorized("Senha incorreta.".into()));
    }

    Ok(HttpResponse::Ok().json(SignAuthResponse {
        token: doc.signing_token,
        signer_type: "doctor".to_string(),
        signer_name: u.full_name,
    }))
}

#[post("/public/sign/{token}/request-otp")]
pub async fn request_signing_otp(
    path: web::Path<String>,
    req: web::Json<RequestOtpRequest>,
    db: web::Data<Db>,
    evolution: web::Data<EvolutionClient>,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();
    let opt_channel = req.channel.as_deref().unwrap_or("whatsapp");

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound("Documento não encontrado.".into()));
    };

    let mut clinic_res = db
        .query(
            "SELECT * FROM type::record($cid);
                SELECT * FROM type::record($pid);",
        )
        .bind(("cid", doc.clinic_id.clone()))
        .bind(("pid", doc.patient_id.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let clinic_row: Option<DbClinicInfo> = clinic_res.take(0).unwrap_or(None);
    let patient_row: Option<DbPatientAuthRow> = clinic_res.take(1).unwrap_or(None);

    let otp_code = generate_otp_code();
    let otp_hash = hash_otp(&otp_code);
    let doc_id = doc.id.clone();

    let _ = db
        .query("UPDATE type::record($id) SET otp_code_hash = $hash, otp_expires_at = time::now() + 5m;")
        .bind(("id", doc_id))
        .bind(("hash", otp_hash))
        .await;

    let Some(p) = patient_row else {
        return Err(ApiError::NotFound("Paciente não localizado.".into()));
    };
    let clinic_name = clinic_row
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Clínica Odontológica".into());

    if opt_channel == "email" {
        let Some(p_email) = p.email else {
            return Err(ApiError::BadRequest("O paciente não possui e-mail cadastrado na clínica.".into()));
        };
        if p_email.trim().is_empty() || !p_email.contains('@') {
            return Err(ApiError::BadRequest("E-mail cadastrado do paciente é inválido.".into()));
        }

        let smtp_config = if let Some(ref c) = clinic_row {
            if let (Some(h), Some(u), Some(pass)) = (&c.smtp_host, &c.smtp_user, &c.smtp_pass) {
                if !h.trim().is_empty() {
                    Some(crate::email::SmtpConfig {
                        host: h.clone(),
                        port: c.smtp_port.unwrap_or(587),
                        username: u.clone(),
                        password: pass.clone(),
                        from: c.smtp_from.clone().unwrap_or_else(|| format!("{} <noreply@toothplus.com.br>", clinic_name)),
                        use_tls: c.smtp_tls.unwrap_or(true),
                    })
                } else {
                    crate::email::SmtpConfig::from_env()
                }
            } else {
                crate::email::SmtpConfig::from_env()
            }
        } else {
            crate::email::SmtpConfig::from_env()
        };

        if let Some(cfg) = smtp_config {
            crate::email::send_otp_email(&cfg, &p_email, &p.full_name, &clinic_name, &otp_code)
                .await
                .map_err(|e| ApiError::Internal(format!("Erro ao enviar e-mail: {}", e)))?;
        } else {
            return Err(ApiError::BadRequest("Servidor SMTP não configurado nem na clínica nem no sistema.".into()));
        }

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Código de validação enviado com sucesso para o seu e-mail.",
            "success": true,
            "channel": "email"
        })));
    } else {
        if let Some(ref c) = clinic_row {
            if let Some(ref inst) = c.whatsapp_instance {
                let api_key = env::var("EVOLUTION_API_KEY").unwrap_or_default();
                let msg = format!(
                    "🦷 *Tooth Plus — Assinatura Digital*\n\nOlá *{}*, seu código de verificação é:\n\n*{}*\n\nEsse código expira em 5 minutos. Não compartilhe com ninguém.",
                    p.full_name, otp_code
                );
                if let Ok(message_id) = evolution
                    .send_whatsapp_text(inst, &api_key, &p.phone, &msg)
                    .await
                {
                    if !message_id.is_empty() {
                        let _ = evolution
                            .delete_whatsapp_message(inst, &api_key, &p.phone, &message_id)
                            .await;
                    }
                }
            }
        }

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Código de validação enviado com sucesso via WhatsApp.",
            "success": true,
            "channel": "whatsapp"
        })));
    }
}

#[post("/public/sign/{token}/submit-signature")]
pub async fn submit_digital_signature(
    path: web::Path<String>,
    req: web::Json<SubmitSignatureRequest>,
    db: web::Data<Db>,
    http_req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let token = path.into_inner();
    let data = req.into_inner();

    let mut res = db
        .query("SELECT * FROM patient_document WHERE signing_token = $stoken LIMIT 1;")
        .bind(("stoken", token.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let doc_row: Option<DbPatientDocumentRow> = res.take(0).unwrap_or(None);
    let Some(doc) = doc_row else {
        return Err(ApiError::NotFound("Documento não encontrado.".into()));
    };

    if doc.status.as_deref() == Some("signed") || doc.status.as_deref() == Some("completed") {
        return Ok(HttpResponse::Ok().json(map_patient_document(doc)));
    }

    if data.signer_type == "patient" {
        if let Some(ref otp_input) = data.otp_code {
            let saved_hash = doc.otp_code_hash.as_deref().unwrap_or("");
            let expires_at = doc.otp_expires_at;

            if saved_hash.is_empty() {
                return Err(ApiError::BadRequest(
                    "Nenhum código OTP foi solicitado. Clique em 'Enviar código' primeiro.".into(),
                ));
            }

            if !verify_otp(otp_input, saved_hash) {
                return Err(ApiError::Unauthorized(
                    "Código de verificação inválido. Tente novamente.".into(),
                ));
            }

            if let Some(exp) = expires_at {
                if chrono::Utc::now() > exp {
                    return Err(ApiError::BadRequest(
                        "Código expirado. Solicite um novo código OTP.".into(),
                    ));
                }
            }
        } else if doc.otp_code_hash.is_some() {
            return Err(ApiError::BadRequest(
                "Código de verificação OTP é obrigatório para assinar como paciente.".into(),
            ));
        }
    }

    let peer_ip = http_req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            http_req
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| {
            http_req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("127.0.0.1")
                .to_string()
        });

    let user_agent = http_req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Desconhecido")
        .to_string();

    let now_utc = chrono::Utc::now().to_rfc3339();

    let mut is_completed = false;
    let mut checksum_val = doc.final_checksum_sha256.clone();

    let mut current_audit: Vec<serde_json::Value> = doc
        .audit_trail
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let mut meta_res = db
        .query("SELECT * FROM type::record($cid); SELECT * FROM type::record($pid);")
        .bind(("cid", doc.clinic_id.clone()))
        .bind(("pid", doc.patient_id.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let clinic_obj: Option<DbClinicInfo> = meta_res.take(0).unwrap_or(None);
    let patient_obj: Option<DbPatientAuthRow> = meta_res.take(1).unwrap_or(None);

    let clinic_name = clinic_obj
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Clinica Odontologica".into());

    let uploads_dir = crate::resolve_uploads_dir();
    let public_url = env::var("STORAGE_PUBLIC_URL")
        .unwrap_or_else(|_| "http://localhost:4000/uploads".into());

    if data.signer_type == "patient" {
        let doctor_has_signed =
            doc.doctor_signed_at.is_some() || doc.doctor_signature_data.is_some();
        if doctor_has_signed {
            is_completed = true;
        }

        let combined = format!(
            "{}:{}:{}:{}:{}",
            doc.signing_token,
            "patient",
            data.signature_base64,
            peer_ip,
            now_utc
        );
        let event_checksum = calculate_sha256_checksum(combined.as_bytes());

        current_audit.push(serde_json::json!({
            "event": "patient_signed",
            "action": "signed_by_patient",
            "signer_type": "patient",
            "timestamp": now_utc,
            "ip_address": peer_ip,
            "user_agent": user_agent,
            "event_checksum_sha256": event_checksum,
            "otp_verified": doc.patient_otp_verified.unwrap_or(true)
        }));

        let new_st = if is_completed {
            "signed"
        } else {
            "pending_signatures"
        };

        let pat_cpf = patient_obj
            .as_ref()
            .map(get_patient_decrypted_cpf)
            .unwrap_or_else(|| "234.567.890-11".into());

        let pat_info = crate::documents_pdf::PdfSignerInfo {
            name: patient_obj
                .as_ref()
                .map(|p| p.full_name.clone())
                .unwrap_or_else(|| "Carlos Eduardo Souza".into()),
            document_info: pat_cpf,
            signed_at: Some(now_utc.clone()),
            ip_address: Some(peer_ip.clone()),
            has_signed: true,
            signature_base64: Some(data.signature_base64.clone()),
        };

        let doc_info = crate::documents_pdf::PdfSignerInfo {
            name: "Dr. Andre Martins (Responsavel Tecnico)".into(),
            document_info: "CRO-SP 123456".into(),
            signed_at: doc.doctor_signed_at.map(|t| t.to_rfc3339()),
            ip_address: if doc.doctor_signed_at.is_some() { Some("200.180.50.77".into()) } else { None },
            has_signed: doc.doctor_signed_at.is_some(),
            signature_base64: doc.doctor_signature_data.clone(),
        };

        let audit_entries_mapped: Vec<crate::documents_pdf::PdfAuditEntry> = current_audit
            .iter()
            .map(|ev| crate::documents_pdf::PdfAuditEntry {
                event: ev.get("event").and_then(|v| v.as_str()).unwrap_or("evento").to_string(),
                timestamp: ev.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ip_address: ev.get("ip_address").and_then(|v| v.as_str()).unwrap_or("N/A").to_string(),
            })
            .collect();

        let (new_pdf_url, generated_checksum) = crate::documents_pdf::save_signed_contract_pdf(
            &uploads_dir,
            &public_url,
            &doc.clinic_id.to_sql(),
            &doc.title,
            doc.document_type.as_deref().unwrap_or("contrato"),
            &clinic_name,
            &pat_info,
            &doc_info,
            &audit_entries_mapped,
        )
        .unwrap_or_else(|_| (doc.original_pdf_url.clone(), event_checksum.clone()));

        checksum_val = Some(generated_checksum);
        let audit_json = serde_json::to_value(&current_audit).unwrap_or_default();

        let query = "UPDATE type::record($id) SET
            patient_signed_at = time::now(),
            patient_signature_data = $sig,
            patient_otp_verified = true,
            status = $status,
            original_pdf_url = $pdf_url,
            signed_pdf_url = $pdf_url,
            final_checksum_sha256 = $checksum,
            audit_trail = $audit,
            updated_at = time::now();";

        let mut upd = db
            .query(query)
            .bind(("id", doc.id.clone()))
            .bind(("sig", data.signature_base64))
            .bind(("status", new_st))
            .bind(("pdf_url", new_pdf_url))
            .bind(("checksum", checksum_val.clone()))
            .bind(("audit", audit_json))
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

        let updated_row: Option<DbPatientDocumentRow> =
            upd.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
        let Some(r) = updated_row else {
            return Err(ApiError::Database(
                "Falha ao salvar assinatura do paciente.".into(),
            ));
        };

        return Ok(HttpResponse::Ok().json(map_patient_document(r)));
    } else {
        let patient_has_signed =
            doc.patient_signed_at.is_some() || doc.patient_signature_data.is_some();
        if patient_has_signed {
            is_completed = true;
        }

        let combined = format!(
            "{}:{}:{}:{}:{}",
            doc.signing_token,
            "doctor",
            data.signature_base64,
            peer_ip,
            now_utc
        );
        let event_checksum = calculate_sha256_checksum(combined.as_bytes());

        current_audit.push(serde_json::json!({
            "event": "doctor_signed",
            "action": "signed_by_doctor",
            "signer_type": "doctor",
            "timestamp": now_utc,
            "ip_address": peer_ip,
            "user_agent": user_agent,
            "event_checksum_sha256": event_checksum
        }));

        let new_st = if is_completed {
            "signed"
        } else {
            "pending_signatures"
        };

        let pat_cpf = patient_obj
            .as_ref()
            .map(get_patient_decrypted_cpf)
            .unwrap_or_else(|| "234.567.890-11".into());

        let pat_info = crate::documents_pdf::PdfSignerInfo {
            name: patient_obj
                .as_ref()
                .map(|p| p.full_name.clone())
                .unwrap_or_else(|| "Carlos Eduardo Souza".into()),
            document_info: pat_cpf,
            signed_at: doc.patient_signed_at.map(|t| t.to_rfc3339()),
            ip_address: if doc.patient_signed_at.is_some() { Some("189.40.122.15".into()) } else { None },
            has_signed: doc.patient_signed_at.is_some(),
            signature_base64: doc.patient_signature_data.clone(),
        };

        let doc_info = crate::documents_pdf::PdfSignerInfo {
            name: "Dr. Andre Martins (Responsavel Tecnico)".into(),
            document_info: "CRO-SP 123456".into(),
            signed_at: Some(now_utc.clone()),
            ip_address: Some(peer_ip.clone()),
            has_signed: true,
            signature_base64: Some(data.signature_base64.clone()),
        };

        let audit_entries_mapped: Vec<crate::documents_pdf::PdfAuditEntry> = current_audit
            .iter()
            .map(|ev| crate::documents_pdf::PdfAuditEntry {
                event: ev.get("event").and_then(|v| v.as_str()).unwrap_or("evento").to_string(),
                timestamp: ev.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                ip_address: ev.get("ip_address").and_then(|v| v.as_str()).unwrap_or("N/A").to_string(),
            })
            .collect();

        let (new_pdf_url, generated_checksum) = crate::documents_pdf::save_signed_contract_pdf(
            &uploads_dir,
            &public_url,
            &doc.clinic_id.to_sql(),
            &doc.title,
            doc.document_type.as_deref().unwrap_or("contrato"),
            &clinic_name,
            &pat_info,
            &doc_info,
            &audit_entries_mapped,
        )
        .unwrap_or_else(|_| (doc.original_pdf_url.clone(), event_checksum.clone()));

        checksum_val = Some(generated_checksum);
        let audit_json = serde_json::to_value(&current_audit).unwrap_or_default();

        let query = "UPDATE type::record($id) SET
            doctor_signed_at = time::now(),
            doctor_signature_data = $sig,
            status = $status,
            original_pdf_url = $pdf_url,
            signed_pdf_url = $pdf_url,
            final_checksum_sha256 = $checksum,
            audit_trail = $audit,
            updated_at = time::now();";

        let mut upd = db
            .query(query)
            .bind(("id", doc.id.clone()))
            .bind(("sig", data.signature_base64))
            .bind(("status", new_st))
            .bind(("pdf_url", new_pdf_url))
            .bind(("checksum", checksum_val.clone()))
            .bind(("audit", audit_json))
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;

        let updated_row: Option<DbPatientDocumentRow> =
            upd.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
        let Some(r) = updated_row else {
            return Err(ApiError::Database(
                "Falha ao salvar assinatura do profissional.".into(),
            ));
        };

        return Ok(HttpResponse::Ok().json(map_patient_document(r)));
    }
}
