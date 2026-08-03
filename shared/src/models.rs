use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClinicAccess {
    pub clinic_id: String,
    pub trading_name: String,
    pub theme_color: String,
    pub logo_url: Option<String>,
    pub role: String,
}
