//! # Módulo de Integração e Serviço de Clínicas (ClinicsApi)

use super::mock_db::DB;
use shared::clinics::{ClinicResponse, UpdateClinicRequest};

pub struct ClinicsApi;

impl ClinicsApi {
    /// Lista todas as clínicas cadastradas.
    pub async fn list_clinics() -> Result<Vec<ClinicResponse>, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;
        Ok(db.clinics.clone())
    }

    /// Obtém os dados de uma clínica específica.
    pub async fn get_clinic_by_id(clinic_id: &str) -> Result<ClinicResponse, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        db.clinics
            .iter()
            .find(|c| c.id == clinic_id)
            .cloned()
            .ok_or_else(|| format!("Clínica {} não encontrada.", clinic_id))
    }

    /// Atualiza configurações ou dados cadastrais de uma clínica.
    pub async fn update_clinic(clinic_id: &str, req: UpdateClinicRequest) -> Result<ClinicResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let clinic = db
            .clinics
            .iter_mut()
            .find(|c| c.id == clinic_id)
            .ok_or_else(|| format!("Clínica {} não encontrada.", clinic_id))?;

        if let Some(corp) = req.corporate_name { clinic.corporate_name = corp; }
        if let Some(trad) = req.trading_name { clinic.trading_name = trad; }
        if let Some(cnpj) = req.document_cnpj { clinic.document_cnpj = cnpj; }
        if let Some(theme) = req.theme_color { clinic.theme_color = theme; }
        if let Some(addr) = req.address { clinic.address = addr; }
        if let Some(rem) = req.auto_reminders { clinic.auto_reminders = rem; }
        if let Some(es) = req.require_esign { clinic.require_esign = es; }
        if let Some(sh) = req.smtp_host { clinic.smtp_host = Some(sh); }
        if let Some(sp) = req.smtp_port { clinic.smtp_port = Some(sp); }
        if let Some(su) = req.smtp_user { clinic.smtp_user = Some(su); }
        if let Some(sf) = req.smtp_from { clinic.smtp_from = Some(sf); }
        if let Some(st) = req.smtp_tls { clinic.smtp_tls = Some(st); }

        Ok(clinic.clone())
    }
}
