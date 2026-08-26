use crate::icons::{IconClose, IconCopy, IconEdit, IconExternalLink, IconInfo, IconWhatsapp};
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
) -> Element {
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
        AppointmentStatus::Pending => ("pending", "#78350f"),
        AppointmentStatus::NoShow => ("no_show", "#7f1d1d"),
        AppointmentStatus::Canceled | AppointmentStatus::CanceledByDoctor | AppointmentStatus::CanceledByPatient => ("canceled", "#334155"),
    };

    let p_phone = "+55 11 98765-4321";
    let encoded_msg = format!("Olá {}, confirmamos seu agendamento para hoje às {} com {}.", patient_name, time_display, doc_name);
    let wa_link = format!("https://web.whatsapp.com/send?phone=5511987654321&text={}", encoded_msg);

    rsx! {
        div {
            class: "event-popover-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "event-popover",
                style: format!("left: {}px; top: {}px; width: 340px; border-radius: 12px; overflow: hidden; box-shadow: 0 16px 40px rgba(0,0,0,0.7);", pop_x, pop_y),
                onclick: move |e| e.stop_propagation(),

                // Top Header com a cor do status
                div {
                    style: format!("background: {}; padding: 16px; color: #ffffff; position: relative;", header_bg),

                    div { style: "display: flex; justify-content: flex-end; gap: 8px; margin-bottom: 8px;",
                        button {
                            r#type: "button",
                            class: "action-btn-icon",
                            style: "color: rgba(255,255,255,0.8); background: rgba(0,0,0,0.2); width: 26px; height: 26px;",
                            title: "Copiar dados do agendamento",
                            IconCopy { size: 13, color: "#ffffff".to_string() }
                        }
                        button {
                            r#type: "button",
                            class: "action-btn-icon",
                            style: "color: rgba(255,255,255,0.8); background: rgba(0,0,0,0.2); width: 26px; height: 26px;",
                            title: "Editar agendamento",
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
                            style: "width: 44px; height: 44px; border-radius: 50%; background: #ffffff; display: flex; align-items: center; justify-content: center; color: #0c1222; font-size: 20px; flex-shrink: 0; box-shadow: 0 2px 8px rgba(0,0,0,0.2);",
                            "👤"
                        }
                        div { style: "flex: 1; min-width: 0;",
                            div { style: "display: flex; align-items: center; gap: 6px;",
                                Link {
                                    to: Route::PatientsView {},
                                    style: "font-size: 15px; font-weight: 800; color: #ffffff; text-decoration: none; display: flex; align-items: center; gap: 4px;",
                                    span { "{patient_name}" }
                                    IconExternalLink { size: 13, color: "rgba(255,255,255,0.8)".to_string() }
                                }
                            }
                            div { style: "font-size: 12.5px; color: rgba(255,255,255,0.85); margin-top: 2px;",
                                "{p_phone}"
                            }
                            div { style: "font-size: 11.5px; color: rgba(255,255,255,0.7); margin-top: 2px;",
                                "Hoje • {time_display}"
                            }
                        }
                    }
                }

                // Body do Popover
                div { style: "background: #182033; padding: 16px; display: flex; flex-direction: column; gap: 12px;",
                    // Status selector
                    div { class: "form-field", style: "margin: 0;",
                        select {
                            class: "form-select",
                            style: "height: 38px; font-weight: 700;",
                            value: "{status_val}",
                            onchange: move |e| on_change_status.call(e.value()),
                            option { value: "confirmed", "📅 Confirmado" }
                            option { value: "in_progress", "📅 Em Atendimento" }
                            option { value: "pending", "📅 Aguardando" }
                            option { value: "completed", "📅 Finalizado" }
                            option { value: "no_show", "📅 Falta" }
                            option { value: "canceled", "📅 Desmarcado" }
                        }
                    }

                    // Botão Reagendar por WhatsApp Web
                    a {
                        href: "{wa_link}",
                        target: "_blank",
                        style: "display: flex; align-items: center; justify-content: center; gap: 8px; background: #0f4a30; border: 1px solid #16a34a; border-radius: 6px; padding: 9px 12px; color: #86efac; font-size: 13px; font-weight: 700; text-decoration: none; transition: background 0.15s ease;",
                        IconWhatsapp { size: 16, color: "#86efac".to_string() }
                        span { "Reagendar por WhatsApp Web" }
                        IconEdit { size: 13, color: "#86efac".to_string() }
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
                        placeholder: "Rótulo da consulta (ex: Avaliação inicial...)",
                        value: if let Some(ref note) = app.notes { "{note}" } else { "" },
                    }
                }
            }
        }
    }
}
