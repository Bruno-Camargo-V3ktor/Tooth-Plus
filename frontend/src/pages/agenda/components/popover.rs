use crate::icons::{IconClose, IconCopy, IconEdit, IconExternalLink, IconInfo, IconUser};
use crate::router::Route;
use shared::appointments::{AppointmentResponse, AppointmentStatus};
use dioxus::prelude::*;

#[component]
pub fn AppointmentPopover(
    app: AppointmentResponse,
    x: f64,
    y: f64,
    on_close: EventHandler<()>,
    on_change_status: EventHandler<String>,
    on_cancel: EventHandler<()>,
    on_copy: EventHandler<AppointmentResponse>,
    on_edit: EventHandler<AppointmentResponse>,
) -> Element {
    let mut selected_patient_id = consume_context::<Signal<Option<String>>>();
    let nav = navigator();

    let patient_name = app.patient_name.clone().unwrap_or_else(|| app.title.clone());
    let (h, m) = super::event_card::extract_hhmm(&app.scheduled_for);
    let end_m = m + (app.duration_minutes.max(0) as u32);
    let end_h = h + (end_m / 60);
    let end_m = end_m % 60;
    let time_display = format!("{:02}:{:02} - {:02}:{:02}", h, m, end_h, end_m);

    let doc_name = app.assigned_users.first().and_then(|u| u.user_name.clone()).unwrap_or_else(|| "Dr(a). Lucas Mendes".to_string());
    let doc_initials = doc_name.split_whitespace().take(2).map(|w| w.chars().next().unwrap_or('D')).collect::<String>();

    let pop_x = (x - 170.0).max(10.0);
    let pop_y = (y + 10.0).max(10.0);

    let (status_val, header_bg) = match app.status {
        AppointmentStatus::Confirmed => ("confirmed", "#1e3a5f"),
        AppointmentStatus::Completed => ("completed", "#14532d"),
        AppointmentStatus::InProgress => ("in_progress", "#0369a1"),
        AppointmentStatus::Waiting => ("waiting", "#475569"),
        AppointmentStatus::Pending => ("pending", "#334155"),
        AppointmentStatus::NoShow => ("no_show", "#7f1d1d"),
        AppointmentStatus::CanceledByPatient => ("canceled_pat", "#450a0a"),
        AppointmentStatus::CanceledByDoctor => ("canceled_doc", "#450a0a"),
        AppointmentStatus::Canceled => ("canceled", "#450a0a"),
    };

    let p_phone = "+55 11 98765-4321";
    let p_id_opt = app.patient_id.clone();

    let app_copy = app.clone();
    let app_edit = app.clone();

    rsx! {
        div {
            class: "event-popover-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "event-popover",
                style: format!("left: {}px; top: {}px; width: 330px; border-radius: 12px; overflow: hidden; box-shadow: 0 16px 40px rgba(0,0,0,0.7);", pop_x, pop_y),
                onclick: move |e| e.stop_propagation(),

                // Top Header com a cor do status
                div {
                    style: format!("background: {}; padding: 14px 16px; color: #ffffff; position: relative;", header_bg),

                    div { style: "display: flex; justify-content: flex-end; gap: 8px; margin-bottom: 8px;",
                        button {
                            r#type: "button",
                            class: "action-btn-icon",
                            style: "color: rgba(255,255,255,0.8); background: rgba(0,0,0,0.2); width: 26px; height: 26px;",
                            title: "Copiar dados do agendamento",
                            onclick: move |_| on_copy.call(app_copy.clone()),
                            IconCopy { size: 13, color: "#ffffff".to_string() }
                        }
                        button {
                            r#type: "button",
                            class: "action-btn-icon",
                            style: "color: rgba(255,255,255,0.8); background: rgba(0,0,0,0.2); width: 26px; height: 26px;",
                            title: "Editar agendamento",
                            onclick: move |_| on_edit.call(app_edit.clone()),
                            IconEdit { size: 13, color: "#ffffff".to_string() }
                        }
                        button {
                            r#type: "button",
                            class: "action-btn-icon",
                            style: "color: rgba(255,255,255,0.8); background: rgba(0,0,0,0.2); width: 26px; height: 26px;",
                            title: "Fechar",
                            onclick: move |_| on_close.call(()),
                            IconClose { size: 14, color: "#ffffff".to_string() }
                        }
                    }

                    div { style: "display: flex; align-items: center; gap: 12px;",
                        div {
                            style: "width: 40px; height: 40px; border-radius: 50%; background: #ffffff; display: flex; align-items: center; justify-content: center; color: #0c1222; flex-shrink: 0;",
                            IconUser { size: 22, color: "#0c1222".to_string() }
                        }
                        div { style: "flex: 1; min-width: 0;",
                            div { style: "display: flex; align-items: center; gap: 6px;",
                                span {
                                    style: "font-size: 15px; font-weight: 800; color: #ffffff; cursor: pointer; display: flex; align-items: center; gap: 4px;",
                                    onclick: move |_| {
                                        if let Some(ref pid) = p_id_opt {
                                            selected_patient_id.set(Some(pid.clone()));
                                            nav.push(Route::PatientsView {});
                                        }
                                    },
                                    span { "{patient_name}" }
                                    IconExternalLink { size: 13, color: "rgba(255,255,255,0.8)".to_string() }
                                }
                            }
                            div { style: "font-size: 12px; color: rgba(255,255,255,0.85); margin-top: 2px;",
                                "{p_phone}"
                            }
                            div { style: "font-size: 11.5px; color: rgba(255,255,255,0.7); margin-top: 2px;",
                                "Hoje • {time_display}"
                            }
                        }
                    }
                }

                // Body do Popover
                div { style: "background: #182033; padding: 14px 16px; display: flex; flex-direction: column; gap: 10px;",
                    // Status selector
                    div { class: "form-field", style: "margin: 0;",
                        select {
                            class: "form-select",
                            style: "height: 38px; font-weight: 700;",
                            value: "{status_val}",
                            onchange: move |e| on_change_status.call(e.value()),
                            option { value: "pending", "Agendado" }
                            option { value: "confirmed", "Confirmada" }
                            option { value: "waiting", "Paciente aguardando" }
                            option { value: "in_progress", "Em atendimento" }
                            option { value: "completed", "Finalizada" }
                            option { value: "no_show", "Falta" }
                            option { value: "canceled_pat", "Cancelado pelo paciente" }
                            option { value: "canceled_doc", "Cancelado pelo profissional" }
                        }
                    }

                    // Profissional
                    div { style: "display: flex; align-items: center; gap: 8px; font-size: 13px; color: #cbd5e1;",
                        div { style: "width: 22px; height: 22px; border-radius: 50%; background: #0284c7; color: #ffffff; font-size: 9px; font-weight: 800; display: flex; align-items: center; justify-content: center;",
                            "{doc_initials}"
                        }
                        span { style: "flex: 1; font-weight: 600;", "{doc_name}" }
                        IconInfo { size: 15, color: "#64748b".to_string() }
                    }

                    // Rótulo da consulta
                    input {
                        class: "form-input",
                        style: "font-size: 12.5px; height: 34px;",
                        placeholder: "Rótulo da consulta...",
                        value: if let Some(ref note) = app.notes { "{note}" } else { "" },
                    }
                }
            }
        }
    }
}
