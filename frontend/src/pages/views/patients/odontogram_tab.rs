//! # Aba de Tratamentos e Odontograma do Paciente (Frontend)
//!
//! Controla o histórico de procedimentos clínicos realizados, dente/região tratada,
//! custos em Reais e modal para lançamento de novos procedimentos.

use crate::api::create_patient_treatment;
use crate::components::icons::{IconCheckCircle, IconTooth};
use dioxus::prelude::*;
use shared::patients::{CreatePatientTreatmentRequest, PatientTreatment};

/// Componente da aba de Tratamentos e Procedimentos do Paciente.
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

    // Form state
    let mut form_procedure_name = use_signal(String::new);
    let mut form_tooth = use_signal(String::new);
    let mut form_status = use_signal(|| "planned".to_string());
    let mut form_cost = use_signal(|| "0,00".to_string());
    let mut form_notes = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let pat_id = patient_id.clone();
    let cid = clinic_id.clone();
    let tok = token.clone();

    let mut handle_submit = move |_| {
        let name = form_procedure_name().trim().to_string();
        if name.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome do procedimento/tratamento.".into()));
            return;
        }

        let cost_clean = form_cost().replace("R$", "").replace(".", "").replace(",", "").trim().to_string();
        let cost_cents = cost_clean.parse::<i64>().unwrap_or(0);

        let req = CreatePatientTreatmentRequest {
            clinic_id: cid.clone(),
            dentist_user_id: None,
            appointment_id: None,
            procedure_name: name,
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
        let reload = reload_patient_details.clone();

        sub_sig.set(true);
        spawn(async move {
            match create_patient_treatment(&t, &p, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Procedimento adicionado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao cadastrar tratamento: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "patient-tab-content",
            div { class: "tab-actions-header",
                div {
                    h3 { class: "tab-title", "Histórico de Tratamentos & Procedimentos" }
                    p { class: "tab-subtitle", "Evolução clínica, dentes/regiões tratadas e custos." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| is_add_modal_open.set(true),
                        IconTooth { size: 16, color: "currentColor".to_string() }
                        span { "Novo Procedimento" }
                    }
                }
            }

            if treatments.is_empty() {
                div { class: "empty-tab-state",
                    IconTooth { size: 48, color: "var(--text-muted, #8c8c8c)".to_string() }
                    p { class: "empty-state-text", "Nenhum procedimento registrado para este paciente." }
                }
            } else {
                div { class: "table-responsive",
                    table { class: "data-table",
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
                                    let cost_brl = format!("R$ {:.2}", (treat.cost_cents as f64) / 100.0);
                                    let status_badge_class = match treat.status.as_str() {
                                        "completed" => "badge-success",
                                        "in_progress" => "badge-primary",
                                        "cancelled" => "badge-danger",
                                        _ => "badge-warning",
                                    };
                                    let status_label = match treat.status.as_str() {
                                        "completed" => "Concluído",
                                        "in_progress" => "Em Andamento",
                                        "cancelled" => "Cancelado",
                                        _ => "Planejado",
                                    };

                                    rsx! {
                                        tr { key: "{treat.id}",
                                            td { class: "font-mono font-xs", "{dt}" }
                                            td { strong { "{treat.procedure_name}" } }
                                            td {
                                                if let Some(ref tooth) = treat.tooth_number {
                                                    span { class: "badge-outline", "Dente {tooth}" }
                                                } else {
                                                    span { class: "text-muted", "Geral" }
                                                }
                                            }
                                            td {
                                                span { class: "{status_badge_class}", "{status_label}" }
                                            }
                                            td { class: "font-mono font-weight-bold", "{cost_brl}" }
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

            if is_add_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Registrar Novo Procedimento" }
                                p { class: "modal-subtitle", "Adicione procedimentos planejados ou concluídos no mapa dental do paciente." }
                            }
                            button { class: "modal-close", onclick: move |_| is_add_modal_open.set(false), "×" }
                        }
                        div { class: "modal-body",
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
                                        option { value: "planned", "Planejado" }
                                        option { value: "in_progress", "Em Andamento" }
                                        option { value: "completed", "Concluído" }
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { "Valor (R$)" }
                                input {
                                    class: "form-input",
                                    placeholder: "0,00",
                                    value: "{form_cost}",
                                    oninput: move |e| form_cost.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Observações Clínicas" }
                                textarea {
                                    class: "form-textarea",
                                    placeholder: "Ex: Procedimento realizado sob anestesia infiltrativa...",
                                    value: "{form_notes}",
                                    oninput: move |e| form_notes.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer",
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
