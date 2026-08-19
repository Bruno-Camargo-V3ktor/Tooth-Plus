//! # Aba de Anamnese Odontológica (Frontend)
//!
//! Exibe e permite editar o questionário dinâmico de anamnese do paciente
//! com sincronização consentida baseada no modelo da clínica (Adulto / Menor).

use crate::api::{save_patient_anamnesis, sync_patient_anamnesis};
use crate::components::icons::{IconHeartPulse, IconRefresh};
use dioxus::prelude::*;
use shared::anamnesis::{AnamnesisResponseItem, SyncAnamnesisRequest};
use shared::patients::{PatientAnamnesis, SaveAnamnesisRequest};

/// Componente da aba de Anamnese Médica e Odontológica do Paciente.
#[component]
pub fn PatientAnamneseTab(
    patient_id: String,
    clinic_id: String,
    token: String,
    anamnesis: Option<PatientAnamnesis>,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let anam = anamnesis.clone().unwrap_or_default();
    let template_type_str = anam.template_type.clone().unwrap_or_else(|| "adult".to_string());

    // Se já tiver respostas dinâmicas gravadas, usa elas; caso contrário, inicializa a partir dos campos legados
    let initial_responses: Vec<AnamnesisResponseItem> = if !anam.custom_responses.is_empty() {
        anam.custom_responses.clone()
    } else {
        vec![
            AnamnesisResponseItem {
                question_id: "al_penicillin".into(),
                category: "Alergias".into(),
                question_text: "Alergia a Penicilina / Antibióticos?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.allergies.iter().any(|a| a.contains("Penicilina"))),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "al_dipyrone".into(),
                category: "Alergias".into(),
                question_text: "Alergia a Dipirona / Anti-inflamatórios?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.allergies.iter().any(|a| a.contains("Dipirona"))),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "al_latex".into(),
                category: "Alergias".into(),
                question_text: "Alergia a Látex?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.allergies.iter().any(|a| a == "Látex")),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "al_anesthetic".into(),
                category: "Alergias".into(),
                question_text: "Alergia a Anestésicos Locais?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.allergies.iter().any(|a| a.contains("Anestésic"))),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "dis_hypertension".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui Hipertensão Arterial (Pressão Alta)?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.systemic_diseases.iter().any(|d| d.contains("Hipertensão"))),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "dis_diabetes".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui Diabetes Mellitus?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.systemic_diseases.iter().any(|d| d == "Diabetes")),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "dis_cardiac".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Possui Cardiopatia ou problemas cardíacos?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.systemic_diseases.iter().any(|d| d == "Cardiopatia")),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "dis_bleeding".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Apresenta sangramento anormal ou distúrio de coagulação?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.has_bleeding_disorder),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "dis_pregnant".into(),
                category: "Saúde Sistêmica".into(),
                question_text: "Está gestante ou amamentando?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.is_pregnant),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "hab_smoker".into(),
                category: "Hábitos".into(),
                question_text: "Fumante ou faz uso de produtos de tabaco?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.smoker),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "hab_bruxism".into(),
                category: "Hábitos".into(),
                question_text: "Apresenta Bruxismo ou apertamento dental?".into(),
                question_type: "yes_no".into(),
                answer_boolean: Some(anam.bruxism),
                answer_text: None,
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "med_continuous".into(),
                category: "Medicamentos".into(),
                question_text: "Medicamentos de uso contínuo:".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: anam.continuous_medications.clone(),
                notes: None,
            },
            AnamnesisResponseItem {
                question_id: "chief_complaint".into(),
                category: "Queixa Principal".into(),
                question_text: "Queixa principal do paciente:".into(),
                question_type: "text".into(),
                answer_boolean: None,
                answer_text: anam.chief_complaint.clone(),
                notes: None,
            },
        ]
    };

    let mut responses_signal = use_signal(|| initial_responses);
    let mut is_saving = use_signal(|| false);
    let mut is_sync_modal_open = use_signal(|| false);
    let mut is_syncing = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();
    let on_reload = reload_patient_details.clone();
    let t_type_for_save = template_type_str.clone();

    let mut handle_save = move |_| {
        let current_resp = responses_signal();

        // Extrai dados para compatibilidade legada
        let mut allergies = Vec::new();
        let mut diseases = Vec::new();
        let mut continuous_meds = None;
        let mut chief_comp = None;
        let mut is_preg = false;
        let mut has_bleed = false;
        let mut smoker = false;
        let mut brux = false;

        for r in &current_resp {
            if r.category == "Alergias" && r.answer_boolean.unwrap_or(false) {
                allergies.push(r.question_text.clone());
            }
            if r.category == "Saúde Sistêmica" && r.answer_boolean.unwrap_or(false) {
                diseases.push(r.question_text.clone());
            }
            if r.question_id.contains("preg") {
                is_preg = r.answer_boolean.unwrap_or(false);
            }
            if r.question_id.contains("bleed") {
                has_bleed = r.answer_boolean.unwrap_or(false);
            }
            if r.question_id.contains("smoke") {
                smoker = r.answer_boolean.unwrap_or(false);
            }
            if r.question_id.contains("brux") {
                brux = r.answer_boolean.unwrap_or(false);
            }
            if r.category == "Medicamentos" {
                continuous_meds = r.answer_text.clone();
            }
            if r.category == "Queixa Principal" {
                chief_comp = r.answer_text.clone();
            }
        }

        let req = SaveAnamnesisRequest {
            clinic_id: cid.clone(),
            template_type: Some(t_type_for_save.clone()),
            custom_responses: Some(current_resp),
            allergies,
            continuous_medications: continuous_meds,
            systemic_diseases: diseases,
            is_pregnant: is_preg,
            has_bleeding_disorder: has_bleed,
            smoker,
            bruxism: brux,
            chief_complaint: chief_comp,
            clinical_notes: None,
        };

        let t = tok.clone();
        let p = pat_id.clone();
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut save_sig = is_saving;
        let reload = on_reload.clone();

        save_sig.set(true);
        spawn(async move {
            match save_patient_anamnesis(&t, &p, req).await {
                Ok(_) => {
                    toast.set(Some("Ficha de anamnese salva com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao salvar anamnese: {}", e)));
                }
            }
            save_sig.set(false);
        });
    };

    let tok_sync = token.clone();
    let cid_sync = clinic_id.clone();
    let pat_id_sync = patient_id.clone();
    let t_type_sync = template_type_str.clone();
    let on_reload_sync = reload_patient_details.clone();

    let mut handle_sync_template = move |_| {
        let req = SyncAnamnesisRequest {
            clinic_id: cid_sync.clone(),
            template_type: Some(t_type_sync.clone()),
        };

        let t = tok_sync.clone();
        let p = pat_id_sync.clone();
        let mut modal_sig = is_sync_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sync_sig = is_syncing;
        let reload = on_reload_sync.clone();

        sync_sig.set(true);
        spawn(async move {
            match sync_patient_anamnesis(&t, &p, req).await {
                Ok(updated_anam) => {
                    modal_sig.set(false);
                    responses_signal.set(updated_anam.custom_responses);
                    toast.set(Some("Ficha atualizada pelo modelo mais recente da clínica!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao sincronizar ficha: {}", e)));
                }
            }
            sync_sig.set(false);
        });
    };

    // Agrupar perguntas por categoria
    let current_responses = responses_signal();
    let mut categories: Vec<String> = Vec::new();
    for r in &current_responses {
        if !categories.contains(&r.category) {
            categories.push(r.category.clone());
        }
    }

    let is_minor_template = template_type_str == "minor";
    let template_badge = if is_minor_template { "Ficha Odontopediátrica (Menor)" } else { "Ficha Clínica Padrão (Adulto)" };

    rsx! {
        div { class: "anamnese-cards-container",
            // Header com Ações e Badges
            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;",
                div { style: "display: flex; align-items: center; gap: 10px;",
                    span { class: "badge-insurance-plan", "{template_badge}" }
                    if !anam.updated_at.is_empty() {
                        span { class: "text-muted font-xs",
                            "Última atualização: {anam.updated_at.chars().take(10).collect::<String>()}"
                        }
                    }
                }
                if can_write {
                    div { style: "display: flex; gap: 10px;",
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            onclick: move |_| is_sync_modal_open.set(true),
                            IconRefresh { size: 14, color: "currentColor".to_string() }
                            span { " Atualizar pelo Modelo Mais Recente" }
                        }
                        button {
                            r#type: "button",
                            class: "btn-primary",
                            disabled: is_saving(),
                            onclick: move |e| handle_save(e),
                            if is_saving() { "Salvando..." } else { "Salvar Ficha de Anamnese" }
                        }
                    }
                }
            }

            // Cards de Perguntas por Categoria
            for cat in categories.iter() {
                {
                    let cat_name = cat.clone();
                    let items: Vec<(usize, AnamnesisResponseItem)> = current_responses
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.category == cat_name)
                        .map(|(idx, r)| (idx, r.clone()))
                        .collect();

                    rsx! {
                        div { key: "{cat_name}", class: "anamnese-card", style: "margin-bottom: 20px;",
                            h3 { class: "anamnese-card-title",
                                IconHeartPulse { size: 16, color: "#0052cc".to_string() }
                                span { " {cat_name}" }
                            }

                            div { style: "display: flex; flex-direction: column; gap: 14px;",
                                for (idx, item) in items {
                                    {
                                        let is_yes_no = item.question_type == "yes_no";
                                        let is_checked = item.answer_boolean.unwrap_or(false);
                                        let text_val = item.answer_text.clone().unwrap_or_default();
                                        let notes_val = item.notes.clone().unwrap_or_default();

                                        rsx! {
                                            div { key: "{item.question_id}", style: "padding: 12px 14px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px;",
                                                if is_yes_no {
                                                    div {
                                                        div { style: "display: flex; align-items: center; justify-content: space-between;",
                                                            label { class: "anamnese-checkbox-label", style: "font-weight: 500; font-size: 14px; margin: 0; color: #1e293b; cursor: pointer;",
                                                                input {
                                                                    r#type: "checkbox",
                                                                    checked: is_checked,
                                                                    disabled: !can_write,
                                                                    onchange: move |e| {
                                                                        let mut list = responses_signal();
                                                                        if idx < list.len() {
                                                                            list[idx].answer_boolean = Some(e.checked());
                                                                            responses_signal.set(list);
                                                                        }
                                                                    }
                                                                }
                                                                span { "{item.question_text}" }
                                                            }
                                                            span {
                                                                class: if is_checked { "badge-insurance-plan" } else { "text-muted font-xs" },
                                                                if is_checked { "Sim / Positivo" } else { "Não" }
                                                            }
                                                        }
                                                        if is_checked {
                                                            div { style: "margin-top: 8px;",
                                                                input {
                                                                    class: "form-input",
                                                                    placeholder: "Observações ou detalhes adicionais sobre esta condição...",
                                                                    value: "{notes_val}",
                                                                    disabled: !can_write,
                                                                    oninput: move |e| {
                                                                        let mut list = responses_signal();
                                                                        if idx < list.len() {
                                                                            list[idx].notes = Some(e.value());
                                                                            responses_signal.set(list);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    div {
                                                        label { style: "display: block; font-size: 13px; font-weight: 500; color: #1e293b; margin-bottom: 6px;", "{item.question_text}" }
                                                        textarea {
                                                            class: "modern-textarea form-input",
                                                            style: "min-height: 80px;",
                                                            placeholder: "Descreva detalhadamente...",
                                                            value: "{text_val}",
                                                            disabled: !can_write,
                                                            oninput: move |e| {
                                                                let mut list = responses_signal();
                                                                if idx < list.len() {
                                                                    list[idx].answer_text = Some(e.value());
                                                                    responses_signal.set(list);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Confirmação: Sincronizar Ficha com Modelo Mais Recente
            if is_sync_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "settings-header",
                            h2 { class: "settings-title", "Atualizar Modelo de Anamnese" }
                            button { class: "close-btn", onclick: move |_| is_sync_modal_open.set(false), "×" }
                        }
                        div { class: "settings-content",
                            p { "Deseja atualizar esta ficha para incluir as novas perguntas do modelo mais recente da clínica?" }
                            p { class: "text-muted font-xs mt-2",
                                "As respostas previamente preenchidas serão preservadas com segurança. Apenas novas perguntas serão adicionadas à ficha do paciente."
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| is_sync_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_syncing(),
                                onclick: move |e| handle_sync_template(e),
                                if is_syncing() { "Atualizando..." } else { "Confirmar Atualização da Ficha" }
                            }
                        }
                    }
                }
            }
        }
    }
}
