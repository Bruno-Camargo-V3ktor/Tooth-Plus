use crate::api::mock_db::DB;
use shared::documents::{
    ContractTemplate, CreateContractTemplateRequest, CreatePatientDocumentRequest, PatientDocument,
};

pub struct DocumentsApi;

impl DocumentsApi {
    pub async fn list_documents(_clinic_id: &str) -> Result<Vec<PatientDocument>, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;
        Ok(db.patient_documents.clone())
    }

    pub async fn list_templates(_clinic_id: &str) -> Result<Vec<ContractTemplate>, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;
        Ok(db.contract_templates.clone())
    }

    pub async fn create_document(req: CreatePatientDocumentRequest) -> Result<PatientDocument, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let p_name = db
            .patients
            .iter()
            .find(|p| p.id == req.patient_id)
            .map(|p| p.full_name.clone());

        let t_title = req.template_id.as_ref().and_then(|tid| {
            db.contract_templates
                .iter()
                .find(|t| &t.id == tid)
                .map(|t| t.title.clone())
        });

        let doc = PatientDocument {
            id: format!("doc:{}", db.patient_documents.len() + 1),
            clinic_id: req.clinic_id,
            patient_id: req.patient_id,
            patient_name: p_name,
            template_id: req.template_id,
            template_title: t_title,
            doctor_user_id: req.doctor_user_id,
            doctor_user_name: Some("Dr. Lucas Mendes - CRO 12345".to_string()),
            appointment_id: req.appointment_id,
            title: req.title,
            document_type: req.document_type,
            original_pdf_url: "/docs/documento_modelo.pdf".to_string(),
            signed_pdf_url: None,
            status: if req.is_already_signed == Some(true) { "signed".to_string() } else { "pending".to_string() },
            signing_token: format!("tok_{:x}", js_sys::Math::random() as u64),
            requires_patient_signature: req.requires_patient_signature.unwrap_or(true),
            requires_doctor_signature: req.requires_doctor_signature.unwrap_or(false),
            allow_any_dentist_signature: true,
            patient_signed_at: if req.is_already_signed == Some(true) { Some(chrono::Utc::now().to_rfc3339()) } else { None },
            patient_signature_data: None,
            doctor_signed_at: None,
            doctor_signature_data: None,
            patient_otp_verified: false,
            checksum_sha256: None,
            audit_trail: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        db.patient_documents.push(doc.clone());
        Ok(doc)
    }

    pub async fn create_template(req: CreateContractTemplateRequest) -> Result<ContractTemplate, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let tpl = ContractTemplate {
            id: format!("tpl:{}", db.contract_templates.len() + 1),
            clinic_id: req.clinic_id,
            title: req.title,
            category: req.category,
            description: req.description,
            pdf_url: req.pdf_url,
            signature_fields: req.signature_fields,
            requires_patient_signature: req.requires_patient_signature.unwrap_or(true),
            requires_doctor_signature: req.requires_doctor_signature.unwrap_or(false),
            allow_any_dentist_signature: req.allow_any_dentist_signature.unwrap_or(true),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        db.contract_templates.push(tpl.clone());
        Ok(tpl)
    }

    pub async fn delete_document(id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;
        db.patient_documents.retain(|d| d.id != id);
        Ok(())
    }
}

use shared::documents::{
    DoctorSignAuthRequest, PatientSignAuthRequest, PublicSigningDocumentResponse,
    RequestOtpRequest, SignAuthResponse, SubmitSignatureRequest,
};

impl DocumentsApi {
    pub async fn get_public_document(token: &str) -> Result<PublicSigningDocumentResponse, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let doc = db
            .patient_documents
            .iter()
            .find(|d| d.signing_token == token || d.id == token)
            .cloned()
            .ok_or_else(|| "Documento não encontrado ou link expirado.".to_string())?;

        let tpl = doc.template_id.as_ref().and_then(|tid| {
            db.contract_templates.iter().find(|t| &t.id == tid).cloned()
        });

        Ok(PublicSigningDocumentResponse {
            document: doc,
            clinic_name: "SmilePlus Odontologia".to_string(),
            clinic_theme_color: "#00a0e4".to_string(),
            clinic_logo_url: None,
            template: tpl,
            anamnesis: None,
            patient_phone_masked: "(11) 9****-1234".to_string(),
            patient_email_masked: Some("m***@gmail.com".to_string()),
            doctor_phone_masked: Some("(11) 9****-5678".to_string()),
            doctor_email_masked: Some("dr***@smileplus.com.br".to_string()),
            require_whatsapp_otp: true,
            has_email_channel: true,
            requires_patient_signature: true,
            requires_doctor_signature: false,
            allow_any_dentist_signature: true,
        })
    }

    pub async fn authenticate_patient(_token: &str, req: PatientSignAuthRequest) -> Result<SignAuthResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        if req.cpf.trim().is_empty() {
            return Err("Informe seu CPF para continuar.".to_string());
        }
        Ok(SignAuthResponse {
            token: "auth_token_sample".to_string(),
            signer_type: "patient".to_string(),
            signer_name: "Maria Barbosa dos Santos".to_string(),
        })
    }

    pub async fn authenticate_doctor(_token: &str, req: DoctorSignAuthRequest) -> Result<SignAuthResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        if req.username.trim().is_empty() {
            return Err("Informe o usuário do dentista.".to_string());
        }
        Ok(SignAuthResponse {
            token: "auth_doctor_sample".to_string(),
            signer_type: "doctor".to_string(),
            signer_name: "Dr. Lucas Mendes - CRO 12345".to_string(),
        })
    }

    pub async fn request_otp(_token: &str, _req: RequestOtpRequest) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        Ok(())
    }

    pub async fn submit_signature(token: &str, req: SubmitSignatureRequest) -> Result<String, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let doc = db
            .patient_documents
            .iter_mut()
            .find(|d| d.signing_token == token || d.id == token)
            .ok_or_else(|| "Documento não encontrado.".to_string())?;

        doc.status = "signed".to_string();
        doc.patient_signed_at = Some(chrono::Utc::now().to_rfc3339());
        doc.patient_signature_data = Some(req.signature_base64);
        doc.patient_otp_verified = true;
        let checksum = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
        doc.checksum_sha256 = Some(checksum.clone());

        Ok(checksum)
    }
}
