use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use crate::security::crypto::{
    decrypt_deterministic, encrypt_deterministic, hash_blind_index, hash_password,
};
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::{DateTime, Datelike, Local, Utc};
use serde::Deserialize;
use shared::documents::PatientDocument;
use shared::patients::{
    CreatePatientExamRequest, CreatePatientRequest, CreatePatientTreatmentRequest,
    Patient, PatientAnamnesis, PatientDetailsResponse, PatientExam, PatientKpis,
    PatientListResponse, PatientTreatment, SaveAnamnesisRequest, UpdatePatientRequest,
};
use surrealdb::types::{RecordId, SurrealValue, ToSql};

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
struct DbPatientRow {
    id: RecordId,
    clinic_id: RecordId,
    full_name: String,
    document_cpf: Option<String>,
    document_cpf_encrypted: Option<String>,
    document_cpf_hash: Option<String>,
    phone: String,
    email: Option<String>,
    birth_date: Option<String>,
    gender: Option<String>,
    marital_status: Option<String>,
    profession: Option<String>,
    emergency_contact_name: Option<String>,
    emergency_contact_phone: Option<String>,
    address_street: Option<String>,
    address_number: Option<String>,
    address_complement: Option<String>,
    address_neighborhood: Option<String>,
    address_city: Option<String>,
    address_state: Option<String>,
    address_zip: Option<String>,
    insurance_plan: Option<String>,
    insurance_number: Option<String>,
    password_hash: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbAnamnesisRow {
    id: Option<RecordId>,
    patient_id: RecordId,
    clinic_id: RecordId,
    allergies: Option<Vec<String>>,
    continuous_medications: Option<String>,
    systemic_diseases: Option<Vec<String>>,
    is_pregnant: Option<bool>,
    has_bleeding_disorder: Option<bool>,
    smoker: Option<bool>,
    bruxism: Option<bool>,
    chief_complaint: Option<String>,
    clinical_notes: Option<String>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbExamRow {
    id: RecordId,
    patient_id: RecordId,
    clinic_id: RecordId,
    title: String,
    exam_type: String,
    requested_by_user_id: Option<RecordId>,
    status: Option<String>,
    requested_date: Option<DateTime<Utc>>,
    result_date: Option<DateTime<Utc>>,
    file_urls: Option<Vec<String>>,
    clinical_interpretation: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbTreatmentRow {
    id: RecordId,
    patient_id: RecordId,
    clinic_id: RecordId,
    dentist_user_id: Option<RecordId>,
    appointment_id: Option<RecordId>,
    procedure_name: String,
    tooth_number: Option<String>,
    status: Option<String>,
    cost_cents: Option<i64>,
    clinical_notes: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, SurrealValue)]
struct DbDocumentRow {
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
    checksum_sha256: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn map_patient(row: DbPatientRow) -> Patient {
    let decrypted_cpf = if let Some(ref enc) = row.document_cpf_encrypted {
        decrypt_deterministic(enc).unwrap_or_else(|_| row.document_cpf.clone().unwrap_or_else(|| "CPF Protegido".into()))
    } else {
        row.document_cpf.clone().unwrap_or_else(|| "Não informado".into())
    };
    let has_pwd = row.password_hash.is_some() && !row.password_hash.as_deref().unwrap_or("").is_empty();

    Patient {
        id: row.id.to_sql(),
        clinic_id: row.clinic_id.to_sql(),
        full_name: row.full_name,
        document_cpf: decrypted_cpf,
        phone: row.phone,
        email: row.email,
        birth_date: row.birth_date,
        gender: row.gender,
        marital_status: row.marital_status,
        profession: row.profession,
        emergency_contact_name: row.emergency_contact_name,
        emergency_contact_phone: row.emergency_contact_phone,
        address_street: row.address_street,
        address_number: row.address_number,
        address_complement: row.address_complement,
        address_neighborhood: row.address_neighborhood,
        address_city: row.address_city,
        address_state: row.address_state,
        address_zip: row.address_zip,
        insurance_plan: row.insurance_plan,
        insurance_number: row.insurance_number,
        has_signature_password: has_pwd,
        created_at: row.created_at.map(|d| d.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339()),
        updated_at: row.updated_at.map(|d| d.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339()),
    }
}

#[derive(Deserialize)]
pub struct PatientQuery {
    pub clinic_id: String,
    pub search: Option<String>,
}

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
        .query("SELECT * FROM patient WHERE clinic_id = $cid ORDER BY created_at DESC;
                SELECT count() FROM patient_document WHERE clinic_id = $cid AND status = 'pending_signatures' GROUP ALL;
                SELECT count() FROM patient_treatment WHERE clinic_id = $cid AND status = 'in_progress' GROUP ALL;")
        .bind(("cid", clinic_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao consultar pacientes: {}", e)))?;

    let raw_patients: Vec<DbPatientRow> = res.take(0).unwrap_or_default();
    
    #[derive(Deserialize, SurrealValue)]
    struct CountRow {
        count: usize,
    }
    let pending_docs_row: Option<CountRow> = res.take(1).ok().and_then(|mut v: Vec<CountRow>| v.pop());
    let active_treat_row: Option<CountRow> = res.take(2).ok().and_then(|mut v: Vec<CountRow>| v.pop());

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
            let matches_cpf = p.document_cpf.contains(&search_term);
            let matches_email = p.email.as_deref().unwrap_or("").to_lowercase().contains(&search_term);

            if matches_name || matches_phone || matches_cpf || matches_email {
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

    let cpf_clean = data.document_cpf.trim();
    let cpf_encrypted = encrypt_deterministic(cpf_clean)
        .map_err(|e| ApiError::Internal(format!("Falha na proteção de dados do CPF: {}", e)))?;
    let cpf_hash = hash_blind_index(cpf_clean);

    let password_hash = if let Some(ref pwd) = data.signature_password {
        if !pwd.trim().is_empty() {
            Some(hash_password(pwd.trim()).map_err(|e| ApiError::Internal(format!("Erro ao gerar hash de senha: {}", e)))?)
        } else { None }
    } else { None };

    let mut res = db
        .query("CREATE patient CONTENT {
            clinic_id: $cid,
            full_name: $full_name,
            document_cpf_encrypted: $cpf_enc,
            document_cpf_hash: $cpf_hash,
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
            password_hash: $pwd_hash,
            created_at: time::now(),
            updated_at: time::now()
        };")
        .bind(("cid", clinic_rec))
        .bind(("full_name", data.full_name.trim().to_string()))
        .bind(("cpf_enc", cpf_encrypted))
        .bind(("cpf_hash", cpf_hash))
        .bind(("phone", data.phone.trim().to_string()))
        .bind(("email", data.email.map(|s| s.trim().to_string())))
        .bind(("birth_date", data.birth_date))
        .bind(("gender", data.gender))
        .bind(("marital_status", data.marital_status))
        .bind(("profession", data.profession.map(|s| s.trim().to_string())))
        .bind(("em_name", data.emergency_contact_name.map(|s| s.trim().to_string())))
        .bind(("em_phone", data.emergency_contact_phone.map(|s| s.trim().to_string())))
        .bind(("st", data.address_street.map(|s| s.trim().to_string())))
        .bind(("num", data.address_number.map(|s| s.trim().to_string())))
        .bind(("comp", data.address_complement.map(|s| s.trim().to_string())))
        .bind(("neigh", data.address_neighborhood.map(|s| s.trim().to_string())))
        .bind(("city", data.address_city.map(|s| s.trim().to_string())))
        .bind(("state", data.address_state.map(|s| s.trim().to_string())))
        .bind(("zip", data.address_zip.map(|s| s.trim().to_string())))
        .bind(("ins_plan", data.insurance_plan.map(|s| s.trim().to_string())))
        .bind(("ins_num", data.insurance_number.map(|s| s.trim().to_string())))
        .bind(("pwd_hash", password_hash))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar paciente: {}", e)))?;

    let created: Option<DbPatientRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = created else {
        return Err(ApiError::Database("Falha ao retornar paciente criado.".into()));
    };

    Ok(HttpResponse::Created().json(map_patient(row)))
}

#[derive(Deserialize)]
pub struct PatientPathQuery {
    pub clinic_id: String,
}

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
        .query("SELECT * FROM type::record($pid);
                SELECT * FROM patient_anamnesis WHERE patient_id = type::record($pid) LIMIT 1;
                SELECT * FROM patient_exam WHERE patient_id = type::record($pid) ORDER BY created_at DESC;
                SELECT * FROM patient_treatment WHERE patient_id = type::record($pid) ORDER BY created_at DESC;
                SELECT * FROM patient_document WHERE patient_id = type::record($pid) ORDER BY created_at DESC;")
        .bind(("pid", pat_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao consultar prontuário do paciente: {}", e)))?;

    let pat_row: Option<DbPatientRow> = res.take(0).unwrap_or(None);
    let Some(row) = pat_row else {
        return Err(ApiError::NotFound("Paciente não encontrado.".into()));
    };
    let patient = map_patient(row);

    let anam_row: Option<DbAnamnesisRow> = res.take(1).unwrap_or(None);
    let anamnesis = anam_row.map(|a| PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
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
    });

    let exam_rows: Vec<DbExamRow> = res.take(2).unwrap_or_default();
    let exams: Vec<PatientExam> = exam_rows.into_iter().map(|e| PatientExam {
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
    }).collect();

    let treat_rows: Vec<DbTreatmentRow> = res.take(3).unwrap_or_default();
    let treatments: Vec<PatientTreatment> = treat_rows.into_iter().map(|t| PatientTreatment {
        id: t.id.to_sql(),
        patient_id: t.patient_id.to_sql(),
        clinic_id: t.clinic_id.to_sql(),
        dentist_user_id: t.dentist_user_id.map(|u| u.to_sql()),
        dentist_user_name: None,
        appointment_id: t.appointment_id.map(|a| a.to_sql()),
        procedure_name: t.procedure_name,
        tooth_number: t.tooth_number,
        status: t.status.unwrap_or_else(|| "planned".into()),
        cost_cents: t.cost_cents.unwrap_or(0),
        clinical_notes: t.clinical_notes,
        created_at: t.created_at.to_rfc3339(),
    }).collect();

    let doc_rows: Vec<DbDocumentRow> = res.take(4).unwrap_or_default();
    let documents: Vec<PatientDocument> = doc_rows.into_iter().map(|d| PatientDocument {
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
        document_type: d.document_type.unwrap_or_else(|| "contract".into()),
        original_pdf_url: d.original_pdf_url,
        signed_pdf_url: d.signed_pdf_url,
        status: d.status.unwrap_or_else(|| "pending_signatures".into()),
        signing_token: d.signing_token,
        patient_signed_at: d.patient_signed_at.map(|dt| dt.to_rfc3339()),
        patient_signature_data: d.patient_signature_data,
        doctor_signed_at: d.doctor_signed_at.map(|dt| dt.to_rfc3339()),
        doctor_signature_data: d.doctor_signature_data,
        patient_otp_verified: d.patient_otp_verified.unwrap_or(false),
        checksum_sha256: d.checksum_sha256,
        created_at: d.created_at.to_rfc3339(),
        updated_at: d.updated_at.to_rfc3339(),
    }).collect();

    Ok(HttpResponse::Ok().json(PatientDetailsResponse {
        patient,
        anamnesis,
        exams,
        treatments,
        documents,
    }))
}

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

    let cpf_clean = data.document_cpf.trim();
    let cpf_encrypted = encrypt_deterministic(cpf_clean)
        .map_err(|e| ApiError::Internal(format!("Falha na proteção de dados do CPF: {}", e)))?;
    let cpf_hash = hash_blind_index(cpf_clean);

    let mut query = "UPDATE type::record($pid) SET
        full_name = $full_name,
        document_cpf_encrypted = $cpf_enc,
        document_cpf_hash = $cpf_hash,
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
        updated_at = time::now()".to_string();

    if let Some(ref pwd) = data.new_signature_password {
        if !pwd.trim().is_empty() {
            query.push_str(", password_hash = $pwd_hash");
        }
    }

    let password_hash = data.new_signature_password.as_deref().and_then(|p| {
        if !p.trim().is_empty() {
            hash_password(p.trim()).ok()
        } else { None }
    });

    let mut q_exec = db
        .query(&query)
        .bind(("pid", pat_rec))
        .bind(("full_name", data.full_name.trim().to_string()))
        .bind(("cpf_enc", cpf_encrypted))
        .bind(("cpf_hash", cpf_hash))
        .bind(("phone", data.phone.trim().to_string()))
        .bind(("email", data.email.map(|s| s.trim().to_string())))
        .bind(("birth_date", data.birth_date))
        .bind(("gender", data.gender))
        .bind(("marital_status", data.marital_status))
        .bind(("profession", data.profession.map(|s| s.trim().to_string())))
        .bind(("em_name", data.emergency_contact_name.map(|s| s.trim().to_string())))
        .bind(("em_phone", data.emergency_contact_phone.map(|s| s.trim().to_string())))
        .bind(("st", data.address_street.map(|s| s.trim().to_string())))
        .bind(("num", data.address_number.map(|s| s.trim().to_string())))
        .bind(("comp", data.address_complement.map(|s| s.trim().to_string())))
        .bind(("neigh", data.address_neighborhood.map(|s| s.trim().to_string())))
        .bind(("city", data.address_city.map(|s| s.trim().to_string())))
        .bind(("state", data.address_state.map(|s| s.trim().to_string())))
        .bind(("zip", data.address_zip.map(|s| s.trim().to_string())))
        .bind(("ins_plan", data.insurance_plan.map(|s| s.trim().to_string())))
        .bind(("ins_num", data.insurance_number.map(|s| s.trim().to_string())));

    if let Some(hash_str) = password_hash {
        q_exec = q_exec.bind(("pwd_hash", hash_str));
    }

    let mut res = q_exec
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar paciente: {}", e)))?;

    let updated: Option<DbPatientRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(row) = updated else {
        return Err(ApiError::NotFound("Paciente não encontrado para atualização.".into()));
    };

    Ok(HttpResponse::Ok().json(map_patient(row)))
}

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

    db.query("DELETE patient_anamnesis WHERE patient_id = type::record($pid);
              DELETE patient_exam WHERE patient_id = type::record($pid);
              DELETE patient_treatment WHERE patient_id = type::record($pid);
              DELETE patient_document WHERE patient_id = type::record($pid);
              DELETE type::record($pid);")
        .bind(("pid", pat_rec))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao excluir paciente: {}", e)))?;

    Ok(HttpResponse::Ok().body("Paciente excluído com sucesso."))
}

#[post("/patients/{id}/anamnesis")]
pub async fn save_anamnesis(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<SaveAnamnesisRequest>,
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
            "Sem permissão para atualizar a ficha médica/anamnese.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);

    let mut res = db
        .query("UPSERT patient_anamnesis SET
            patient_id = $pid,
            clinic_id = $cid,
            allergies = $allergies,
            continuous_medications = $meds,
            systemic_diseases = $diseases,
            is_pregnant = $preg,
            has_bleeding_disorder = $bleed,
            smoker = $smoker,
            bruxism = $brux,
            chief_complaint = $complaint,
            clinical_notes = $notes,
            updated_at = time::now()
            WHERE patient_id = $pid;")
        .bind(("pid", pat_rec.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("allergies", data.allergies.clone()))
        .bind(("meds", data.continuous_medications.clone()))
        .bind(("diseases", data.systemic_diseases.clone()))
        .bind(("preg", data.is_pregnant))
        .bind(("bleed", data.has_bleeding_disorder))
        .bind(("smoker", data.smoker))
        .bind(("brux", data.bruxism))
        .bind(("complaint", data.chief_complaint.clone()))
        .bind(("notes", data.clinical_notes.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao salvar anamnese: {}", e)))?;

    let saved: Option<DbAnamnesisRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(a) = saved else {
        return Err(ApiError::Database("Erro ao recuperar registro de anamnese.".into()));
    };

    Ok(HttpResponse::Ok().json(PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
        allergies: a.allergies.unwrap_or(data.allergies),
        continuous_medications: a.continuous_medications.or(data.continuous_medications),
        systemic_diseases: a.systemic_diseases.unwrap_or(data.systemic_diseases),
        is_pregnant: a.is_pregnant.unwrap_or(data.is_pregnant),
        has_bleeding_disorder: a.has_bleeding_disorder.unwrap_or(data.has_bleeding_disorder),
        smoker: a.smoker.unwrap_or(data.smoker),
        bruxism: a.bruxism.unwrap_or(data.bruxism),
        chief_complaint: a.chief_complaint.or(data.chief_complaint),
        clinical_notes: a.clinical_notes.or(data.clinical_notes),
        updated_at: a.updated_at.map(|d| d.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339()),
    }))
}

#[post("/patients/{id}/exams")]
pub async fn create_exam(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<CreatePatientExamRequest>,
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
            "Sem permissão para adicionar exames ao paciente.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    let auth_rec = parse_record_id("user", &auth.id);

    let mut res = db
        .query("CREATE patient_exam CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            title: $title,
            exam_type: $etype,
            requested_by_user_id: $uid,
            status: 'received',
            file_urls: $urls,
            clinical_interpretation: $notes,
            requested_date: time::now(),
            result_date: time::now(),
            created_at: time::now()
        };")
        .bind(("pid", pat_rec))
        .bind(("cid", clinic_rec))
        .bind(("title", data.title.trim().to_string()))
        .bind(("etype", data.exam_type))
        .bind(("uid", auth_rec))
        .bind(("urls", data.file_urls.clone()))
        .bind(("notes", data.clinical_interpretation.clone()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao registrar exame: {}", e)))?;

    let created: Option<DbExamRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(e) = created else {
        return Err(ApiError::Database("Erro ao salvar exame.".into()));
    };

    Ok(HttpResponse::Created().json(PatientExam {
        id: e.id.to_sql(),
        patient_id: e.patient_id.to_sql(),
        clinic_id: e.clinic_id.to_sql(),
        title: e.title,
        exam_type: e.exam_type,
        requested_by_user_id: e.requested_by_user_id.map(|u| u.to_sql()),
        requested_by_user_name: None,
        status: e.status.unwrap_or_else(|| "received".into()),
        requested_date: e.requested_date.map(|d| d.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339()),
        result_date: e.result_date.map(|d| d.to_rfc3339()),
        file_urls: e.file_urls.unwrap_or(data.file_urls),
        clinical_interpretation: e.clinical_interpretation.or(data.clinical_interpretation),
        created_at: e.created_at.to_rfc3339(),
    }))
}

#[post("/patients/{id}/treatments")]
pub async fn create_treatment(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<CreatePatientTreatmentRequest>,
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
            "Sem permissão para adicionar procedimentos/tratamentos.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    
    let dentist_rec = if let Some(ref d_id) = data.dentist_user_id {
        parse_record_id("user", d_id)
    } else {
        parse_record_id("user", &auth.id)
    };

    let appt_rec = data.appointment_id.as_deref().map(|a| parse_record_id("appointment", a));

    let mut res = db
        .query("CREATE patient_treatment CONTENT {
            patient_id: $pid,
            clinic_id: $cid,
            dentist_user_id: $uid,
            appointment_id: $aid,
            procedure_name: $pname,
            tooth_number: $tooth,
            status: $status,
            cost_cents: $cost,
            clinical_notes: $notes,
            created_at: time::now()
        };")
        .bind(("pid", pat_rec))
        .bind(("cid", clinic_rec))
        .bind(("uid", dentist_rec))
        .bind(("aid", appt_rec))
        .bind(("pname", data.procedure_name.trim().to_string()))
        .bind(("tooth", data.tooth_number.map(|s| s.trim().to_string())))
        .bind(("status", data.status.clone()))
        .bind(("cost", data.cost_cents))
        .bind(("notes", data.clinical_notes.map(|s| s.trim().to_string())))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao criar tratamento: {}", e)))?;

    let created: Option<DbTreatmentRow> = res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(t) = created else {
        return Err(ApiError::Database("Erro ao salvar procedimento.".into()));
    };

    Ok(HttpResponse::Created().json(PatientTreatment {
        id: t.id.to_sql(),
        patient_id: t.patient_id.to_sql(),
        clinic_id: t.clinic_id.to_sql(),
        dentist_user_id: t.dentist_user_id.map(|u| u.to_sql()),
        dentist_user_name: None,
        appointment_id: t.appointment_id.map(|a| a.to_sql()),
        procedure_name: t.procedure_name,
        tooth_number: t.tooth_number,
        status: t.status.unwrap_or(data.status),
        cost_cents: t.cost_cents.unwrap_or(data.cost_cents),
        clinical_notes: t.clinical_notes,
        created_at: t.created_at.to_rfc3339(),
    }))
}
