use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClinicResponse {
    pub id: String,
    pub corporate_name: String,
    pub trading_name: String,
    pub document_cnpj: String,
    pub theme_color: String,
    pub logo_url: Option<String>,
    pub whatsapp_instance: Option<String>,
    pub address: ClinicAddress,
    pub auto_reminders: bool,
    pub require_esign: bool,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_user: Option<String>,
    pub smtp_from: Option<String>,
    pub smtp_tls: Option<bool>,
    /// Hora de abertura da clínica (0-23), padrão 8
    #[serde(default = "default_opening_hour")]
    pub opening_hour: u32,
    /// Hora de fechamento da clínica (0-23), padrão 19
    #[serde(default = "default_closing_hour")]
    pub closing_hour: u32,
    /// Lista de rótulos/tags de agendamento da clínica
    #[serde(default = "default_appointment_labels")]
    pub appointment_labels: Vec<String>,
}

fn default_opening_hour() -> u32 { 8 }
fn default_closing_hour() -> u32 { 19 }
fn default_appointment_labels() -> Vec<String> {
    vec![
        "Primeira Consulta".to_string(),
        "Retorno".to_string(),
        "Avaliação".to_string(),
        "Urgência".to_string(),
        "Cirurgia".to_string(),
        "Manutenção".to_string(),
        "Ortodontia".to_string(),
        "Prótese".to_string(),
    ]
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClinicAddress {
    pub street: String,
    pub number: String,
    pub complement: Option<String>,
    pub neighborhood: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateClinicRequest {
    pub corporate_name: Option<String>,
    pub trading_name: Option<String>,
    pub document_cnpj: Option<String>,
    pub theme_color: Option<String>,
    pub address: Option<ClinicAddress>,
    pub auto_reminders: Option<bool>,
    pub require_esign: Option<bool>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub smtp_from: Option<String>,
    pub smtp_tls: Option<bool>,
    pub opening_hour: Option<u32>,
    pub closing_hour: Option<u32>,
    pub appointment_labels: Option<Vec<String>>,
}
