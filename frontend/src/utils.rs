use chrono::{DateTime, Local, NaiveDate};
use shared::patients::Patient;

pub fn format_date_br(date_str: &str) -> String {
    let trimmed = date_str.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return dt.with_timezone(&Local).format("%d/%m/%Y").to_string();
    }

    let clean: String = trimmed.chars().take(10).collect();
    if let Ok(nd) = NaiveDate::parse_from_str(&clean, "%Y-%m-%d") {
        return nd.format("%d/%m/%Y").to_string();
    }

    let parts: Vec<&str> = clean.split('-').collect();
    if parts.len() == 3 && parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2 {
        return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
    }

    trimmed.to_string()
}

pub fn format_datetime_br(date_str: &str) -> String {
    let trimmed = date_str.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return dt
            .with_timezone(&Local)
            .format("%d/%m/%Y às %H:%M")
            .to_string();
    }

    let clean: String = trimmed.chars().take(10).collect();
    let parts: Vec<&str> = clean.split('-').collect();
    if parts.len() == 3 && parts[0].len() == 4 {
        return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
    }

    trimmed.to_string()
}

pub fn format_currency_br(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reais = abs_cents / 100;
    let centavos = abs_cents % 100;

    let reais_str = reais.to_string();
    let mut formatted_reais = String::new();
    let len = reais_str.len();

    for (idx, ch) in reais_str.chars().enumerate() {
        if idx > 0 && (len - idx) % 3 == 0 {
            formatted_reais.push('.');
        }
        formatted_reais.push(ch);
    }

    if is_negative {
        format!("-R$ {},{:02}", formatted_reais, centavos)
    } else {
        format!("R$ {},{:02}", formatted_reais, centavos)
    }
}

pub fn replace_template_variables(
    template: &str,
    patient: Option<&Patient>,
    clinic_name: Option<&str>,
    clinic_cnpj: Option<&str>,
    doctor_name: Option<&str>,
) -> String {
    let mut result = template.to_string();
    let today_br = Local::now().format("%d/%m/%Y").to_string();

    let c_name = clinic_name.unwrap_or("Clínica Odontológica");
    let c_cnpj = clinic_cnpj.unwrap_or("00.000.000/0001-00");
    let d_name = doctor_name.unwrap_or("Dr(a). Cirurgião-Dentista");

    result = result
        .replace("{{clinica_nome}}", c_name)
        .replace("{{nome_clinica}}", c_name)
        .replace("{{clinica_cnpj}}", c_cnpj)
        .replace("{{cnpj_clinica}}", c_cnpj)
        .replace("{{doutor_nome}}", d_name)
        .replace("{{nome_doutor}}", d_name)
        .replace("{{data_hoje}}", &today_br)
        .replace("{{data_atual}}", &today_br);

    if let Some(p) = patient {
        let p_name = &p.full_name;
        let p_cpf = p.document_cpf.as_deref().unwrap_or(p.document_rg.as_deref().unwrap_or("Não informado"));
        let p_phone = &p.phone;
        let p_email = p.email.as_deref().unwrap_or("Não informado");
        let p_insurance = p.insurance_plan.as_deref().unwrap_or("Particular");
        let p_birth = p
            .birth_date
            .as_deref()
            .map(format_date_br)
            .unwrap_or_else(|| "Não informada".into());

        let address_full = format!(
            "{}, {} - {}, {} - {} (CEP: {})",
            p.address_street.as_deref().unwrap_or(""),
            p.address_number.as_deref().unwrap_or("S/N"),
            p.address_neighborhood.as_deref().unwrap_or(""),
            p.address_city.as_deref().unwrap_or(""),
            p.address_state.as_deref().unwrap_or(""),
            p.address_zip.as_deref().unwrap_or("")
        );

        result = result
            .replace("{{paciente_nome}}", p_name)
            .replace("{{nome_paciente}}", p_name)
            .replace("{{paciente_cpf}}", p_cpf)
            .replace("{{cpf_paciente}}", p_cpf)
            .replace("{{cpf}}", p_cpf)
            .replace("{{paciente_telefone}}", p_phone)
            .replace("{{telefone_paciente}}", p_phone)
            .replace("{{telefone}}", p_phone)
            .replace("{{paciente_email}}", p_email)
            .replace("{{email_paciente}}", p_email)
            .replace("{{paciente_endereco}}", &address_full)
            .replace("{{endereco_paciente}}", &address_full)
            .replace("{{paciente_convenio}}", p_insurance)
            .replace("{{convenio}}", p_insurance)
            .replace("{{paciente_nascimento}}", &p_birth)
            .replace("{{data_nascimento}}", &p_birth);
    }

    result
}

/// Verifica se um token JWT está expirado com base no campo `exp`.
pub fn is_token_expired(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return true;
    }
    use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
    use base64::Engine;

    let payload_b64 = parts[1];
    let decoded = URL_SAFE_NO_PAD.decode(payload_b64).or_else(|_| {
        let mut padded = payload_b64.to_string();
        while padded.len() % 4 != 0 {
            padded.push('=');
        }
        STANDARD.decode(padded)
    });

    if let Ok(bytes) = decoded {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(exp) = val.get("exp").and_then(|e| e.as_i64()) {
                let now = chrono::Utc::now().timestamp();
                return now >= exp;
            }
        }
    }
    false
}

#[cfg(target_arch = "wasm32")]
pub fn get_storage_item(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item(key).ok())
        .flatten()
}

#[cfg(target_arch = "wasm32")]
pub fn set_storage_item(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.set_item(key, value);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn remove_storage_item(key: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
        let _ = storage.remove_item(key);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_storage_item(_key: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_storage_item(_key: &str, _value: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn remove_storage_item(_key: &str) {}

pub fn save_session(session: &shared::auth::LoginResponse) {
    if let Ok(json) = serde_json::to_string(session) {
        set_storage_item("toothplus_session", &json);
    }
}

pub fn load_session() -> Option<shared::auth::LoginResponse> {
    let sess: Option<shared::auth::LoginResponse> = get_storage_item("toothplus_session")
        .and_then(|json| serde_json::from_str(&json).ok());
    if let Some(ref s) = sess {
        if is_token_expired(&s.token) {
            clear_session();
            return None;
        }
    }
    sess
}

pub fn save_active_clinic(clinic: &shared::models::ClinicAccess) {
    if let Ok(json) = serde_json::to_string(clinic) {
        set_storage_item("toothplus_active_clinic", &json);
    }
}

pub fn load_active_clinic() -> Option<shared::models::ClinicAccess> {
    get_storage_item("toothplus_active_clinic")
        .and_then(|json| serde_json::from_str(&json).ok())
}

pub fn clear_session() {
    remove_storage_item("toothplus_session");
    remove_storage_item("toothplus_active_clinic");
}
