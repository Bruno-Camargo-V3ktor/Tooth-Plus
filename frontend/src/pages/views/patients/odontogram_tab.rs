//! # Aba de Histórico de Procedimentos e Tratamentos Odontológicos (Frontend)
//!
//! Controla os procedimentos realizados, planejados ou em andamento,
//! com registro de dente/região, valores e observações clínicas.

use crate::api::create_patient_treatment;
use crate::components::icons::IconTooth;
use dioxus::prelude::*;
use shared::patients::{CreatePatientTreatmentRequest, PatientTreatment};

/// Formata valor em centavos para moeda BRL.
fn format_currency(cents: i64) -> String {
    let reals = cents / 100;
    let centavos = cents % 100;
    format!("R$ {},{:02}", reals, centavos)
}

/// Componente da aba de procedimentos e tratamentos odontológicos.
#[component]
pub fn PatientOdontogramTab(
    patient_id: String,
    clinic_id: String,
    token: String,
    treatments: Vec<PatientTreatment>,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_add_modal_open = use_signal(|| false);

    let mut form_procedure_name = use_signal(String::new);
    let mut form_tooth = use_signal(String::new);
    let mut form_status = use_signal(|| "completed".to_string());
    let mut form_cost = use_signal(String::new);
    let mut form_notes = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();
    let on_reload = reload_patient_details.clone();

    let mut handle_submit = move |_| {
        let proc_name = form_procedure_name().trim().to_string();
        if proc_name.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome do procedimento realizado.".into()));
            return;
        }

        let cost_clean = form_cost().trim().replace(',', ".").replace("R$", "").replace(' ', "");
        let cost_cents = if let Ok(val) = cost_clean.parse::<f64>() {
            (val * 100.0).round() as i64
        } else {
            0
        };

        let req = CreatePatientTreatmentRequest {
            clinic_id: cid.clone(),
            dentist_user_id: None,
            appointment_id: None,
            procedure_name: proc_name,
            tooth_number: if form_tooth().trim().is_empty() { None } else { Some(form_tooth().trim().to_string()) },
            status: form_status(),
            cost_cents,
            clinical_notes: if form_notes().trim().is_empty() { None } else { Some(form_notes().trim().to_string()) },
        };

        let t = tok.clone();
        let p = pat_id.clone();
        let mut open_sig = is_add_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_submitting;
        let reload = on_reload.clone();

        sub_sig.set(true);
        spawn(async move {
            match create_patient_treatment(&t, &p, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Procedimento registrado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao registrar procedimento: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "patient-tab-content",
            div { class: "tab-header-actions-row",
                div { class: "tab-header-title-group",
                    h3 { class: "tab-header-title", "Histórico de Procedimentos e Evolução" }
                    p { class: "tab-header-desc", "Registro detalhado de intervenções clínicas odontológicas." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| is_add_modal_open.set(true),
                        IconTooth { size: 16, color: "#ffffff".to_string() }
                        span { " Registrar Procedimento" }
                    }
                }
            }

            if treatments.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconTooth { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum procedimento registrado" }
                    p { "Registre restaurações, procedimentos cirúrgicos, manutenções ortodônticas ou evoluções clínicas." }
                }
            } else {
                div { class: "table-container",
                    table { class: "modern-table",
                        thead {
                            tr {
                                th { "Data" }
                                th { "Procedimento" }
                                th { "Dente / Região" }
                                th { "Status" }
                                th { "Valor" }
                                th { "Observações" }
                            }
                        }
                        tbody {
                            for treat in &treatments {
                                {
                                    let dt = treat.created_at.chars().take(10).collect::<String>();
                                    let cost_brl = format_currency(treat.cost_cents);
                                    let is_completed = treat.status == "completed";
                                    let is_in_progress = treat.status == "in_progress";

                                    rsx! {
                                        tr { key: "{treat.id}",
                                            td { class: "font-mono font-xs", "{dt}" }
                                            td { strong { class: "text-dark", "{treat.procedure_name}" } }
                                            td {
                                                if let Some(ref tooth) = treat.tooth_number {
                                                    span { class: "badge-outline", "Dente {tooth}" }
                                                } else {
                                                    span { class: "text-muted", "Geral" }
                                                }
                                            }
                                            td {
                                                if is_completed {
                                                    span { class: "badge-completed", "Concluído" }
                                                } else if is_in_progress {
                                                    span { class: "badge-pending", "Em Andamento" }
                                                } else {
                                                    span { class: "badge-outline", "Planejado" }
                                                }
                                            }
                                            td { class: "font-mono font-bold", "{cost_brl}" }
                                            td { class: "text-muted font-xs",
                                                "{treat.clinical_notes.as_deref().unwrap_or(\"-\")}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal Aprimorado: Registrar Novo Procedimento
            if is_add_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal", style: "max-width: 600px;",
                        div { class: "settings-header",
                            div {
                                h2 { class: "settings-title", "Registrar Novo Procedimento" }
                                p { class: "text-muted font-xs mt-1",
                                    "Adicione procedimentos planejados ou concluídos no histórico do paciente."
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_add_modal_open.set(false), "×" }
                        }
                        div { class: "settings-content",
                            div { class: "form-group",
                                label { "Procedimento *" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Restauração em Resina Composta, Extração...",
                                    value: "{form_procedure_name}",
                                    oninput: move |e| form_procedure_name.set(e.value())
                                }
                            }
                            div { class: "form-grid-2",
                                div { class: "form-group",
                                    label { "Dente / Região" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: 16, 21, Arcada Superior",
                                        value: "{form_tooth}",
                                        oninput: move |e| form_tooth.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Status" }
                                    select {
                                        class: "form-input",
                                        value: "{form_status}",
                                        onchange: move |e| form_status.set(e.value()),
                                        option { value: "completed", "Concluído" }
                                        option { value: "in_progress", "Em Andamento" }
                                        option { value: "planned", "Planejado" }
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { "Valor (R$)" }
                                div { class: "currency-input-wrapper",
                                    span { class: "currency-prefix", "R$" }
                                    input {
                                        class: "form-input currency-input-field",
                                        placeholder: "0,00",
                                        value: "{form_cost}",
                                        oninput: move |e| form_cost.set(e.value())
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { "Observações Clínicas" }
                                textarea {
                                    class: "form-input",
                                    style: "min-height: 85px; resize: vertical;",
                                    placeholder: "Ex: Procedimento realizado sob anestesia infiltrativa, isolamento relativo...",
                                    value: "{form_notes}",
                                    oninput: move |e| form_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| is_add_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_submitting(),
                                onclick: move |e| handle_submit(e),
                                if is_submitting() { "Salvando..." } else { "Salvar Procedimento" }
                            }
                        }
                    }
                }
            }
        }
    }
}
