use crate::icons::{IconClose, IconTrash};
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
    let time_display = format!("{:02}:{:02}", h, m);
    let day_name = app.scheduled_for.split('T').next().unwrap_or(&app.scheduled_for);

    let pop_x = (x - 160.0).max(10.0);
    let pop_y = (y + 10.0).max(10.0);

    let status_val = match app.status {
        AppointmentStatus::Confirmed => "confirmed",
        AppointmentStatus::Completed => "completed",
        AppointmentStatus::InProgress => "in_progress",
        AppointmentStatus::Pending => "pending",
        AppointmentStatus::Canceled | AppointmentStatus::CanceledByDoctor | AppointmentStatus::CanceledByPatient => "canceled",
        AppointmentStatus::NoShow => "no_show",
    };

    rsx! {
        div {
            class: "event-popover-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "event-popover",
                style: format!("left: {}px; top: {}px;", pop_x, pop_y),
                onclick: move |e| e.stop_propagation(),

                div { class: "popover-header",
                    div { class: "popover-avatar", "👤" }
                    div { class: "popover-patient-info",
                        div { class: "popover-patient-name", "{patient_name}" }
                        div { class: "popover-datetime", "{day_name} • {time_display}" }
                    }
                    div { class: "popover-actions",
                        button {
                            class: "popover-action-btn danger",
                            title: "Cancelar agendamento",
                            onclick: move |_| on_cancel.call(()),
                            IconTrash { size: 14, color: "currentColor".to_string() }
                        }
                        button {
                            class: "popover-action-btn",
                            title: "Fechar",
                            onclick: move |_| on_close.call(()),
                            IconClose { size: 14, color: "currentColor".to_string() }
                        }
                    }
                }

                div { class: "popover-body",
                    select {
                        class: "popover-status-select",
                        value: "{status_val}",
                        onchange: move |e| on_change_status.call(e.value()),
                        option { value: "confirmed", "Confirmado" }
                        option { value: "in_progress", "Em Atendimento" }
                        option { value: "pending", "Pendente" }
                        option { value: "completed", "Concluído" }
                        option { value: "canceled", "Cancelado" }
                    }
                }
            }
        }
    }
}
