use crate::icons::IconClose;
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn PatientDetailsModal(
    patient: Patient,
    active_tab: Signal<String>,
    on_close: EventHandler<()>,
) -> Element {
    let initial = patient.full_name.chars().next().unwrap_or('P').to_string();

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "modal-box patient-details-box",
                onclick: move |e| e.stop_propagation(),

                div { class: "patient-profile-header",
                    div { class: "patient-profile-avatar", "{initial}" }
                    div { class: "patient-profile-info",
                        h2 { "{patient.full_name}" }
                        div { class: "patient-profile-pills",
                            span { "📞 {patient.phone}" }
                            if let Some(ref email) = patient.email {
                                span { "✉️ {email}" }
                            }
                            if let Some(ref cpf) = patient.document_cpf {
                                span { "🆔 CPF: {cpf}" }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "modal-close-btn",
                        onclick: move |_| on_close.call(()),
                        IconClose { size: 18, color: "currentColor".to_string() }
                    }
                }

                div { class: "tab-underline-bar",
                    button {
                        class: if *active_tab.read() == "info" { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| active_tab.set("info".to_string()),
                        "Dados Cadastrais"
                    }
                    button {
                        class: if *active_tab.read() == "anamnese" { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| active_tab.set("anamnese".to_string()),
                        "Anamnese & Alertas"
                    }
                    button {
                        class: if *active_tab.read() == "procedimentos" { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| active_tab.set("procedimentos".to_string()),
                        "Procedimentos & Evolução"
                    }
                }

                div { class: "patient-details-content",
                    if *active_tab.read() == "info" {
                        div { class: "patient-info-grid",
                            div { class: "info-item-box",
                                span { class: "info-item-label", "Data de Nascimento" }
                                span { class: "info-item-val", "{patient.birth_date.clone().unwrap_or_else(|| \"Não informado\".to_string())}" }
                            }
                            div { class: "info-item-box",
                                span { class: "info-item-label", "Gênero" }
                                span { class: "info-item-val", "{patient.gender.clone().unwrap_or_else(|| \"Não informado\".to_string())}" }
                            }
                            div { class: "info-item-box",
                                span { class: "info-item-label", "Plano Odontológico" }
                                span { class: "info-item-val", "{patient.insurance_plan.clone().unwrap_or_else(|| \"Particular\".to_string())}" }
                            }
                            div { class: "info-item-box",
                                span { class: "info-item-label", "Endereço" }
                                span { class: "info-item-val", "{patient.address_street.clone().unwrap_or_else(|| \"Endereço não cadastrado\".to_string())}" }
                            }
                        }
                    } else if *active_tab.read() == "anamnese" {
                        div { class: "patient-info-grid",
                            div { class: "info-item-box",
                                span { class: "info-item-label", "Status da Anamnese" }
                                div { class: "anamnesis-alert-row",
                                    span { class: "anamnesis-chip chip-success", "🩺 Questionário Cadastrado" }
                                }
                            }
                        }
                    } else {
                        div { style: "padding: 24px; text-align: center; color: #94a3b8;",
                            p { "Nenhum tratamento ou procedimento em andamento cadastrado para este paciente." }
                        }
                    }
                }
            }
        }
    }
}
