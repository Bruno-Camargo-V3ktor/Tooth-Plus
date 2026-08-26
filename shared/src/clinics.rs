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
}

fn default_opening_hour() -> u32 { 8 }
fn default_closing_hour() -> u32 { 19 }

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
}
