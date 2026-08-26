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

#[derive(Clone, PartialEq)]
pub struct DoctorCol {
    pub id: String,
    pub name: String,
    pub initials: String,
    pub color: String,
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
    let doctors = vec![
        DoctorCol { id: "usr:dra_fernanda".to_string(), name: "Fernanda".to_string(), initials: "FS".to_string(), color: "#d97706".to_string() },
        DoctorCol { id: "usr:dr_lucas".to_string(), name: "Lucas".to_string(), initials: "LG".to_string(), color: "#16a34a".to_string() },
        DoctorCol { id: "usr:dra_luria".to_string(), name: "Luria".to_string(), initials: "LE".to_string(), color: "#0284c7".to_string() },
    ];

    let is_day_view = view_mode == "day";

    rsx! {
        div { class: "agenda-grid-scroll",
            div { class: if is_day_view { "agenda-grid agenda-grid-day" } else { "agenda-grid" },
                // Header Time Spacer
                div { class: "agenda-header-time-spacer" }

                if is_day_view {
                    // Header Colunas por Dentista no Dia
                    for (i, doc) in doctors.iter().enumerate() {
                        {
                            let doc_c = doc.clone();
                            let d_date = days.first().map(|d| d.date_str.clone()).unwrap_or_default();
                            rsx! {
                                div { key: "{doc.id}", class: "agenda-header-day", style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 12px;",
                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                        if i == 0 {
                                            button {
                                                r#type: "button",
                                                style: "width: 22px; height: 22px; border-radius: 50%; background: #16a34a; color: #ffffff; font-size: 13px; font-weight: 700; border: none; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                                                "+"
                                            }
                                        }
                                        div {
                                            span { style: "font-size: 11px; color: #38bdf8; font-weight: 700;", "Qua. " }
                                            strong { style: "font-size: 14px; color: #38bdf8;", "26" }
                                            span { style: "font-size: 11px; color: #64748b;", " /08" }
                                        }
                                    }

                                    div { style: "display: flex; align-items: center; gap: 6px;",
                                        div { style: format!("width: 22px; height: 22px; border-radius: 50%; background: {}; color: #ffffff; font-size: 9.5px; font-weight: 800; display: flex; align-items: center; justify-content: center;", doc_c.color),
                                            "{doc_c.initials}"
                                        }
                                        span { style: "font-size: 12px; font-weight: 700; color: #38bdf8;", "{doc_c.name}" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Header Colunas dos 7 Dias
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
                }

                // Linha de "Dia inteiro"
                div { class: "agenda-time-label", style: "font-size: 10.5px; color: #64748b;", "Dia inteiro" }
                if is_day_view {
                    for doc in doctors.iter() {
                        div { key: "allday-{doc.id}", class: "agenda-cell", style: "height: 28px; background: rgba(255,255,255,0.01);" }
                    }
                } else {
                    for d in days.iter() {
                        div { key: "allday-{d.date_str}", class: "agenda-cell", style: "height: 28px; background: rgba(255,255,255,0.01);" }
                    }
                }

                // Linhas de Horas
                for h in open_hour..=close_hour {
                    div { key: "{h}-label", class: "agenda-time-label", "{h:02}h00" }

                    if is_day_view {
                        for doc in doctors.iter() {
                            {
                                let doc_id = doc.id.clone();
                                let d_date = days.first().map(|d| d.date_str.clone()).unwrap_or_default();
                                let is_blocked = h < 8 || h == 12 || h > 18;

                                let slot_apps: Vec<AppointmentResponse> = appointments
                                    .iter()
                                    .filter(|a| {
                                        let (ah, _) = super::event_card::extract_hhmm(&a.scheduled_for);
                                        ah == h && (a.assigned_users.iter().any(|u| u.user_id == doc_id) || doc_id == "usr:dr_lucas")
                                    })
                                    .cloned()
                                    .collect();

                                rsx! {
                                    div {
                                        key: "slot-{doc.id}-{h}",
                                        class: if is_blocked { "agenda-cell agenda-cell-blocked" } else { "agenda-cell" },
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
                    } else {
                        for d in days.iter() {
                            {
                                let d_date = d.date_str.clone();
                                let is_today = d.is_today;
                                let is_blocked = h < 8 || h == 12 || h > 18;

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
                                        class: if is_blocked { "agenda-cell agenda-cell-blocked" } else if is_today { "agenda-cell today-col" } else { "agenda-cell" },
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
}
