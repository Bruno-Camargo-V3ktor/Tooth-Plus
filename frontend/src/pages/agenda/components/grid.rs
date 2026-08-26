use shared::appointments::AppointmentResponse;
use super::event_card::EventCard;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct DayColumn {
    pub name: String,
    pub num: String,
    pub date_str: String,
    pub is_today: bool,
}

#[component]
pub fn AgendaGrid(
    view_mode: String,
    days: Vec<DayColumn>,
    open_hour: u32,
    close_hour: u32,
    appointments: Vec<AppointmentResponse>,
    dentist_filter: String,
    on_slot_click: EventHandler<(String, u32)>,
    on_event_click: EventHandler<(f64, f64, String)>,
) -> Element {
    let grid_cls = if view_mode == "day" { "agenda-grid agenda-grid-day" } else { "agenda-grid" };

    rsx! {
        div { class: "agenda-grid-scroll",
            div { class: "{grid_cls}",
                // Cabeçalho dos Dias
                div { class: "agenda-header-time-spacer" }
                for d in days.iter() {
                    div {
                        key: "{d.date_str}",
                        class: if d.is_today { "agenda-header-day today-col" } else { "agenda-header-day" },
                        div { class: "agenda-header-day-name", "{d.name}" }
                        div {
                            class: if d.is_today { "agenda-header-day-num today-num" } else { "agenda-header-day-num" },
                            "{d.num}"
                        }
                    }
                }

                // Linhas de Horas
                for h in open_hour..=close_hour {
                    div { key: "{h}-label", class: "agenda-time-label", "{h:02}h00" }

                    for d in days.iter() {
                        {
                            let d_date = d.date_str.clone();
                            let is_today = d.is_today;
                            let slot_apps: Vec<AppointmentResponse> = appointments
                                .iter()
                                .filter(|a| {
                                    if !a.scheduled_for.starts_with(&d_date) { return false; }
                                    if dentist_filter != "all" {
                                        let assigned = a.assigned_users.iter().any(|u| u.user_id == dentist_filter);
                                        if !assigned { return false; }
                                    }
                                    let (ah, _) = super::event_card::extract_hhmm(&a.scheduled_for);
                                    ah == h
                                })
                                .cloned()
                                .collect();

                            rsx! {
                                div {
                                    key: "{d_date}-{h}",
                                    class: if is_today { "agenda-cell today-col" } else { "agenda-cell" },
                                    onclick: move |_| on_slot_click.call((d_date.clone(), h)),

                                    for (idx, app) in slot_apps.into_iter().enumerate() {
                                        EventCard {
                                            key: "{app.id}",
                                            app,
                                            idx,
                                            on_click: move |coords| on_event_click.call(coords),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
