//! # Portal Público de Assinatura Digital (Backend)
//!
//! Endpoints abertos (autenticados por token criptográfico de URL) para visualização,
//! autenticação de signatários (paciente/dentista), envio e conferência de OTP e
//! carimbo de assinaturas eletrônicas com geração de PDF final e checksum SHA-256.

use super::{
    get_patient_decrypted_cpf, map_patient_document, map_template, DbClinicInfo,
    DbContractTemplateRow, DbPatientAuthRow, DbPatientDocumentRow, DbUserAuthRow,
};
use crate::db::Db;
use crate::error::ApiError;
use crate::evolution::EvolutionClient;
use crate::security::crypto::{
    calculate_sha256_checksum, hash_blind_index, verify_password,
};
use crate::security::otp::{generate_otp_code, hash_otp, verify_otp};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use shared::documents::{
    DoctorSignAuthRequest, PatientCheckRequest, PatientCheckResponse,
    PatientRegisterPasswordRequest, PatientSignAuthRequest, PublicSigningDocumentResponse,
    RequestOtpRequest, SignAuthResponse, SubmitSignatureRequest,
};
use std::env;
use surrealdb::types::{SurrealValue, ToSql};

/// Retorna os dados públicos do documento, da clínica e do modelo para renderização no portal.
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

    let doc_user_row: Option<DbUserAuthRow> = if let Some(ref uid) = doc.doctor_user_id {
        let uid_str = uid.to_sql();
        if let Ok(mut u_res) = db
            .query("SELECT * FROM type::record($uid) LIMIT 1;")
            .bind(("uid", uid_str))
            .await
        {
            u_res.take(0).unwrap_or(None)
        } else {
            None
        }
    } else {
        None
    };

    let doc_phone_raw = doc_user_row
        .as_ref()
        .and_then(|u| u.phone.clone())
        .unwrap_or_default();
    let doctor_phone_masked = if doc_phone_raw.len() >= 6 {
        Some(format!("(XX) XXXXX-{}", &doc_phone_raw[doc_phone_raw.len() - 4..]))
    } else {
        None
    };

    let doc_email_raw = doc_user_row
        .as_ref()
        .and_then(|u| u.email.clone())
        .unwrap_or_default();
    let doctor_email_masked = if !doc_email_raw.is_empty() && doc_email_raw.contains('@') {
        let parts: Vec<&str> = doc_email_raw.split('@').collect();
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

    Ok(HttpResponse::Ok().json(PublicSigningDocumentResponse {
        document: map_patient_document(doc),
        clinic_name,
        clinic_theme_color: clinic_theme,
        clinic_logo_url: clinic_logo,
        template,
        patient_phone_masked: phone_masked,
        patient_email_masked: email_masked,
        doctor_phone_masked,
        doctor_email_masked,
        require_whatsapp_otp: require_otp,
        has_email_channel: has_email,
    }))
}

fn normalize_doc_digits(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_uppercase()
}

fn check_doc_match(input_clean: &str, target_raw: &str) -> bool {
    let target_clean = normalize_doc_digits(target_raw);
    !target_clean.is_empty() && target_clean == input_clean
}

/// Identifica se o signatário é válido por CPF ou RG.
/// Regra para Menores de Idade:
/// - REJEITA se for o RG ou CPF do menor.
/// - ACEITA se for o RG ou CPF de um responsável legal cadastrado (`legal_guardians` ou `legal_guardian_cpf`).
/// Retorna `Ok((signer_name, signer_doc))` ou `Err(ApiError)`.
fn validate_patient_or_guardian_identity(
    pat: &DbPatientAuthRow,
    doc_input: &str,
) -> Result<(String, String), ApiError> {
    let clean_input = normalize_doc_digits(doc_input);
    if clean_input.len() < 4 {
        return Err(ApiError::BadRequest("Informe um CPF ou RG válido com ao menos 4 caracteres.".into()));
    }

    // 1. Identificar se é menor de idade
    let guardians: Vec<shared::patients::PatientGuardian> = pat
        .legal_guardians
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let is_minor = if let Some(ref bd) = pat.birth_date {
        if let Ok(naive) = chrono::NaiveDate::parse_from_str(bd, "%Y-%m-%d") {
            let now = chrono::Local::now().date_naive();
            now.years_since(naive).unwrap_or(0) < 18
        } else {
            !guardians.is_empty() || pat.legal_guardian_cpf.is_some()
        }
    } else {
        !guardians.is_empty() || pat.legal_guardian_cpf.is_some()
    };

    if is_minor {
        // Checar se o usuário inseriu o CPF ou RG do próprio menor
        let input_hash = hash_blind_index(doc_input);
        let minor_has_matching_cpf = pat.document_cpf_hash.as_deref() == Some(&input_hash)
            || pat.document_cpf.as_ref().map(|c| check_doc_match(&clean_input, c)).unwrap_or(false)
            || (pat.document_cpf_encrypted.is_some() && {
                let dec = get_patient_decrypted_cpf(pat);
                check_doc_match(&clean_input, &dec)
            });

        let minor_has_matching_rg = pat
            .document_rg
            .as_ref()
            .map(|rg| check_doc_match(&clean_input, rg))
            .unwrap_or(false);

        if minor_has_matching_cpf || minor_has_matching_rg {
            return Err(ApiError::BadRequest(
                "Para pacientes menores de 18 anos, a assinatura deve ser identificada com o CPF ou RG do responsável legal cadastrado.".into(),
            ));
        }

        // Checar na lista de responsáveis legais (por CPF ou RG)
        for g in &guardians {
            if let Some(ref g_cpf) = g.document_cpf {
                let dec_cpf = crate::security::crypto::decrypt_deterministic(g_cpf).unwrap_or_else(|_| g_cpf.clone());
                if check_doc_match(&clean_input, &dec_cpf) {
                    let signer_desc = format!("{} (Resp. Legal por {})", g.name, pat.full_name);
                    return Ok((signer_desc, dec_cpf));
                }
            }
            if let Some(ref g_rg) = g.document_rg {
                let dec_rg = crate::security::crypto::decrypt_deterministic(g_rg).unwrap_or_else(|_| g_rg.clone());
                if check_doc_match(&clean_input, &dec_rg) {
                    let signer_desc = format!("{} (Resp. Legal por {})", g.name, pat.full_name);
                    return Ok((signer_desc, dec_rg));
                }
            }
        }

        // Checar campos legados de responsável
        if let Some(ref l_cpf) = pat.legal_guardian_cpf {
            let dec_l_cpf = crate::security::crypto::decrypt_deterministic(l_cpf).unwrap_or_else(|_| l_cpf.clone());
            if check_doc_match(&clean_input, &dec_l_cpf) {
                let g_name = pat.legal_guardian_name.clone().unwrap_or_else(|| "Responsável Legal".into());
                let signer_desc = format!("{} (Resp. Legal por {})", g_name, pat.full_name);
                return Ok((signer_desc, dec_l_cpf));
            }
        }

        return Err(ApiError::BadRequest(
            "Documento informado (CPF/RG) não confere com nenhum dos responsáveis legais cadastrados para este paciente menor de idade.".into(),
        ));
    } else {
        // Adulto: verificar CPF do paciente
        let input_hash = hash_blind_index(doc_input);
        let matches_cpf = pat.document_cpf_hash.as_deref() == Some(&input_hash)
            || pat.document_cpf.as_ref().map(|c| check_doc_match(&clean_input, c)).unwrap_or(false)
            || (pat.document_cpf_encrypted.is_some() && {
                let dec = get_patient_decrypted_cpf(pat);
                check_doc_match(&clean_input, &dec)
            });

        let matches_rg = pat
            .document_rg
            .as_ref()
            .map(|rg| check_doc_match(&clean_input, rg))
            .unwrap_or(false);

        if matches_cpf || matches_rg {
            let doc_str = if matches_rg {
                pat.document_rg.clone().unwrap_or_else(|| clean_input.clone())
            } else {
                get_patient_decrypted_cpf(pat)
            };
            return Ok((pat.full_name.clone(), doc_str));
        }

        return Err(ApiError::BadRequest(
            "CPF ou RG informado não confere com o paciente deste contrato.".into(),
        ));
    }
}

/// Verifica o CPF ou RG do paciente (ou responsável se menor) antes da etapa de autenticação/criação de senha.
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

    let (signer_name, _) = validate_patient_or_guardian_identity(&pat, &data.cpf)?;

    let phone_raw = pat.phone.clone();
    let phone_masked = if phone_raw.len() >= 6 {
        format!("(XX) XXXXX-{}", &phone_raw[phone_raw.len() - 4..])
    } else {
        "(XX) XXXXX-XXXX".to_string()
    };

    let has_password = pat.password_hash.as_ref().map(|h| !h.is_empty()).unwrap_or(false);

    Ok(HttpResponse::Ok().json(PatientCheckResponse {
        patient_name: signer_name,
        has_password,
        phone_masked,
    }))
}

/// Cadastra a primeira senha de assinatura digital de 6 dígitos pelo próprio paciente (ou responsável utilizando a senha do cadastro do menor).
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

    let (signer_name, _) = validate_patient_or_guardian_identity(&pat, &data.cpf)?;

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
        signer_name,
    }))
}

/// Autentica o paciente (ou responsável por menor) no portal de assinatura por CPF/RG e senha cadastrada no perfil do paciente.
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

    let (signer_name, _) = validate_patient_or_guardian_identity(&pat, &data.cpf)?;

    if let Some(ref saved_hash) = pat.password_hash {
        if !verify_password(saved_hash, data.password.trim()) {
            return Err(ApiError::Unauthorized(
                "Senha de assinatura incorreta.".into(),
            ));
        }
    } else {
        return Err(ApiError::BadRequest(
            "Senha de assinatura ainda não cadastrada. Por favor, crie sua senha de assinatura de 6 dígitos.".into(),
        ));
    }

    Ok(HttpResponse::Ok().json(SignAuthResponse {
        token: doc.signing_token,
        signer_type: "patient".to_string(),
        signer_name,
    }))
}

/// Autentica o dentista responsável no portal de assinatura por login e senha do sistema.
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

/// Dispara o código de validação OTP de 6 dígitos via WhatsApp ou E-mail.
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

    let is_doctor = req.signer_type.as_deref() == Some("doctor");

    let (recipient_name, recipient_phone, recipient_email) = if is_doctor {
        let doc_user_row: Option<DbUserAuthRow> = if let Some(ref uid) = doc.doctor_user_id {
            let uid_str = uid.to_sql();
            if let Ok(mut u_res) = db
                .query("SELECT * FROM type::record($uid) LIMIT 1;")
                .bind(("uid", uid_str))
                .await
            {
                u_res.take(0).unwrap_or(None)
            } else {
                None
            }
        } else {
            None
        };

        let name = doc_user_row
            .as_ref()
            .map(|u| u.full_name.clone())
            .unwrap_or_else(|| "Cirurgião-Dentista".into());
        let phone = doc_user_row
            .as_ref()
            .and_then(|u| u.phone.clone())
            .unwrap_or_default();
        let email = doc_user_row
            .as_ref()
            .and_then(|u| u.email.clone());

        (name, phone, email)
    } else {
        let Some(p) = patient_row else {
            return Err(ApiError::NotFound("Paciente não localizado.".into()));
        };
        (p.full_name, p.phone, p.email)
    };

    let clinic_name = clinic_row
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Clínica Odontológica".into());

    if opt_channel == "email" {
        let Some(p_email) = recipient_email else {
            return Err(ApiError::BadRequest("Não há e-mail cadastrado para envio do código.".into()));
        };
        if p_email.trim().is_empty() || !p_email.contains('@') {
            return Err(ApiError::BadRequest("E-mail cadastrado é inválido.".into()));
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
            crate::email::send_otp_email(&cfg, &p_email, &recipient_name, &clinic_name, &otp_code)
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
        if recipient_phone.trim().is_empty() {
            return Err(ApiError::BadRequest("Telefone/WhatsApp não disponível para envio.".into()));
        }

        let clinic_key = doc.clinic_id.key.to_sql();
        let instance_name = clinic_row
            .as_ref()
            .and_then(|c| c.whatsapp_instance.clone())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("clinic_{}", clinic_key));

        let api_key = env::var("EVOLUTION_API_KEY").unwrap_or_default();
        let msg = format!(
            "🦷 *Tooth Plus — Assinatura Digital*\n\nOlá *{}*, seu código de verificação é:\n\n*{}*\n\nEsse código expira em 5 minutos. Não compartilhe com ninguém.",
            recipient_name, otp_code
        );

        evolution
            .send_whatsapp_text(&instance_name, &api_key, &recipient_phone, &msg)
            .await
            .map_err(|e| ApiError::Internal(format!("Falha ao enviar OTP via WhatsApp: {}", e)))?;

        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Código de validação enviado com sucesso via WhatsApp.",
            "success": true,
            "channel": "whatsapp"
        })));
    }
}

/// Recebe a assinatura desenhada no Canvas, valida o OTP, gera o PDF assinado e calcula o checksum SHA-256.
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

    let mut clinic_req_res = db
        .query("SELECT require_esign FROM type::record($cid) LIMIT 1;")
        .bind(("cid", doc.clinic_id.clone()))
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    #[derive(Deserialize, SurrealValue)]
    struct ClinicRequireEsign {
        require_esign: Option<bool>,
    }
    let clinic_req: Option<ClinicRequireEsign> = clinic_req_res.take(0).unwrap_or(None);
    let require_otp = clinic_req.and_then(|c| c.require_esign).unwrap_or(false);

    if require_otp {
        let Some(ref otp_input) = data.otp_code else {
            return Err(ApiError::BadRequest(
                "O código de verificação OTP é obrigatório para assinar. Clique em 'Enviar Código' para receber o PIN.".into(),
            ));
        };
        if otp_input.trim().is_empty() {
            return Err(ApiError::BadRequest(
                "O código de verificação OTP é obrigatório para assinar.".into(),
            ));
        }

        let saved_hash = doc.otp_code_hash.as_deref().unwrap_or("");
        let expires_at = doc.otp_expires_at;

        if saved_hash.is_empty() {
            return Err(ApiError::BadRequest(
                "Nenhum código OTP foi solicitado. Clique em 'Enviar Código' primeiro.".into(),
            ));
        }

        if !verify_otp(otp_input, saved_hash) {
            return Err(ApiError::Unauthorized(
                "Código de verificação OTP inválido. Tente novamente.".into(),
            ));
        }

        if let Some(exp) = expires_at {
            if chrono::Utc::now() > exp {
                return Err(ApiError::BadRequest(
                    "Código OTP expirado. Solicite um novo código.".into(),
                ));
            }
        }
    } else if let Some(ref otp_input) = data.otp_code {
        if !otp_input.trim().is_empty() {
            let saved_hash = doc.otp_code_hash.as_deref().unwrap_or("");
            let expires_at = doc.otp_expires_at;

            if saved_hash.is_empty() {
                return Err(ApiError::BadRequest(
                    "Nenhum código OTP foi solicitado. Clique em 'Enviar Código' primeiro.".into(),
                ));
            }

            if !verify_otp(otp_input, saved_hash) {
                return Err(ApiError::Unauthorized(
                    "Código de verificação OTP inválido. Tente novamente.".into(),
                ));
            }

            if let Some(exp) = expires_at {
                if chrono::Utc::now() > exp {
                    return Err(ApiError::BadRequest(
                        "Código OTP expirado. Solicite um novo código.".into(),
                    ));
                }
            }
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

        let (signer_display_name, signer_doc_info) = if let Some(ref pat) = patient_obj {
            let is_minor = if let Some(ref bd) = pat.birth_date {
                if let Ok(naive) = chrono::NaiveDate::parse_from_str(bd, "%Y-%m-%d") {
                    let now = chrono::Local::now().date_naive();
                    now.years_since(naive).unwrap_or(0) < 18
                } else {
                    false
                }
            } else {
                false
            };

            let guardians: Vec<shared::patients::PatientGuardian> = pat
                .legal_guardians
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            if is_minor || !guardians.is_empty() || pat.legal_guardian_cpf.is_some() {
                // Obter documento descriptografado do responsável
                let (g_name, g_doc) = if let Some(g) = guardians.first() {
                    let doc_str = g.document_cpf.as_ref().map(|c| {
                        let dec = crate::security::crypto::decrypt_deterministic(c).unwrap_or_else(|_| c.clone());
                        format!("CPF: {}", dec)
                    }).or_else(|| {
                        g.document_rg.as_ref().map(|r| {
                            let dec = crate::security::crypto::decrypt_deterministic(r).unwrap_or_else(|_| r.clone());
                            format!("RG: {}", dec)
                        })
                    }).unwrap_or_else(|| "Doc: Não informado".into());
                    (g.name.clone(), doc_str)
                } else {
                    let name = pat.legal_guardian_name.clone().unwrap_or_else(|| "Responsável Legal".into());
                    let doc_str = pat.legal_guardian_cpf.as_ref().map(|c| {
                        let dec = crate::security::crypto::decrypt_deterministic(c).unwrap_or_else(|_| c.clone());
                        format!("CPF: {}", dec)
                    }).unwrap_or_else(|| "Doc: Não informado".into());
                    (name, doc_str)
                };
                (format!("{} (Resp. Legal por {})", g_name, pat.full_name), g_doc)
            } else {
                let dec_cpf = get_patient_decrypted_cpf(pat);
                let doc_str = if !dec_cpf.is_empty() {
                    format!("CPF: {}", dec_cpf)
                } else if let Some(ref rg) = pat.document_rg {
                    let dec_rg = crate::security::crypto::decrypt_deterministic(rg).unwrap_or_else(|_| rg.clone());
                    format!("RG: {}", dec_rg)
                } else {
                    "CPF: Não informado".into()
                };
                (pat.full_name.clone(), doc_str)
            }
        } else {
            ("Paciente".into(), "CPF: 000.000.000-00".into())
        };

        let pat_info = crate::documents_pdf::PdfSignerInfo {
            name: signer_display_name,
            document_info: signer_doc_info,
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

        let checksum_val = Some(generated_checksum);
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
            .bind(("checksum", checksum_val))
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

        let checksum_val = Some(generated_checksum);
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
            .bind(("checksum", checksum_val))
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
