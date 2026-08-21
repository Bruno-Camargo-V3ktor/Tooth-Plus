//! # Emissão e Gerenciamento de Documentos de Pacientes (Backend)
//!
//! Controla a emissão de contratos e termos odontológicos vinculados a pacientes,
//! substituição de variáveis dinâmicas e listagem consolidada de documentos.

use super::{
    clinic_record_id, get_patient_decrypted_cpf, map_patient_document, map_template,
    parse_record_id, DbClinicInfo, DbContractTemplateRow, DbPatientAuthRow,
    DbPatientDocumentRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{delete, get, post, web, HttpResponse};
use serde::Deserialize;
use shared::documents::{
    ContractTemplate, CreatePatientDocumentRequest, DocumentsKpis, DocumentsListResponse,
};
use surrealdb::types::SurrealValue;
use uuid::Uuid;

/// Query string para listagem e filtragem de documentos clínicos.
#[derive(Deserialize)]
pub struct DocumentsQuery {
    pub clinic_id: String,
    pub patient_id: Option<String>,
    pub status: Option<String>,
}

/// Query simples com ID da clínica.
#[derive(Deserialize)]
pub struct ClinicQuery {
    pub clinic_id: String,
}

/// Lista documentos emitidos e modelos para a clínica informada com indicadores agregados (KPIs).
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

/// Emite um novo documento/contrato vinculado ao paciente com token único para assinatura.
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

    let mut clinic_res = db
        .query("SELECT * FROM type::record($cid)")
        .bind(("cid", clinic_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    let clinic_row: Option<DbClinicInfo> = clinic_res.take(0).unwrap_or(None);
    let clinic_name = clinic_row
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Smile Plus Dental Clinic".into());

    let mut pat_res = db
        .query("SELECT * FROM type::record($pid)")
        .bind(("pid", pat_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    let pat_row: Option<DbPatientAuthRow> = pat_res.take(0).unwrap_or(None);
    let p_name = pat_row
        .as_ref()
        .map(|p| p.full_name.clone())
        .unwrap_or_else(|| "Paciente".into());
    let p_cpf = pat_row
        .as_ref()
        .map(get_patient_decrypted_cpf)
        .unwrap_or_else(|| "000.000.000-00".into());

    let doc_rec = data
        .doctor_user_id
        .as_deref()
        .map(|d| parse_record_id("user", d))
        .or_else(|| Some(parse_record_id("user", &auth.id)));

    let mut doc_name = "Dr(a). Cirurgião-Dentista".to_string();
    let doc_cro = "CRO-SP 123456".to_string();
    if let Some(ref uid) = doc_rec {
        if let Ok(mut u_res) = db
            .query("SELECT full_name FROM type::record($uid)")
            .bind(("uid", uid.clone()))
            .await
        {
            #[derive(Deserialize, SurrealValue)]
            struct UserBrief {
                full_name: String,
            }
            if let Some(u) = u_res.take(0).ok().and_then(|mut v: Vec<UserBrief>| v.pop()) {
                doc_name = u.full_name;
            }
        }
    }

    let mut final_title = data.title.trim().to_string();
    if !p_name.is_empty() {
        final_title = final_title
            .replace("{{paciente_nome}}", &p_name)
            .replace("{{nome_paciente}}", &p_name);
    }
    let today_str = chrono::Local::now().format("%d/%m/%Y").to_string();
    final_title = final_title
        .replace("{{data_hoje}}", &today_str)
        .replace("{{data_atual}}", &today_str)
        .replace("{{clinica_nome}}", &clinic_name)
        .replace("{{dentista_nome}}", &doc_name)
        .replace("{{doutor_nome}}", &doc_name);

    let is_upload_mode = data.is_already_signed.unwrap_or(false)
        || data.document_type == "static_upload";

    let (pdf_url, signed_pdf_url, initial_status, pat_signed_at, doc_signed_at) = if is_upload_mode {
        let u = data
            .signed_pdf_url
            .or(data.pdf_url)
            .unwrap_or_else(|| "https://placehold.co/800x1100/ffffff/0f172a?text=Documento+Assinado".into());
        (u.clone(), Some(u), "signed", Some(chrono::Utc::now()), Some(chrono::Utc::now()))
    } else {
        let uploads_dir = crate::resolve_uploads_dir();
        let public_url = std::env::var("STORAGE_PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:4000/uploads".into());

        let pat_info = crate::documents_pdf::PdfSignerInfo {
            name: p_name.clone(),
            document_info: format!("CPF: {}", p_cpf),
            signed_at: None,
            ip_address: None,
            has_signed: false,
            signature_base64: None,
        };
        let doc_info = crate::documents_pdf::PdfSignerInfo {
            name: doc_name.clone(),
            document_info: doc_cro.clone(),
            signed_at: None,
            ip_address: None,
            has_signed: false,
            signature_base64: None,
        };

        let audit_entries = vec![crate::documents_pdf::PdfAuditEntry {
            event: "Documento emitido para assinatura eletronica".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            ip_address: "127.0.0.1".into(),
        }];

        let generated = crate::documents_pdf::save_signed_contract_pdf(
            &uploads_dir,
            &public_url,
            &data.clinic_id,
            &final_title,
            &data.document_type,
            &clinic_name,
            &pat_info,
            &doc_info,
            &audit_entries,
        );

        let u = match generated {
            Ok((url, _)) => url,
            Err(_) => data.pdf_url.unwrap_or_else(|| "http://localhost:4000/uploads/sample_placeholder_template.pdf".into()),
        };
        (u, None, "pending_signatures", None, None)
    };

    let appt_rec = data
        .appointment_id
        .as_deref()
        .map(|a| parse_record_id("appointment", a));

    let tpl_rec = data
        .template_id
        .as_deref()
        .map(|t| parse_record_id("contract_template", t));

    let req_pat = data.requires_patient_signature.unwrap_or(true);
    let req_doc = data.requires_doctor_signature.unwrap_or(false);
    let allow_any_doc = data.allow_any_dentist_signature.unwrap_or(true);

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
            signed_pdf_url: $signed_pdf_url,
            status: $st,
            signing_token: $stoken,
            requires_patient_signature: $req_pat,
            requires_doctor_signature: $req_doc,
            allow_any_dentist_signature: $allow_any_doc,
            patient_signed_at: $ps_at,
            doctor_signed_at: $ds_at,
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
        .bind(("signed_pdf_url", signed_pdf_url))
        .bind(("st", initial_status))
        .bind(("stoken", doc_token))
        .bind(("req_pat", req_pat))
        .bind(("req_doc", req_doc))
        .bind(("allow_any_doc", allow_any_doc))
        .bind(("ps_at", pat_signed_at))
        .bind(("ds_at", doc_signed_at))
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

/// Exclui um documento clínico emitido.
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
