use crate::icons::IconInfo;
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

    let marital_display = match patient.marital_status.as_deref() {
        Some("single") => "Solteiro(a)",
        Some("married") => "Casado(a)",
        Some("divorced") => "Divorciado(a)",
        Some("widowed") => "Viúvo(a)",
        _ => "Não informado",
    };

    let plan_display = patient.insurance_plan.clone().unwrap_or_else(|| "Particular".to_string());
    let insurance_num = patient.insurance_number.clone().unwrap_or_else(|| "—".to_string());

    let address_str = match (&patient.address_street, &patient.address_number, &patient.address_city) {
        (Some(street), Some(num), Some(city)) => format!("{}, {} - {}, {}", street, num, patient.address_neighborhood.as_deref().unwrap_or(""), city),
        (Some(street), Some(num), None) => format!("{}, {}", street, num),
        _ => "Endereço não cadastrado".to_string(),
    };

    let emergency_str = match (&patient.emergency_contact_name, &patient.emergency_contact_phone) {
        (Some(name), Some(ph)) => format!("{} ({})", name, ph),
        (Some(name), None) => name.clone(),
        _ => "Não informado".to_string(),
    };

    let patient_appts: Vec<AppointmentResponse> = appointments
        .into_iter()
        .filter(|a| a.patient_id.as_deref() == Some(&patient.id) || a.patient_name.as_deref() == Some(&patient.full_name))
        .collect();

    rsx! {
        div { class: "patient-tab-grid-2",
            div { style: "display: flex; flex-direction: column; gap: 16px;",
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Dados Pessoais" }
                    }
                    div { class: "patient-card-body",
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Nome completo" }
                            span { class: "info-data-val", "{patient.full_name}" }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Sexo" }
                            span { class: "info-data-val", "{gender_display}" }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Celular" }
                            span { class: "info-data-val", "{patient.phone}" }
                        }
                        if let Some(email) = patient.email.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "E-mail" }
                                span { class: "info-data-val", "{email}" }
                            }
                        }
                        if let Some(cpf) = patient.document_cpf.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "CPF" }
                                span { class: "info-data-val", "{cpf}" }
                            }
                        }
                        if let Some(rg) = patient.document_rg.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "RG" }
                                span { class: "info-data-val", "{rg}" }
                            }
                        }
                        if let Some(bd) = patient.birth_date.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "Nascimento" }
                                span { class: "info-data-val", "{bd}" }
                            }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Estado civil" }
                            span { class: "info-data-val", "{marital_display}" }
                        }
                        if let Some(prof) = patient.profession.as_ref() {
                            div { class: "info-data-row",
                                span { class: "info-data-label", "Profissão" }
                                span { class: "info-data-val", "{prof}" }
                            }
                        }
                    }
                }

                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Endereço & Contato de Emergência" }
                    }
                    div { class: "patient-card-body",
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Endereço" }
                            span { class: "info-data-val", "{address_str}" }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Emergência" }
                            span { class: "info-data-val", "{emergency_str}" }
                        }
                    }
                }

                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Dados do Plano Odontológico" }
                    }
                    div { class: "patient-card-body",
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Plano / Convênio" }
                            span { class: "info-data-val", "{plan_display}" }
                        }
                        div { class: "info-data-row",
                            span { class: "info-data-label", "Carteirinha" }
                            span { class: "info-data-val", "{insurance_num}" }
                        }
                    }
                }
            }

            div { style: "display: flex; flex-direction: column; gap: 16px;",
                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Consultas & Atendimentos" }
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

                div { class: "patient-card",
                    div { class: "patient-card-header",
                        h3 { class: "patient-card-title", "Preferências de Comunicação" }
                    }
                    div { class: "patient-card-body",
                        p { style: "font-size: 12.5px; color: #64748b; margin: 0 0 6px 0;", "Permitir o envio de notificações:" }

                        div { class: "switch-toggle-row",
                            div { class: "switch-label-group",
                                span { "Lembretes de consulta e retorno" }
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
                                span { "Campanhas de orientação preventiva" }
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
