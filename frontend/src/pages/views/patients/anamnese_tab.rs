//! # Aba de Anamnese Odontológica (Frontend)
//!
//! Exibe a ficha médica, condições sistêmicas, alergias, medicações de uso contínuo,
//! queixa principal e modal para atualização clínica.

use crate::api::save_patient_anamnesis;
use crate::components::icons::{IconCheckCircle, IconHeartPulse};
use dioxus::prelude::*;
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
    let mut is_edit_modal_open = use_signal(|| false);

    // Modal state
    let anam = anamnesis.clone().unwrap_or_default();
    let mut form_allergies_penicillin = use_signal(|| anam.allergies.iter().any(|a| a == "Penicilina"));
    let mut form_allergies_dipyrone = use_signal(|| anam.allergies.iter().any(|a| a == "Dipirona"));
    let mut form_allergies_latex = use_signal(|| anam.allergies.iter().any(|a| a == "Látex"));
    let mut form_allergies_anesthetic = use_signal(|| anam.allergies.iter().any(|a| a == "Anestésico Local"));
    let mut form_medications = use_signal(|| anam.continuous_medications.clone().unwrap_or_default());
    let mut form_disease_diabetes = use_signal(|| anam.systemic_diseases.iter().any(|d| d == "Diabetes"));
    let mut form_disease_hypertension = use_signal(|| anam.systemic_diseases.iter().any(|d| d == "Hipertensão"));
    let mut form_disease_cardiac = use_signal(|| anam.systemic_diseases.iter().any(|d| d == "Cardiopatia"));
    let mut form_is_pregnant = use_signal(|| anam.is_pregnant);
    let mut form_has_bleeding = use_signal(|| anam.has_bleeding_disorder);
    let mut form_smoker = use_signal(|| anam.smoker);
    let mut form_bruxism = use_signal(|| anam.bruxism);
    let mut form_chief_complaint = use_signal(|| anam.chief_complaint.clone().unwrap_or_default());
    let mut form_clinical_notes = use_signal(|| anam.clinical_notes.clone().unwrap_or_default());
    let mut is_saving = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();

    let mut handle_save = move |_| {
        let mut allergies = Vec::new();
        if form_allergies_penicillin() { allergies.push("Penicilina".into()); }
        if form_allergies_dipyrone() { allergies.push("Dipirona".into()); }
        if form_allergies_latex() { allergies.push("Látex".into()); }
        if form_allergies_anesthetic() { allergies.push("Anestésico Local".into()); }

        let mut diseases = Vec::new();
        if form_disease_diabetes() { diseases.push("Diabetes".into()); }
        if form_disease_hypertension() { diseases.push("Hipertensão".into()); }
        if form_disease_cardiac() { diseases.push("Cardiopatia".into()); }

        let req = SaveAnamnesisRequest {
            clinic_id: cid.clone(),
            allergies,
            continuous_medications: if form_medications().trim().is_empty() { None } else { Some(form_medications().trim().to_string()) },
            systemic_diseases: diseases,
            is_pregnant: form_is_pregnant(),
            has_bleeding_disorder: form_has_bleeding(),
            smoker: form_smoker(),
            bruxism: form_bruxism(),
            chief_complaint: if form_chief_complaint().trim().is_empty() { None } else { Some(form_chief_complaint().trim().to_string()) },
            clinical_notes: if form_clinical_notes().trim().is_empty() { None } else { Some(form_clinical_notes().trim().to_string()) },
        };

        let t = tok.clone();
        let p = pat_id.clone();
        let mut open_sig = is_edit_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut save_sig = is_saving;
        let reload = reload_patient_details.clone();

        save_sig.set(true);
        spawn(async move {
            match save_patient_anamnesis(&t, &p, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Ficha de Anamnese atualizada com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao salvar anamnese: {}", e)));
                }
            }
            save_sig.set(false);
        });
    };

    let anam_data = anamnesis.clone().unwrap_or_default();

    rsx! {
        div { class: "patient-tab-content",
            div { class: "tab-actions-header",
                div {
                    h3 { class: "tab-title", "Ficha de Anamnese Odontológica" }
                    p { class: "tab-subtitle", "Histórico médico, queixas e condições de saúde sistêmica." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| is_edit_modal_open.set(true),
                        IconHeartPulse { size: 16, color: "currentColor".to_string() }
                        span { "Atualizar Anamnese" }
                    }
                }
            }

            div { class: "anamnese-details-grid",
                div { class: "anamnese-card alert-card",
                    h4 { class: "anamnese-card-title text-danger", "⚠️ Alergias Informadas" }
                    if anam_data.allergies.is_empty() {
                        p { class: "text-muted", "Nenhuma alergia relatada pelo paciente." }
                    } else {
                        div { class: "tags-wrap",
                            for alg in &anam_data.allergies {
                                span { class: "badge-danger", "{alg}" }
                            }
                        }
                    }
                }

                div { class: "anamnese-card warning-card",
                    h4 { class: "anamnese-card-title text-warning", "🩺 Doenças Sistêmicas & Condições" }
                    div { class: "conditions-list",
                        div { class: "condition-item",
                            span { "Diabetes:" }
                            strong { if anam_data.systemic_diseases.iter().any(|d| d == "Diabetes") { "Sim" } else { "Não" } }
                        }
                        div { class: "condition-item",
                            span { "Hipertensão:" }
                            strong { if anam_data.systemic_diseases.iter().any(|d| d == "Hipertensão") { "Sim" } else { "Não" } }
                        }
                        div { class: "condition-item",
                            span { "Cardiopatia:" }
                            strong { if anam_data.systemic_diseases.iter().any(|d| d == "Cardiopatia") { "Sim" } else { "Não" } }
                        }
                        div { class: "condition-item",
                            span { "Gestante:" }
                            strong { if anam_data.is_pregnant { "Sim" } else { "Não" } }
                        }
                        div { class: "condition-item",
                            span { "Distúrbio de Coagulação:" }
                            strong { if anam_data.has_bleeding_disorder { "Sim" } else { "Não" } }
                        }
                        div { class: "condition-item",
                            span { "Fumante:" }
                            strong { if anam_data.smoker { "Sim" } else { "Não" } }
                        }
                        div { class: "condition-item",
                            span { "Bruxismo:" }
                            strong { if anam_data.bruxism { "Sim" } else { "Não" } }
                        }
                    }
                }

                div { class: "anamnese-card full-width",
                    h4 { class: "anamnese-card-title", "💊 Medicamentos de Uso Contínuo" }
                    p { class: "notes-text",
                        "{anam_data.continuous_medications.as_deref().unwrap_or(\"Nenhum medicamento informado.\")}"
                    }
                }

                div { class: "anamnese-card full-width",
                    h4 { class: "anamnese-card-title", "🎯 Queixa Principal do Paciente" }
                    p { class: "notes-text",
                        "{anam_data.chief_complaint.as_deref().unwrap_or(\"Não relatada.\")}"
                    }
                }

                div { class: "anamnese-card full-width",
                    h4 { class: "anamnese-card-title", "📝 Observações Clínicas Complementares" }
                    p { class: "notes-text",
                        "{anam_data.clinical_notes.as_deref().unwrap_or(\"Nenhuma observação clínica registrada.\")}"
                    }
                }
            }

            if is_edit_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal modal-large",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Atualizar Ficha de Anamnese" }
                                p { class: "modal-subtitle", "Histórico médico, alergias, medicações de uso contínuo e queixa principal." }
                            }
                            button { class: "modal-close", onclick: move |_| is_edit_modal_open.set(false), "×" }
                        }
                        div { class: "modal-body scrollable",
                            div { class: "form-section-title", "Alergias Conhecidas" }
                            div { class: "checkbox-grid-4",
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_allergies_penicillin(),
                                        onchange: move |e| form_allergies_penicillin.set(e.checked())
                                    }
                                    span { "Penicilina" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_allergies_dipyrone(),
                                        onchange: move |e| form_allergies_dipyrone.set(e.checked())
                                    }
                                    span { "Dipirona" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_allergies_latex(),
                                        onchange: move |e| form_allergies_latex.set(e.checked())
                                    }
                                    span { "Látex" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_allergies_anesthetic(),
                                        onchange: move |e| form_allergies_anesthetic.set(e.checked())
                                    }
                                    span { "Anestésicos" }
                                }
                            }

                            div { class: "form-section-title mt-4", "Condições Sistêmicas & Hábitos" }
                            div { class: "checkbox-grid-4",
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_disease_diabetes(),
                                        onchange: move |e| form_disease_diabetes.set(e.checked())
                                    }
                                    span { "Diabetes" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_disease_hypertension(),
                                        onchange: move |e| form_disease_hypertension.set(e.checked())
                                    }
                                    span { "Hipertensão" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_disease_cardiac(),
                                        onchange: move |e| form_disease_cardiac.set(e.checked())
                                    }
                                    span { "Cardiopatia" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_is_pregnant(),
                                        onchange: move |e| form_is_pregnant.set(e.checked())
                                    }
                                    span { "Gestante" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_has_bleeding(),
                                        onchange: move |e| form_has_bleeding.set(e.checked())
                                    }
                                    span { "Distúrbio Hemorrágico" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_smoker(),
                                        onchange: move |e| form_smoker.set(e.checked())
                                    }
                                    span { "Fumante" }
                                }
                                label { class: "checkbox-card",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_bruxism(),
                                        onchange: move |e| form_bruxism.set(e.checked())
                                    }
                                    span { "Bruxismo" }
                                }
                            }

                            div { class: "form-group mt-4",
                                label { "Medicamentos de Uso Contínuo" }
                                textarea {
                                    class: "form-textarea",
                                    placeholder: "Ex: Losartana 50mg pela manhã, Insulina NPH à noite...",
                                    value: "{form_medications}",
                                    oninput: move |e| form_medications.set(e.value())
                                }
                            }

                            div { class: "form-group",
                                label { "Queixa Principal" }
                                textarea {
                                    class: "form-textarea",
                                    placeholder: "Ex: Dor no dente 16 ao mastigar alimentos frios...",
                                    value: "{form_chief_complaint}",
                                    oninput: move |e| form_chief_complaint.set(e.value())
                                }
                            }

                            div { class: "form-group",
                                label { "Observações Clínicas" }
                                textarea {
                                    class: "form-textarea",
                                    placeholder: "Ex: Paciente demonstra ansiedade em procedimentos cirúrgicos...",
                                    value: "{form_clinical_notes}",
                                    oninput: move |e| form_clinical_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_edit_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_saving(),
                                onclick: move |e| handle_save(e),
                                if is_saving() { "Salvando..." } else { "Salvar Anamnese" }
                            }
                        }
                    }
                }
            }
        }
    }
}
