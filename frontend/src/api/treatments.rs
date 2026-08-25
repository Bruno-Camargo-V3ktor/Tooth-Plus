//! # Módulo de Integração e Serviço de Procedimentos & Orçamentos (TreatmentsApi)

use super::mock_db::DB;
use shared::treatments::{
    CreateTreatmentTemplateRequest, TreatmentTemplate, UpdateTreatmentTemplateRequest,
};

pub struct TreatmentsApi;

impl TreatmentsApi {
    /// Lista o catálogo de procedimentos padrão da clínica.
    pub async fn list_templates(clinic_id: &str) -> Result<Vec<TreatmentTemplate>, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let list = db
            .treatment_templates
            .iter()
            .filter(|t| t.clinic_id == clinic_id)
            .cloned()
            .collect();

        Ok(list)
    }

    /// Cria um procedimento no catálogo padrão.
    pub async fn create_template(req: CreateTreatmentTemplateRequest) -> Result<TreatmentTemplate, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let tpl = TreatmentTemplate {
            id: format!("tpl:{}", db.treatment_templates.len() + 1),
            clinic_id: req.clinic_id,
            name: req.name,
            category: req.category,
            description: req.description,
            default_price_cents: req.default_price_cents,
            estimated_duration_minutes: req.estimated_duration_minutes,
            dental_regions: req.dental_regions,
            target_teeth: req.target_teeth,
            required_materials: req.required_materials,
            required_equipment: req.required_equipment,
            post_care_instructions: req.post_care_instructions,
            clinical_notes: req.clinical_notes,
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        db.treatment_templates.push(tpl.clone());
        Ok(tpl)
    }

    /// Atualiza um modelo de procedimento no catálogo.
    pub async fn update_template(
        template_id: &str,
        req: UpdateTreatmentTemplateRequest,
    ) -> Result<TreatmentTemplate, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let tpl = db
            .treatment_templates
            .iter_mut()
            .find(|t| t.id == template_id)
            .ok_or_else(|| format!("Procedimento {} não encontrado.", template_id))?;

        tpl.name = req.name;
        tpl.category = req.category;
        tpl.description = req.description;
        tpl.default_price_cents = req.default_price_cents;
        tpl.estimated_duration_minutes = req.estimated_duration_minutes;
        tpl.dental_regions = req.dental_regions;
        tpl.target_teeth = req.target_teeth;
        tpl.required_materials = req.required_materials;
        tpl.required_equipment = req.required_equipment;
        tpl.post_care_instructions = req.post_care_instructions;
        tpl.clinical_notes = req.clinical_notes;
        tpl.is_active = req.is_active;
        tpl.updated_at = chrono::Utc::now().to_rfc3339();

        Ok(tpl.clone())
    }

    /// Remove um procedimento do catálogo.
    pub async fn delete_template(template_id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let initial = db.treatment_templates.len();
        db.treatment_templates.retain(|t| t.id != template_id);

        if db.treatment_templates.len() == initial {
            return Err(format!("Procedimento {} não encontrado.", template_id));
        }

        Ok(())
    }
}
