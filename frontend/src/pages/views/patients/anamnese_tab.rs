//! # Aba de Anamnese Odontológica (Frontend)
//!
//! Exibe e permite editar os 4 blocos de condições clínicas e saúde geral do paciente:
//! 1. Alergias Conhecidas
//! 2. Doenças Sistêmicas & Condições Especiais
//! 3. Medicamentos de Uso Contínuo
//! 4. Queixa Principal e Observações Clínicas

use crate::api::save_patient_anamnesis;
use crate::components::icons::IconHeartPulse;
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
    let anam = anamnesis.clone().unwrap_or_default();
    let mut form_allergies_penicillin = use_signal(|| anam.allergies.iter().any(|a| a == "Penicilina" || a == "Penicilina / Antibióticos"));
    let mut form_allergies_dipyrone = use_signal(|| anam.allergies.iter().any(|a| a == "Dipirona" || a == "Dipirona / Anti-inflamatórios"));
    let mut form_allergies_latex = use_signal(|| anam.allergies.iter().any(|a| a == "Látex"));
    let mut form_allergies_anesthetic = use_signal(|| anam.allergies.iter().any(|a| a == "Anestésicos Locais" || a == "Anestésico Local"));
    let mut form_medications = use_signal(|| anam.continuous_medications.clone().unwrap_or_default());
    let mut form_disease_diabetes = use_signal(|| anam.systemic_diseases.iter().any(|d| d == "Diabetes"));
    let mut form_disease_hypertension = use_signal(|| anam.systemic_diseases.iter().any(|d| d == "Hipertensão Arterial" || d == "Hipertensão"));
    let mut form_disease_cardiac = use_signal(|| anam.systemic_diseases.iter().any(|d| d == "Cardiopatia"));
    let mut form_has_bleeding = use_signal(|| anam.has_bleeding_disorder || anam.systemic_diseases.iter().any(|d| d == "Distúrbio Hemorrágico / Sangramento"));
    let mut form_is_pregnant = use_signal(|| anam.is_pregnant);
    let mut form_smoker = use_signal(|| anam.smoker);
    let mut form_bruxism = use_signal(|| anam.bruxism || anam.systemic_diseases.iter().any(|d| d == "Bruxismo / Apertamento Dental"));
    let mut form_chief_complaint = use_signal(|| anam.chief_complaint.clone().unwrap_or_default());
    let mut is_saving = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();

    let mut handle_save = move |_| {
        let mut allergies = Vec::new();
        if form_allergies_penicillin() { allergies.push("Penicilina / Antibióticos".into()); }
        if form_allergies_dipyrone() { allergies.push("Dipirona / Anti-inflamatórios".into()); }
        if form_allergies_latex() { allergies.push("Látex".into()); }
        if form_allergies_anesthetic() { allergies.push("Anestésicos Locais".into()); }

        let mut diseases = Vec::new();
        if form_disease_diabetes() { diseases.push("Diabetes".into()); }
        if form_disease_hypertension() { diseases.push("Hipertensão Arterial".into()); }
        if form_disease_cardiac() { diseases.push("Cardiopatia".into()); }
        if form_has_bleeding() { diseases.push("Distúrbio Hemorrágico / Sangramento".into()); }
        if form_bruxism() { diseases.push("Bruxismo / Apertamento Dental".into()); }

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
            clinical_notes: None,
        };

        let t = tok.clone();
        let p = pat_id.clone();
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut save_sig = is_saving;
        let reload = reload_patient_details.clone();

        save_sig.set(true);
        spawn(async move {
            match save_patient_anamnesis(&t, &p, req).await {
                Ok(_) => {
                    toast.set(Some("Anamnese salva com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao salvar anamnese: {}", e)));
                }
            }
            save_sig.set(false);
        });
    };

    rsx! {
        div { class: "anamnese-cards-container",
            // 1. Alergias Conhecidas
            div { class: "anamnese-card",
                h3 { class: "anamnese-card-title", "1. Alergias Conhecidas" }
                div { class: "anamnese-checkbox-row",
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_allergies_penicillin(),
                            onchange: move |e| form_allergies_penicillin.set(e.checked()),
                        }
                        span { "Penicilina / Antibióticos" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_allergies_dipyrone(),
                            onchange: move |e| form_allergies_dipyrone.set(e.checked()),
                        }
                        span { "Dipirona / Anti-inflamatórios" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_allergies_latex(),
                            onchange: move |e| form_allergies_latex.set(e.checked()),
                        }
                        span { "Látex" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_allergies_anesthetic(),
                            onchange: move |e| form_allergies_anesthetic.set(e.checked()),
                        }
                        span { "Anestésicos Locais" }
                    }
                }
            }

            // 2. Doenças Sistêmicas & Condições Especiais
            div { class: "anamnese-card",
                h3 { class: "anamnese-card-title", "2. Doenças Sistêmicas & Condições Especiais" }
                div { class: "anamnese-checkbox-row",
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_disease_diabetes(),
                            onchange: move |e| form_disease_diabetes.set(e.checked()),
                        }
                        span { "Diabetes" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_disease_hypertension(),
                            onchange: move |e| form_disease_hypertension.set(e.checked()),
                        }
                        span { "Hipertensão Arterial" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_disease_cardiac(),
                            onchange: move |e| form_disease_cardiac.set(e.checked()),
                        }
                        span { "Cardiopatia" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_has_bleeding(),
                            onchange: move |e| form_has_bleeding.set(e.checked()),
                        }
                        span { "Distúrbio Hemorrágico / Sangramento" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_is_pregnant(),
                            onchange: move |e| form_is_pregnant.set(e.checked()),
                        }
                        span { "Gestante" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_smoker(),
                            onchange: move |e| form_smoker.set(e.checked()),
                        }
                        span { "Fumante" }
                    }
                    label { class: "anamnese-checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: form_bruxism(),
                            onchange: move |e| form_bruxism.set(e.checked()),
                        }
                        span { "Bruxismo / Apertamento Dental" }
                    }
                }
            }

            // 3. Medicamentos de Uso Contínuo
            div { class: "anamnese-card",
                h3 { class: "anamnese-card-title", "3. Medicamentos de Uso Contínuo" }
                input {
                    class: "anamnese-input-field",
                    placeholder: "Ex: Losartana 50mg, Metformina 850mg, Anticoagulantes...",
                    value: "{form_medications}",
                    oninput: move |e| form_medications.set(e.value()),
                }
            }

            // 4. Queixa Principal e Observações Clínicas
            div { class: "anamnese-card",
                h3 { class: "anamnese-card-title", "4. Queixa Principal e Observações Clínicas" }
                div { class: "form-group",
                    label { class: "text-xs font-semibold text-muted mb-1 block", "Queixa Principal relatada pelo Paciente" }
                    textarea {
                        class: "anamnese-input-field",
                        style: "min-height: 80px; resize: vertical;",
                        placeholder: "Descreva a queixa relatada pelo paciente, histórico de dor ou objetivo da consulta...",
                        value: "{form_chief_complaint}",
                        oninput: move |e| form_chief_complaint.set(e.value()),
                    }
                }
            }

            if can_write {
                div { class: "flex justify-end mt-2",
                    button {
                        class: "btn-primary",
                        disabled: is_saving(),
                        onclick: move |e| handle_save(e),
                        IconHeartPulse { size: 16, color: "#ffffff".to_string() }
                        span { if is_saving() { " Salvando Alterações..." } else { " Salvar Alterações na Anamnese" } }
                    }
                }
            }
        }
    }
}
