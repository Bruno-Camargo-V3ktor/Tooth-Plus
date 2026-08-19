//! # Ficha e Modelos de Anamnese Odontológica (Backend)
//!
//! Controla o histórico de saúde sistêmica, alergias, medicações de uso contínuo,
//! questionários customizáveis para adultos e menores, e templates de anamnese da clínica.

use super::{clinic_record_id, parse_record_id, DbAnamnesisRow, DbAnamnesisTemplateRow};
use crate::db::Db;
use crate::error::ApiError;
use crate::security::auth_guard::{check_permission, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use shared::anamnesis::{
    AnamnesisQuestion, AnamnesisResponseItem, AnamnesisTemplate, SaveAnamnesisTemplateRequest,
    SyncAnamnesisRequest,
};
use shared::patients::{PatientAnamnesis, SaveAnamnesisRequest};
use surrealdb::types::ToSql;

/// Retorna perguntas padrão para adultos.
fn default_adult_questions() -> Vec<AnamnesisQuestion> {
    vec![
        AnamnesisQuestion {
            id: "al_penicillin".into(),
            category: "Alergias".into(),
            question_text: "Alergia a Penicilina / Antibióticos?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "al_dipyrone".into(),
            category: "Alergias".into(),
            question_text: "Alergia a Dipirona / Anti-inflamatórios?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "al_anesthetic".into(),
            category: "Alergias".into(),
            question_text: "Alergia a Anestésicos Locais?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "al_latex".into(),
            category: "Alergias".into(),
            question_text: "Alergia a Látex?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "dis_hypertension".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui Hipertensão Arterial (Pressão Alta)?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "dis_diabetes".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui Diabetes Mellitus?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "dis_cardiac".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Possui Cardiopatia ou problemas cardíacos?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "dis_bleeding".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Apresenta sangramento anormal ou distúrbio de coagulação?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "dis_pregnant".into(),
            category: "Saúde Sistêmica".into(),
            question_text: "Está gestante ou amamentando?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "hab_smoker".into(),
            category: "Hábitos".into(),
            question_text: "Fumante ou faz uso de produtos derivados de tabaco?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "hab_bruxism".into(),
            category: "Hábitos".into(),
            question_text: "Apresenta Bruxismo, apertamento dental ou dores na ATM?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "med_continuous".into(),
            category: "Medicamentos".into(),
            question_text: "Faz uso de algum medicamento contínuo? Quais?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "chief_complaint".into(),
            category: "Queixa Principal".into(),
            question_text: "Qual a queixa principal ou motivo da consulta?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
    ]
}

/// Retorna perguntas padrão para menores / odontopediatria.
fn default_minor_questions() -> Vec<AnamnesisQuestion> {
    vec![
        AnamnesisQuestion {
            id: "ped_al_meds".into(),
            category: "Alergias Pediátricas".into(),
            question_text: "A criança possui alergia a algum medicamento?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_respiratory".into(),
            category: "Histórico Pediátrico".into(),
            question_text: "Possui asma, bronquite ou problemas respiratórios frequentes?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_cardiac".into(),
            category: "Histórico Pediátrico".into(),
            question_text: "Possui cardiopatia congênita ou histórico de febre reumática?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_habits".into(),
            category: "Hábitos Infantis".into(),
            question_text: "Possui hábito de sucção de dedo, chupeta ou roer unhas?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_mouth_breathing".into(),
            category: "Hábitos Infantis".into(),
            question_text: "Respira pela boca ou ronca ao dormir?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_trauma".into(),
            category: "Histórico Odontológico".into(),
            question_text: "Já sofreu queda ou trauma na região da face e dentes?".into(),
            question_type: "yes_no".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_medications".into(),
            category: "Medicamentos".into(),
            question_text: "Uso contínuo de medicamentos pela criança?".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
        AnamnesisQuestion {
            id: "ped_chief_complaint".into(),
            category: "Queixa Principal".into(),
            question_text: "Queixa principal relatada pelos pais / responsáveis:".into(),
            question_type: "text".into(),
            options: None,
            required: false,
        },
    ]
}

/// Salva ou atualiza a ficha de anamnese do paciente com histórico médico e hábitos.
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

    if !check_permission(&db, &auth.id, &clinic_str, "anamnese:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para atualizar a ficha médica/anamnese.".into(),
        ));
    }


    let clinic_rec = parse_record_id("clinic", &data.clinic_id);
    let responses = data.custom_responses.unwrap_or_default();
    let template_type = data.template_type.unwrap_or_else(|| "adult".into());

    let mut res = db
        .query(
            "UPSERT patient_anamnesis SET
            patient_id = $pid,
            clinic_id = $cid,
            template_type = $ttype,
            custom_responses = $responses,
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
            WHERE patient_id = $pid;",
        )
        .bind(("pid", pat_rec.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("ttype", template_type.clone()))
        .bind(("responses", serde_json::to_value(&responses).unwrap_or_default()))
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

    let saved: Option<DbAnamnesisRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(a) = saved else {
        return Err(ApiError::Database(
            "Erro ao recuperar registro de anamnese.".into(),
        ));
    };

    let saved_responses = a
        .custom_responses
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or(responses);

    Ok(HttpResponse::Ok().json(PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
        template_type: a.template_type.or(Some(template_type)),
        custom_responses: saved_responses,
        allergies: a.allergies.unwrap_or(data.allergies),
        continuous_medications: a.continuous_medications.or(data.continuous_medications),
        systemic_diseases: a.systemic_diseases.unwrap_or(data.systemic_diseases),
        is_pregnant: a.is_pregnant.unwrap_or(data.is_pregnant),
        has_bleeding_disorder: a
            .has_bleeding_disorder
            .unwrap_or(data.has_bleeding_disorder),
        smoker: a.smoker.unwrap_or(data.smoker),
        bruxism: a.bruxism.unwrap_or(data.bruxism),
        chief_complaint: a.chief_complaint.or(data.chief_complaint),
        clinical_notes: a.clinical_notes.or(data.clinical_notes),
        updated_at: a
            .updated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    }))
}

/// Retorna os modelos de anamnese configurados para a clínica (Adulto e Menor).
#[get("/clinics/{id}/anamnesis-templates")]
pub async fn get_anamnesis_templates(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let clinic_str = clinic_record_id(&clinic_id);

    let can_read = check_permission(&db, &auth.id, &clinic_str, "anamnese:read")
        .await
        .unwrap_or(false)
        || check_permission(&db, &auth.id, &clinic_str, "anamnese:manage_templates")
            .await
            .unwrap_or(false);

    if !can_read {
        return Err(ApiError::Forbidden(
            "Sem permissão para visualizar modelos de anamnese.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &clinic_id);

    let mut res = db
        .query("SELECT * FROM anamnesis_template WHERE clinic_id = $cid;")
        .bind(("cid", clinic_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let rows: Vec<DbAnamnesisTemplateRow> = res.take(0).unwrap_or_default();

    let mut templates: Vec<AnamnesisTemplate> = rows
        .into_iter()
        .map(|r| {
            let q_list = r
                .questions
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            AnamnesisTemplate {
                id: r.id.to_sql(),
                clinic_id: r.clinic_id.to_sql(),
                template_type: r.template_type,
                title: r.title,
                questions: q_list,
                created_at: r.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: r.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }
        })
        .collect();


    // Se não tiver modelo Adulto salvo, cria a resposta padrão
    if !templates.iter().any(|t| t.template_type == "adult") {
        templates.push(AnamnesisTemplate {
            id: "default_adult".into(),
            clinic_id: clinic_rec.to_sql(),
            template_type: "adult".into(),
            title: "Ficha Padrão - Adulto".into(),
            questions: default_adult_questions(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        });
    }

    // Se não tiver modelo Infantil salvo, cria a resposta padrão
    if !templates.iter().any(|t| t.template_type == "minor") {
        templates.push(AnamnesisTemplate {
            id: "default_minor".into(),
            clinic_id: clinic_rec.to_sql(),
            template_type: "minor".into(),
            title: "Ficha Padrão - Menor / Odontopediatria".into(),
            questions: default_minor_questions(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        });
    }

    Ok(HttpResponse::Ok().json(templates))
}

/// Salva ou atualiza um modelo de anamnese da clínica (Adulto ou Menor).
#[post("/clinics/{id}/anamnesis-templates")]
pub async fn save_anamnesis_template(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<SaveAnamnesisTemplateRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let clinic_id = path.into_inner();
    let clinic_str = clinic_record_id(&clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "anamnese:manage_templates")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para configurar modelos de anamnese.".into(),
        ));
    }

    let clinic_rec = parse_record_id("clinic", &clinic_id);
    let data = req.into_inner();

    let mut res = db
        .query(
            "UPSERT anamnesis_template SET
            clinic_id = $cid,
            template_type = $ttype,
            title = $title,
            questions = $questions,
            updated_at = time::now()
            WHERE clinic_id = $cid AND template_type = $ttype;",
        )
        .bind(("cid", clinic_rec.clone()))
        .bind(("ttype", data.template_type.clone()))
        .bind(("title", data.title.clone()))
        .bind(("questions", serde_json::to_value(&data.questions).unwrap_or_default()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao salvar modelo de anamnese: {}", e)))?;

    let saved: Option<DbAnamnesisTemplateRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;

    let Some(t) = saved else {
        return Err(ApiError::Database("Falha ao salvar modelo de anamnese.".into()));
    };

    let questions = t
        .questions
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or(data.questions);

    Ok(HttpResponse::Ok().json(AnamnesisTemplate {
        id: t.id.to_sql(),
        clinic_id: t.clinic_id.to_sql(),
        template_type: t.template_type,
        title: t.title,
        questions,
        created_at: t.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        updated_at: t.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
    }))
}

/// Sincroniza a ficha do paciente com o modelo mais recente de anamnese da clínica.
#[post("/patients/{id}/anamnesis/sync")]
pub async fn sync_patient_anamnesis(
    auth: AuthenticatedUser,
    path: web::Path<String>,
    req: web::Json<SyncAnamnesisRequest>,
    db: web::Data<Db>,
) -> Result<HttpResponse, ApiError> {
    let pat_rec = parse_record_id("patient", &path.into_inner());
    let data = req.into_inner();
    let clinic_str = clinic_record_id(&data.clinic_id);

    if !check_permission(&db, &auth.id, &clinic_str, "anamnese:write")
        .await
        .unwrap_or(false)
    {
        return Err(ApiError::Forbidden(
            "Sem permissão para sincronizar anamnese do paciente.".into(),
        ));
    }


    let clinic_rec = parse_record_id("clinic", &data.clinic_id);

    // Buscar ficha existente do paciente
    let mut a_res = db
        .query("SELECT * FROM patient_anamnesis WHERE patient_id = type::record($pid) LIMIT 1;")
        .bind(("pid", pat_rec.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    let existing_anam: Option<DbAnamnesisRow> = a_res.take(0).unwrap_or(None);

    let template_type = data
        .template_type
        .or_else(|| existing_anam.as_ref().and_then(|a| a.template_type.clone()))
        .unwrap_or_else(|| "adult".into());

    // Buscar o modelo mais recente da clínica
    let mut t_res = db
        .query("SELECT * FROM anamnesis_template WHERE clinic_id = $cid AND template_type = $ttype LIMIT 1;")
        .bind(("cid", clinic_rec.clone()))
        .bind(("ttype", template_type.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;

    let t_row: Option<DbAnamnesisTemplateRow> = t_res.take(0).unwrap_or(None);
    let template_questions: Vec<AnamnesisQuestion> = t_row
        .and_then(|t| t.questions)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(|| {
            if template_type == "minor" {
                default_minor_questions()
            } else {
                default_adult_questions()
            }
        });

    let existing_responses: Vec<AnamnesisResponseItem> = existing_anam
        .as_ref()
        .and_then(|a| a.custom_responses.clone())
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Mesclar: preservar respostas existentes onde question_id ou question_text baterem
    let mut updated_responses = Vec::new();
    for q in template_questions {
        if let Some(prev) = existing_responses.iter().find(|r| r.question_id == q.id || r.question_text == q.question_text) {
            updated_responses.push(AnamnesisResponseItem {
                question_id: q.id,
                category: q.category,
                question_text: q.question_text,
                question_type: q.question_type,
                answer_boolean: prev.answer_boolean,
                answer_text: prev.answer_text.clone(),
                notes: prev.notes.clone(),
            });
        } else {
            updated_responses.push(AnamnesisResponseItem {
                question_id: q.id,
                category: q.category,
                question_text: q.question_text,
                question_type: q.question_type,
                answer_boolean: Some(false),
                answer_text: None,
                notes: None,
            });
        }
    }

    let mut res = db
        .query(
            "UPSERT patient_anamnesis SET
            patient_id = $pid,
            clinic_id = $cid,
            template_type = $ttype,
            custom_responses = $responses,
            updated_at = time::now()
            WHERE patient_id = $pid;",
        )
        .bind(("pid", pat_rec.clone()))
        .bind(("cid", clinic_rec.clone()))
        .bind(("ttype", template_type.clone()))
        .bind(("responses", serde_json::to_value(&updated_responses).unwrap_or_default()))
        .await
        .map_err(|e| ApiError::Database(format!("Erro ao atualizar anamnese: {}", e)))?;

    let saved: Option<DbAnamnesisRow> =
        res.take(0).map_err(|e| ApiError::Database(e.to_string()))?;
    let Some(a) = saved else {
        return Err(ApiError::Database("Erro ao sincronizar anamnese.".into()));
    };

    let final_responses = a
        .custom_responses
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or(updated_responses);

    Ok(HttpResponse::Ok().json(PatientAnamnesis {
        id: a.id.map(|t| t.to_sql()),
        patient_id: a.patient_id.to_sql(),
        clinic_id: a.clinic_id.to_sql(),
        template_type: a.template_type.or(Some(template_type)),
        custom_responses: final_responses,
        allergies: a.allergies.unwrap_or_default(),
        continuous_medications: a.continuous_medications,
        systemic_diseases: a.systemic_diseases.unwrap_or_default(),
        is_pregnant: a.is_pregnant.unwrap_or(false),
        has_bleeding_disorder: a.has_bleeding_disorder.unwrap_or(false),
        smoker: a.smoker.unwrap_or(false),
        bruxism: a.bruxism.unwrap_or(false),
        chief_complaint: a.chief_complaint,
        clinical_notes: a.clinical_notes,
        updated_at: a
            .updated_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    }))

}

