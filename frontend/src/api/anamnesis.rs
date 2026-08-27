use super::mock_db::DB;
use shared::anamnesis::{AnamnesisQuestion, AnamnesisTemplate, SaveAnamnesisTemplateRequest};

pub struct AnamnesisApi;

impl AnamnesisApi {
    /// Lista os modelos de anamnese configurados para a clínica.
    pub async fn list_templates(_clinic_id: &str) -> Result<Vec<AnamnesisTemplate>, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;
        Ok(db.anamnesis_templates.clone())
    }

    /// Salva ou cria um modelo de anamnese com suas perguntas.
    pub async fn save_template(req: SaveAnamnesisTemplateRequest) -> Result<AnamnesisTemplate, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let existing = db.anamnesis_templates.iter_mut().find(|t| t.clinic_id == req.clinic_id && t.template_type == req.template_type);

        if let Some(tpl) = existing {
            tpl.title = req.title;
            tpl.questions = req.questions;
            tpl.updated_at = chrono::Utc::now().to_rfc3339();
            return Ok(tpl.clone());
        }

        let new_tpl = AnamnesisTemplate {
            id: format!("anam:{}", db.anamnesis_templates.len() + 1),
            clinic_id: req.clinic_id,
            template_type: req.template_type,
            title: req.title,
            questions: req.questions,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        db.anamnesis_templates.push(new_tpl.clone());
        Ok(new_tpl)
    }

    /// Exclui um modelo de anamnese.
    pub async fn delete_template(template_id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let initial = db.anamnesis_templates.len();
        db.anamnesis_templates.retain(|t| t.id != template_id);

        if db.anamnesis_templates.len() == initial {
            return Err("Modelo de anamnese não encontrado.".to_string());
        }

        Ok(())
    }
}
