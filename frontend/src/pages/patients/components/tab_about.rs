use crate::icons::{IconCopy, IconInfo, IconMessageSquare};
use crate::router::Route;
use shared::appointments::AppointmentResponse;
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn TabAbout(
    patient: Patient,
    appointments: Vec<AppointmentResponse>,
) -> Element {
    let mut msg_service = use_signal(|| true);
    let mut msg_marketing = use_signal(|| true);
    let navigator = use_navigator();

    let gender_display = match patient.gender.as_deref() {
        Some("female") => "Feminino",
        Some("male") => "Masculino",
        _ => "Não informado",
    };

    let plan_display = patient.insurance_plan.clone().unwrap_or_else(|| "Particular".to_string());
    let has_cpf = patient.document_cpf.is_some();
    let patient_appts: Vec<AppointmentResponse> = appointments
        .into_iter()
        .filter(|a| a.patient_id.as_deref() == Some(&patient.id) || a.patient_name.as_deref() == Some(&patient.full_name))
        .collect();

    rsx! {
        div { class: "patient-tab-grid-2",
            // Coluna Esquerda
            div { style: "display: flex; flex-direction: column; gap: 16px;",
                // Card Dados Pessoais
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Dados pessoais" }
                    }
                    div { class: "patient-card-body",
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Número paciente" }
                            span { class: "info-data-val", "25" }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Sexo" }
                            span { class: "info-data-val", "{gender_display}" }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Celular" }
                            span { class: "info-data-val", "{patient.phone}" }
                        }
                        if let Some(cpf) = patient.document_cpf.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "CPF" }
                                span { class: "info-data-val", "{cpf}" }
                            }
                        }
                        if let Some(bd) = patient.birth_date.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "Data de Nascimento" }
                                span { class: "info-data-val", "{bd}" }
                            }
                        }
                    }
                }

                // Card Dados do Plano
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Dados do plano" }
                    }
                    div { class: "patient-card-body",
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Plano" }
                            span { class: "info-data-val", "{plan_display}" }
                        }
                    }
                }

                // Card Aplicativo Meu Doutor
                div { class: "patient-card",
                    div { class: "patient-card-body",
                        h3 { style: "font-size: 15px; font-weight: 700; color: #f8fafc; margin: 0 0 6px 0;", "Código para o aplicativo Meu Doutor" }
                        p { style: "font-size: 13px; color: #94a3b8; line-height: 1.45; margin: 0;",
                            "Oriente o paciente a baixar o aplicativo "
                            strong { style: "color: #f1f5f9;", "Meu Doutor" }
                            " e envie o código abaixo para o seu paciente entrar no aplicativo."
                        }

                        div { class: "app-doctor-code-row",
                            div { class: "app-doctor-code-box", if has_cpf { "DOU-8492" } else { "------" } }
                            button {
                                r#type: "button",
                                class: "btn-invite-pill",
                                IconCopy { size: 14, color: "currentColor".to_string() }
                                span { "CONVIDAR" }
                            }
                        }

                        if !has_cpf {
                            div { class: "app-alert-warning-box",
                                span { "⚠️ Paciente " }
                                strong { "sem CPF cadastrado" }
                                span { ". Para gerar o código, é necessário informar o CPF do paciente na ficha." }
                            }
                        }
                    }
                }
            }

            // Coluna Direita
            div { style: "display: flex; flex-direction: column; gap: 16px;",
                // Card Consultas
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Consultas" }
                    }
                    div { class: "patient-card-body",
                        if patient_appts.is_empty() {
                            p { style: "font-size: 13px; color: #94a3b8; margin: 0;", "Nenhuma consulta agendada para este paciente." }
                        } else {
                            for app in patient_appts {
                                {
                                    let formatted_time = app.scheduled_for.replace('T', " às ").replace(":00Z", "");
                                    rsx! {
                                        div { key: "{app.id}", style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid rgba(255,255,255,0.05);",
                                            div {
                                                div { style: "font-weight: 700; color: #f1f5f9; font-size: 13px;", "{formatted_time}" }
                                                div { style: "font-size: 12px; color: #94a3b8;", "Dr. Lucas Mendes • {app.title}" }
                                            }
                                            div { style: "display: flex; align-items: center; gap: 12px;",
                                                span { style: "font-size: 12px; color: #38bdf8; font-weight: 600;", "Agendada" }
                                                button {
                                                    r#type: "button",
                                                    class: "btn-primary",
                                                    style: "padding: 4px 10px; font-size: 11px;",
                                                    onclick: move |_| {
                                                        navigator.push(Route::AgendaView {});
                                                    },
                                                    "VER NA AGENDA"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Card Mensagens
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Mensagens" }
                    }
                    div { class: "patient-card-body", style: "padding: 32px 18px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 8px;",
                        IconMessageSquare { size: 32, color: "#475569".to_string() }
                        span { style: "font-size: 13px; color: #94a3b8;", "Você ainda não enviou nenhuma mensagem para este paciente" }
                    }
                }

                // Card Comunicação
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Comunicação" }
                    }
                    div { class: "patient-card-body",
                        p { style: "font-size: 12.5px; color: #64748b; margin: 0 0 6px 0;", "Permitir o envio de:" }

                        div { class: "switch-toggle-row",
                            div { class: "switch-label-group",
                                span { "Mensagens relacionadas ao serviço prestado" }
                                IconInfo { size: 14, color: "#64748b".to_string() }
                            }
                            label { class: "switch-input-custom",
                                input {
                                    r#type: "checkbox",
                                    checked: "{msg_service}",
                                    onchange: move |e| msg_service.set(e.checked()),
                                }
                                span { class: "switch-slider" }
                            }
                        }

                        div { class: "switch-toggle-row",
                            div { class: "switch-label-group",
                                span { "Campanha de marketing" }
                                IconInfo { size: 14, color: "#64748b".to_string() }
                            }
                            label { class: "switch-input-custom",
                                input {
                                    r#type: "checkbox",
                                    checked: "{msg_marketing}",
                                    onchange: move |e| msg_marketing.set(e.checked()),
                                }
                                span { class: "switch-slider" }
                            }
                        }
                    }
                }
            }
        }
    }
}
