//! # Operações Cadastrais de Pacientes (CRUD)
//!
//! Contém os endpoints de listagem com pesquisa, cadastro com proteção determinística
//! de CPF, leitura de prontuário integrado, atualização e exclusão de pacientes.

use super::{
    clinic_record_id, map_patient, parse_record_id, DbAnamnesisRow, DbDocumentRow,
    DbExamRow, DbPatientRow, DbTreatmentRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use crate::security::crypto::{
    encrypt_deterministic, hash_blind_index, hash_password,
};
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::{Datelike, Local};
use serde::Deserialize;
use shared::documents::PatientDocument;
use shared::patients::{
    CreatePatientRequest, PatientAnamnesis, PatientDetailsResponse, PatientExam,
    PatientKpis, PatientListResponse, PatientTreatment, UpdatePatientRequest,
};
use surrealdb::types::{SurrealValue, ToSql};

/// Query string para listagem de pacientes.
#[derive(Deserialize)]
pub struct PatientQuery {
    pub clinic_id: String,
    pub search: Option<String>,
}

/// Query simples com ID da clínica.
#[derive(Deserialize)]
pub struct PatientPathQuery {
    pub clinic_id: String,
}

/// Lista os pacientes da clínica com suporte a busca textual e KPIs superiores.
#[get("/patients")]
pub async fn list_patients(
    auth: AuthenticatedUser,
    query: web::Query<PatientQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "patients:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para visualizar pacientes desta clínica.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &query.clinic_id);

    let mut res = db
        .query(
            "SELECT * FROM patient WHERE clinic_id = $cid ORDER BY created_at DESC;
             SELECT count() FROM patient_document WHERE clinic_id = $cid AND status = 'pending_signatures' GROUP ALL;
             SELECT count() FROM patient_treatment WHERE clinic_id = $cid AND status = 'in_progress' GROUP ALL;",
        )
        .bind(("cid", clinic_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao consultar pacientes: {}", e)))?;

    let raw_patients: Vec<DbPatientRow> = res.take(0).unwrap_or_default();

    #[derive(Deserialize, SurrealValue)]
    struct CountRow {
        count: usize,
    }
    let pending_docs_row: Option<CountRow> =
        res.take(1).ok().and_then(|mut v: Vec<CountRow>| v.pop());
    let active_treat_row: Option<CountRow> =
        res.take(2).ok().and_then(|mut v: Vec<CountRow>| v.pop());

    let now_month = Local::now().month();
    let now_year = Local::now().year();

    let mut new_this_month = 0;
    let mut mapped_patients = Vec::new();

    let search_term = query.search.as_deref().unwrap_or("").trim().to_lowercase();

    for row in raw_patients {
        if let Some(created) = row.created_at {
            if created.month() == now_month && created.year() == now_year {
                new_this_month += 1;
            }
        }

        let p = map_patient(row);

        if search_term.is_empty() {
            mapped_patients.push(p);
        } else {
            let matches_name = p.full_name.to_lowercase().contains(&search_term);
            let matches_phone = p.phone.contains(&search_term);
            let matches_cpf = p.document_cpf.as_deref().unwrap_or("").contains(&search_term);
            let matches_rg = p.document_rg.as_deref().unwrap_or("").contains(&search_term);
            let matches_email = p
                .email
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&search_term);

            if matches_name || matches_phone || matches_cpf || matches_rg || matches_email {
                mapped_patients.push(p);
            }
        }
    }

    let kpis = PatientKpis {
        total_patients: mapped_patients.len(),
        new_this_month,
        pending_documents_count: pending_docs_row.map(|r| r.count).unwrap_or(0),
        active_treatments_count: active_treat_row.map(|r| r.count).unwrap_or(0),
    };

    let total = mapped_patients.len();

    Ok(HttpResponse::Ok().json(PatientListResponse {
        items: mapped_patients,
        kpis,
        total,
    }))
}

/// Cadastra um novo paciente aplicando criptografia AES-256 no CPF e blind index.
#[post("/patients")]
pub async fn create_patient(
    auth: AuthenticatedUser,
    req: web::Json<CreatePatientRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "patients:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para cadastrar pacientes nesta clínica.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);

    let cpf_clean = data.document_cpf.as_deref().unwrap_or("").trim();
    let has_cpf = !cpf_clean.is_empty();
    let has_rg = data
        .document_rg
        .as_ref()
        .map(|rg| !rg.trim().is_empty())
        .unwrap_or(false);

    if !has_cpf && !has_rg {
        return Err(ApiError::BadRequest(
            "O paciente deve possuir obrigatoriamente CPF ou RG cadastrado.".into(),
        ));
    }

    let cpf_encrypted = if has_cpf {
        Some(encrypt_deterministic(cpf_clean)
            .map_err(|e| ApiError::Internal(format!("Falha na proteção de dados do CPF: {}", e)))?)
    } else {
        None
    };
    let cpf_hash = if has_cpf {
        Some(hash_blind_index(cpf_clean))
    } else {
        None
    };

    let guardians_list = data.legal_guardians.unwrap_or_default();
    let encrypted_guardians: Vec<shared::patients::PatientGuardian> = guardians_list
        .into_iter()
        .map(|mut g| {
            if let Some(ref cpf) = g.document_cpf {
                let t = cpf.trim();
                if !t.is_empty() {
                    g.document_cpf = encrypt_deterministic(t).ok().or(Some(t.to_string()));
                }
            }
            if let Some(ref rg) = g.document_rg {
                let t = rg.trim();
                if !t.is_empty() {
                    g.document_rg = encrypt_deterministic(t).ok().or(Some(t.to_string()));
                }
            }
            g
        })
        .collect();

    let enc_guardian_cpf = data.legal_guardian_cpf.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            encrypt_deterministic(t).ok().or(Some(t.to_string()))
        }
    });

    let mut res = db
        .query(
            "CREATE patient CONTENT {
            clinic_id: $cid,
            full_name: $full_name,
            document_cpf_encrypted: $cpf_enc,
            document_cpf_hash: $cpf_hash,
            document_rg: $rg,
            legal_guardians: $guardians,
            legal_guardian_name: $g_name,
            legal_guardian_cpf: $g_cpf,
            phone: $phone,
            email: $email,
            birth_date: $birth_date,
            gender: $gender,
            marital_status: $marital_status,
            profession: $profession,
            emergency_contact_name: $em_name,
            emergency_contact_phone: $em_phone,
            address_street: $st,
            address_number: $num,
            address_complement: $comp,
            address_neighborhood: $neigh,
            address_city: $city,
            address_state: $state,
            address_zip: $zip,
            insurance_plan: $ins_plan,
            insurance_number: $ins_num,
            password_hash: NONE,
            created_at: time::now(),
            updated_at: time::now()
        };",
        )
        .bind(("cid", clinic_rec.clone()))
        .bind(("full_name", data.full_name.trim().to_string()))
        .bind(("cpf_enc", cpf_encrypted))
        .bind(("cpf_hash", cpf_hash))
        .bind(("rg", data.document_rg.map(|s| s.trim().to_string())))
        .bind(("guardians", serde_json::to_value(&encrypted_guardians).unwrap_or_default()))
        .bind(("g_name", data.legal_guardian_name.map(|s| s.trim().to_string())))
        .bind(("g_cpf", enc_guardian_cpf))
        .bind(("phone", data.phone.trim().to_string()))
        .bind(("email", data.email.map(|s| s.trim().to_string())))
        .bind(("birth_date", data.birth_date.clone()))
        .bind(("gender", data.gender))
        .bind(("marital_status", data.marital_status))
        .bind(("profession", data.profession.map(|s| s.trim().to_string())))
        .bind((
            "em_name",
            data.emergency_contact_name.map(|s| s.trim().to_string()),
        ))
        .bind((
            "em_phone",
            data.emergency_contact_phone.map(|s| s.trim().to_string()),
        ))
        .bind(("st", data.address_street.map(|s| s.trim().to_string())))
        .bind(("num", data.address_number.map(|s| s.trim().to_string())))
        .bind((
            "comp",
            data.address_complement.map(|s| s.trim().to_string()),
        ))
        .bind((
            "neigh",
            data.address_neighborhood.map(|s| s.trim().to_string()),
        ))
        .bind(("city", data.address_city.map(|s| s.trim().to_string())))
        .bind(("state", data.address_state.map(|s| s.trim().to_string())))
        .bind(("zip", data.address_zip.map(|s| s.trim().to_string())))
        .bind((
            "ins_plan",
            data.insurance_plan.map(|s| s.trim().to_string()),
        ))
        .bind((
            "ins_num",
            data.insurance_number.map(|s| s.trim().to_string()),
        ))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar paciente: {}", e)))?;

    let created: Option<DbPatientRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = created else {
        return Err(ApiError::Database(
            "Falha ao retornar paciente criado.".into(),
        ));
    };

    // Auto-inicializar Ficha de Anamnese a partir do modelo da clínica
    let is_minor = if let Some(ref bd) = data.birth_date {
        if let Ok(naive) = chrono::NaiveDate::parse_from_str(bd, "%Y-%m-%d") {
            let now = chrono::Local::now().date_naive();
            now.years_since(naive).unwrap_or(0) < 18
        } else {
            false
        }
    } else {
        false
    };
    let target_template_type = if is_minor { "minor" } else { "adult" };

    let mut t_res = db
        .query("SELECT * FROM anamnesis_template WHERE clinic_id = $cid AND template_type = $ttype LIMIT 1;")
        .bind(("cid", clinic_rec.clone()))
        .bind(("ttype", target_template_type.to_string()))
        .await;

    let custom_responses: Vec<shared::anamnesis::AnamnesisResponseItem> = if let Ok(ref mut res_set) = t_res {
        let t_row: Option<super::DbAnamnesisTemplateRow> = res_set.take(0).unwrap_or(None);
        if let Some(template) = t_row {
            let questions: Vec<shared::anamnesis::AnamnesisQuestion> = template
                .questions
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            questions.into_iter().map(|q| {
                shared::anamnesis::AnamnesisResponseItem {
                    question_id: q.id,
                    category: q.category,
                    question_text: q.question_text,
                    question_type: q.question_type,
                    answer_boolean: Some(false),
                    answer_text: None,
                    notes: None,
                }
            }).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };


    let _ = db
        .query(
            "CREATE patient_anamnesis CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            template_type: $ttype,
            custom_responses: $responses,
            allergies: [],
            continuous_medications: NONE,
            systemic_diseases: [],
            is_pregnant: false,
            has_bleeding_disorder: false,
            smoker: false,
            bruxism: false,
            chief_complaint: NONE,
            clinical_notes: NONE,
            updated_at: time::now()
        };",
        )
        .bind(("pid", row.id.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("ttype", target_template_type.to_string()))
        .bind(("responses", serde_json::to_value(&custom_responses).unwrap_or_default()))
        .await;

    Ok(HttpResponse::Created().json(map_patient(row)))
}

/// Retorna o prontuário completo e integrado do paciente com todas as abas.
#[get("/patients/{id}")]
pub async fn get_patient_details(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<PatientPathQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "patients:read")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para consultar o prontuário deste paciente.".into(),
        ));
    }

    let mut res = db
        .query(
            "SELECT * FROM patient WHERE id = $pid LIMIT 1;
             SELECT * FROM patient_anamnesis WHERE patient_id = $pid LIMIT 1;
             SELECT * FROM patient_exam WHERE patient_id = $pid ORDER BY created_at DESC;
             SELECT * FROM patient_treatment WHERE patient_id = $pid ORDER BY created_at DESC;
             SELECT * FROM patient_document WHERE patient_id = $pid ORDER BY created_at DESC;
             SELECT * FROM patient_treatment_plan WHERE patient_id = $pid ORDER BY created_at DESC;",
        )
        .bind(("pid", pat_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao consultar prontuário do paciente: {}", e)))?;

    let pat_row: Option<DbPatientRow> = res.take(0).ok().and_then(|mut v: Vec<DbPatientRow>| v.pop());
    let Some(row) = pat_row else {
        return Err(ApiError::NotFound("Paciente não encontrado.".into()));
    };
    let patient = map_patient(row);

    let can_read_anamnese = check_permission(&db, &auth.id, &clinic_str, "anamnese:read")
        .await
        .unwrap_or(false);
    let can_read_exams = check_permission(&db, &auth.id, &clinic_str, "exams:read")
        .await
        .unwrap_or(false);
    let can_read_treatments = check_permission(&db, &auth.id, &clinic_str, "treatments:read")
        .await
        .unwrap_or(false);
    let can_read_documents = check_permission(&db, &auth.id, &clinic_str, "documents:read")
        .await
        .unwrap_or(false);

    let anam_row: Option<DbAnamnesisRow> = if can_read_anamnese {
        res.take(1).ok().and_then(|mut v: Vec<DbAnamnesisRow>| v.pop())
    } else {
        None
    };
    let anamnesis = anam_row.map(|a| PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
        template_type: a.template_type,
        custom_responses: a.custom_responses.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
        allergies: a.allergies.unwrap_or_default(),
        continuous_medications: a.continuous_medications,
        systemic_diseases: a.systemic_diseases.unwrap_or_default(),
        is_pregnant: a.is_pregnant.unwrap_or(false),
        has_bleeding_disorder: a.has_bleeding_disorder.unwrap_or(false),
        smoker: a.smoker.unwrap_or(false),
        bruxism: a.bruxism.unwrap_or(false),
        chief_complaint: a.chief_complaint,
        clinical_notes: a.clinical_notes,
        updated_at: a.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        signature_status: a.signature_status,
        signing_token: a.signing_token,
        signed_at: a.signed_at.map(|d| d.to_rfc3339()),
        signed_pdf_url: a.signed_pdf_url,
    });


    let exam_rows: Vec<DbExamRow> = if can_read_exams {
        res.take(2).unwrap_or_default()
    } else {
        vec![]
    };
    let exams: Vec<PatientExam> = exam_rows
        .into_iter()
        .map(|e| PatientExam {
            id: e.id.to_sql(),
            patient_id: e.patient_id.to_sql(),
            clinic_id: e.clinic_id.to_sql(),
            title: e.title,
            exam_type: e.exam_type,
            requested_by_user_id: e.requested_by_user_id.map(|u| u.to_sql()),
            requested_by_user_name: None,
            status: e.status.unwrap_or_else(|| "received".into()),
            requested_date: e.requested_date.map(|d| d.to_rfc3339()).unwrap_or_default(),
            result_date: e.result_date.map(|d| d.to_rfc3339()),
            file_urls: e.file_urls.unwrap_or_default(),
            clinical_interpretation: e.clinical_interpretation,
            created_at: e.created_at.to_rfc3339(),
        })
        .collect();

    let treat_rows: Vec<DbTreatmentRow> = if can_read_treatments {
        res.take(3).unwrap_or_default()
    } else {
        vec![]
    };
    let treatments: Vec<PatientTreatment> = treat_rows
        .into_iter()
        .map(|t| PatientTreatment {
            id: t.id.to_sql(),
            patient_id: t.patient_id.to_sql(),
            clinic_id: t.clinic_id.to_sql(),
            dentist_user_id: t.dentist_user_id.map(|u| u.to_sql()),
            dentist_user_name: None,
            appointment_id: t.appointment_id.map(|a| a.to_sql()),
            document_id: t.document_id.map(|d| d.to_sql()),
            exam_id: t.exam_id.map(|e| e.to_sql()),
            treatment_plan_id: t.treatment_plan_id.map(|p| p.to_sql()),
            transaction_id: t.transaction_id.map(|x| x.to_sql()),
            procedure_category: t.procedure_category,
            procedure_name: t.procedure_name,
            tooth_number: t.tooth_number,
            surfaces: t.surfaces,
            materials_used: t.materials_used,
            status: t.status.unwrap_or_else(|| "planned".into()),
            cost_cents: t.cost_cents.unwrap_or(0),
            post_care_instructions: t.post_care_instructions,
            clinical_notes: t.clinical_notes,
            performed_at: t.performed_at.map(|d| d.to_rfc3339()),
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();


    let doc_rows: Vec<DbDocumentRow> = if can_read_documents {
        res.take(4).unwrap_or_default()
    } else {
        vec![]
    };
    let documents: Vec<PatientDocument> = doc_rows
        .into_iter()
        .map(|d| {
            let is_anam = d.document_type.as_deref() == Some("anamnesis");
            let doc_type = d.document_type.unwrap_or_else(|| "contract".into());
            let req_pat = if is_anam { true } else { d.requires_patient_signature.unwrap_or(true) };
            let req_doc = if is_anam { false } else { d.requires_doctor_signature.unwrap_or(false) };
            let allow_any = if is_anam { false } else { d.allow_any_dentist_signature.unwrap_or(true) };

            PatientDocument {
                id: d.id.to_sql(),
                clinic_id: d.clinic_id.to_sql(),
                patient_id: d.patient_id.to_sql(),
                patient_name: Some(patient.full_name.clone()),
                template_id: d.template_id.map(|t| t.to_sql()),
                template_title: None,
                doctor_user_id: d.doctor_user_id.map(|u| u.to_sql()),
                doctor_user_name: None,
                appointment_id: d.appointment_id.map(|a| a.to_sql()),
                title: d.title,
                document_type: doc_type,
                original_pdf_url: d.original_pdf_url,
                signed_pdf_url: d.signed_pdf_url,
                status: d.status.unwrap_or_else(|| "pending_signatures".into()),
                signing_token: d.signing_token,
                requires_patient_signature: req_pat,
                requires_doctor_signature: req_doc,
                allow_any_dentist_signature: allow_any,
                patient_signed_at: d.patient_signed_at.map(|dt| dt.to_rfc3339()),
                patient_signature_data: d.patient_signature_data,
                doctor_signed_at: d.doctor_signed_at.map(|dt| dt.to_rfc3339()),
                doctor_signature_data: d.doctor_signature_data,
                patient_otp_verified: d.patient_otp_verified.unwrap_or(false),
                checksum_sha256: d.final_checksum_sha256,
                audit_trail: d.audit_trail.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                created_at: d.created_at.to_rfc3339(),
                updated_at: d.updated_at.to_rfc3339(),
            }
        })
        .collect();

    use crate::routes::patients::treatment_plans::{DbTreatmentPlanRow, map_plan};
    let plan_rows: Vec<DbTreatmentPlanRow> = if can_read_treatments {
        res.take(5).unwrap_or_default()
    } else {
        vec![]
    };
    let treatment_plans: Vec<shared::treatments::PatientTreatmentPlan> =
        plan_rows.into_iter().map(|r| map_plan(r, None)).collect();

    Ok(HttpResponse::Ok().json(PatientDetailsResponse {
        patient,
        anamnesis,
        exams,
        treatments,
        treatment_plans,
        documents,
    }))
}

/// Atualiza as informações cadastrais e dados de contato do paciente.
#[put("/patients/{id}")]
pub async fn update_patient(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<UpdatePatientRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "patients:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para editar dados do paciente.".into(),
        ));
    }

    let cpf_clean = data.document_cpf.as_deref().unwrap_or("").trim();
    let has_cpf = !cpf_clean.is_empty();

    // Buscar registro existente para preservar documentos criptografados caso venham mascarados da interface
    let mut exist_res = db
        .query("SELECT * FROM type::record($pid);")
        .bind(("pid", pat_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao verificar paciente existente: {}", e)))?;
    let exist_patient: Option<DbPatientRow> = exist_res.take(0).unwrap_or(None);

    let (cpf_encrypted, cpf_hash) = if cpf_clean.contains('*') {
        // Dado mascarado recebido do formulário: preserva os valores originais criptografados
        let enc = exist_patient
            .as_ref()
            .and_then(|r| r.document_cpf_encrypted.clone());
        let hash = exist_patient
            .as_ref()
            .and_then(|r| r.document_cpf_hash.clone());
        (enc, hash)
    } else if has_cpf {
        let enc = Some(encrypt_deterministic(cpf_clean)
            .map_err(|e| ApiError::Internal(format!("Falha na proteção de dados do CPF: {}", e)))?);
        let hash = Some(hash_blind_index(cpf_clean));
        (enc, hash)
    } else {
        (None, None)
    };

    let rg_value = if let Some(ref rg) = data.document_rg {
        let t = rg.trim();
        if t.contains('*') {
            exist_patient.as_ref().and_then(|r| r.document_rg.clone())
        } else if !t.is_empty() {
            Some(t.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let exist_guardians: Vec<shared::patients::PatientGuardian> = exist_patient
        .as_ref()
        .and_then(|r| r.legal_guardians.as_ref())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let guardians_list = data.legal_guardians.unwrap_or_default();
    let encrypted_guardians: Vec<shared::patients::PatientGuardian> = guardians_list
        .into_iter()
        .map(|mut g| {
            let matched_exist = exist_guardians.iter().find(|eg| eg.name.trim().eq_ignore_ascii_case(g.name.trim()));
            if let Some(ref cpf) = g.document_cpf {
                let t = cpf.trim();
                if t.contains('*') {
                    // Preserva o valor criptografado original já existente no banco de dados
                    g.document_cpf = matched_exist.and_then(|eg| eg.document_cpf.clone()).or(Some(t.to_string()));
                } else if !t.is_empty() {
                    // Criptografa o novo CPF do responsável legal
                    g.document_cpf = encrypt_deterministic(t).ok().or(Some(t.to_string()));
                }
            }
            if let Some(ref rg) = g.document_rg {
                let t = rg.trim();
                if t.contains('*') {
                    // Preserva o valor criptografado original já existente no banco de dados
                    g.document_rg = matched_exist.and_then(|eg| eg.document_rg.clone()).or(Some(t.to_string()));
                } else if !t.is_empty() {
                    // Criptografa o novo RG do responsável legal
                    g.document_rg = encrypt_deterministic(t).ok().or(Some(t.to_string()));
                }
            }
            g
        })
        .collect();

    let enc_guardian_cpf = data.legal_guardian_cpf.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else if t.contains('*') {
            exist_patient.as_ref().and_then(|r| r.legal_guardian_cpf.clone())
        } else {
            encrypt_deterministic(t).ok().or(Some(t.to_string()))
        }
    });

    let query = "UPDATE type::record($pid) SET
        full_name = $full_name,
        document_cpf_encrypted = $cpf_enc,
        document_cpf_hash = $cpf_hash,
        document_rg = $rg,
        legal_guardians = $guardians,
        legal_guardian_name = $g_name,
        legal_guardian_cpf = $g_cpf,
        phone = $phone,
        email = $email,
        birth_date = $birth_date,
        gender = $gender,
        marital_status = $marital_status,
        profession = $profession,
        emergency_contact_name = $em_name,
        emergency_contact_phone = $em_phone,
        address_street = $st,
        address_number = $num,
        address_complement = $comp,
        address_neighborhood = $neigh,
        address_city = $city,
        address_state = $state,
        address_zip = $zip,
        insurance_plan = $ins_plan,
        insurance_number = $ins_num,
        updated_at = time::now()"
        .to_string();

    let q_exec = db
        .query(&query)
        .bind(("pid", pat_rec))
        .bind(("full_name", data.full_name.trim().to_string()))
        .bind(("cpf_enc", cpf_encrypted))
        .bind(("cpf_hash", cpf_hash))
        .bind(("rg", rg_value))
        .bind(("guardians", serde_json::to_value(&encrypted_guardians).unwrap_or_default()))
        .bind(("g_name", data.legal_guardian_name.map(|s| s.trim().to_string())))
        .bind(("g_cpf", enc_guardian_cpf))
        .bind(("phone", data.phone.trim().to_string()))
        .bind(("email", data.email.map(|s| s.trim().to_string())))
        .bind(("birth_date", data.birth_date))
        .bind(("gender", data.gender))
        .bind(("marital_status", data.marital_status))
        .bind(("profession", data.profession.map(|s| s.trim().to_string())))
        .bind((
            "em_name",
            data.emergency_contact_name.map(|s| s.trim().to_string()),
        ))
        .bind((
            "em_phone",
            data.emergency_contact_phone.map(|s| s.trim().to_string()),
        ))
        .bind(("st", data.address_street.map(|s| s.trim().to_string())))
        .bind(("num", data.address_number.map(|s| s.trim().to_string())))
        .bind((
            "comp",
            data.address_complement.map(|s| s.trim().to_string()),
        ))
        .bind((
            "neigh",
            data.address_neighborhood.map(|s| s.trim().to_string()),
        ))
        .bind(("city", data.address_city.map(|s| s.trim().to_string())))
        .bind(("state", data.address_state.map(|s| s.trim().to_string())))
        .bind(("zip", data.address_zip.map(|s| s.trim().to_string())))
        .bind((
            "ins_plan",
            data.insurance_plan.map(|s| s.trim().to_string()),
        ))
        .bind((
            "ins_num",
            data.insurance_number.map(|s| s.trim().to_string()),
        ));

    let mut res = q_exec
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar paciente: {}", e)))?;


    let updated: Option<DbPatientRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = updated else {
        return Err(ApiError::NotFound(
            "Paciente não encontrado para atualização.".into(),
        ));
    };

    Ok(HttpResponse::Ok().json(map_patient(row)))
}

/// Exclui o paciente e todos os registros relacionados (anamnese, exames, tratamentos, documentos).
#[delete("/patients/{id}")]
pub async fn delete_patient(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<PatientPathQuery>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let clinic_str = clinic_record_id(&query.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "patients:delete")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem privilégios para excluir pacientes desta clínica.".into(),
        ));
    }

    db.query(
        "DELETE patient_anamnesis WHERE patient_id = $pid;
         DELETE patient_exam WHERE patient_id = $pid;
         DELETE patient_treatment WHERE patient_id = $pid;
         DELETE patient_document WHERE patient_id = $pid;
         DELETE patient WHERE id = $pid;",
    )
    .bind(("pid", pat_rec))
    .await
    .map_err(|e| ApiError::Database(format!("Erro ao excluir paciente: {}", e)))?;

    Ok(HttpResponse::Ok().body("Paciente excluído com sucesso."))
}
