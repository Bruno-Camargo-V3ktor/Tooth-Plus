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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateClinicRequest {
    pub corporate_name: Option<String>,
    pub trading_name: Option<String>,
    pub document_cnpj: Option<String>,
    pub theme_color: Option<String>,
    pub address: Option<ClinicAddress>,
    pub auto_reminders: Option<bool>,
    pub require_esign: Option<bool>,
}
