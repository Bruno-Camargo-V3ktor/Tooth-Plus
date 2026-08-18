//! # Visualização em Linha do Tempo e Filtros da Agenda (Frontend)
//!
//! Controla os controles de navegação por data (dia anterior/seguinte),
//! filtros por dentista, status e busca, além da renderização das consultas na grade horária.

use crate::components::icons::{
    IconCalendar, IconChevronLeft, IconChevronRight, IconClock, IconEdit, IconPlus, IconSearch,
    IconTrash,
};
use chrono::Datelike;
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, AppointmentType,
};

/// Barra de ferramentas superior da agenda (filtros e navegação de data).
#[component]
pub fn AgendaToolbar(
    selected_date: Signal<String>,
    search_query: Signal<String>,
    filter_member: Signal<String>,
    filter_status: Signal<String>,
    filter_type: Signal<String>,
    resources: AgendaResourcesResponse,
    can_write: bool,
    on_new_appointment: EventHandler<()>,
) -> Element {
    let handle_prev_day = move |_| {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&selected_date(), "%Y-%m-%d") {
            let prev = d - chrono::Duration::days(1);
            selected_date.set(prev.format("%Y-%m-%d").to_string());
        }
    };

    let handle_next_day = move |_| {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&selected_date(), "%Y-%m-%d") {
            let next = d + chrono::Duration::days(1);
            selected_date.set(next.format("%Y-%m-%d").to_string());
        }
    };

    let handle_today = move |_| {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        selected_date.set(today);
    };

    let date_label =
        if let Ok(parsed_d) = chrono::NaiveDate::parse_from_str(&selected_date(), "%Y-%m-%d") {
            let weekday_pt = match parsed_d.weekday() {
                chrono::Weekday::Mon => "Segunda-feira",
                chrono::Weekday::Tue => "Terça-feira",
                chrono::Weekday::Wed => "Quarta-feira",
                chrono::Weekday::Thu => "Quinta-feira",
                chrono::Weekday::Fri => "Sexta-feira",
                chrono::Weekday::Sat => "Sábado",
                chrono::Weekday::Sun => "Domingo",
            };
            format!("{} • {}", parsed_d.format("%d/%m/%Y"), weekday_pt)
        } else {
            selected_date()
        };

    rsx! {
        div { class: "agenda-controls-bar",
            div { class: "agenda-date-nav",
                button { class: "agenda-nav-btn", onclick: handle_prev_day, title: "Dia Anterior",
                    IconChevronLeft { size: 18, color: "currentColor".to_string() }
                }
                button { class: "agenda-today-btn", onclick: handle_today, "Hoje" }
                button { class: "agenda-nav-btn", onclick: handle_next_day, title: "Próximo Dia",
                    IconChevronRight { size: 18, color: "currentColor".to_string() }
                }
                div { class: "agenda-date-picker-wrapper",
                    IconCalendar { size: 16, color: "currentColor".to_string() }
                    input {
                        class: "agenda-date-picker-input",
                        r#type: "date",
                        value: "{selected_date}",
                        onchange: move |e| selected_date.set(e.value())
                    }
                    span { class: "agenda-date-display-label", "{date_label}" }
                }
            }

            div { class: "agenda-filters-group",
                div { class: "modern-search-bar",
                    div { class: "search-icon", IconSearch { size: 16, color: "currentColor".to_string() } }
                    input {
                        class: "search-input",
                        placeholder: "Filtrar por título ou paciente...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                }

                select {
                    class: "agenda-filter-select",
                    value: "{filter_member}",
                    onchange: move |e| filter_member.set(e.value()),
                    option { value: "all", "Todos os Profissionais" }
                    for member in &resources.team_members {
                        option { value: "{member.id}", "{member.name}" }
                    }
                }

                select {
                    class: "agenda-filter-select",
                    value: "{filter_status}",
                    onchange: move |e| filter_status.set(e.value()),
                    option { value: "all", "Todos os Status" }
                    option { value: "pending", "Pendente" }
                    option { value: "confirmed", "Confirmado" }
                    option { value: "in_progress", "Em Atendimento" }
                    option { value: "completed", "Concluído" }
                    option { value: "canceled", "Cancelado" }
                    option { value: "no_show", "Não Compareceu" }
                }

                select {
                    class: "agenda-filter-select",
                    value: "{filter_type}",
                    onchange: move |e| filter_type.set(e.value()),
                    option { value: "all", "Todos os Tipos" }
                    option { value: "consultation", "Consulta" }
                    option { value: "treatment", "Tratamento" }
                    option { value: "surgery", "Cirurgia" }
                    option { value: "return", "Retorno" }
                    option { value: "meeting", "Reunião" }
                    option { value: "other", "Outro" }
                }

                if can_write {
                    button {
                        class: "btn-primary btn-novo-agendamento",
                        onclick: move |_| on_new_appointment.call(()),
                        IconPlus { size: 16, color: "currentColor".to_string() }
                        span { " Novo Agendamento" }
                    }
                }
            }
        }
    }
}

/// Visualização da grade diária de atendimentos da clínica.
#[component]
pub fn DayTimelineView(
    appointments: Vec<AppointmentResponse>,
    can_write: bool,
    can_delete: bool,
    on_slot_click: EventHandler<i32>,
    on_edit: EventHandler<AppointmentResponse>,
    on_status_change: EventHandler<AppointmentResponse>,
    on_delete: EventHandler<AppointmentResponse>,
) -> Element {
    if appointments.is_empty() {
        return rsx! {
            div { class: "agenda-empty-day-state",
                div { class: "agenda-empty-icon-circle",
                    IconCalendar { size: 36, color: "#0052cc".to_string() }
                }
                h3 { class: "agenda-empty-title", "Nenhum agendamento neste dia" }
                p { class: "agenda-empty-desc", "Utilize o botão acima ou clique em qualquer horário para agendar um novo procedimento." }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| on_slot_click.call(9),
                        IconPlus { size: 16, color: "currentColor".to_string() }
                        span { " Agendar Horário" }
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "agenda-timeline-view",
            for hour in 7..=20 {
                HourlySlotRow {
                    key: "{hour}",
                    hour,
                    appointments: appointments.clone(),
                    can_write,
                    can_delete,
                    on_slot_click: on_slot_click.clone(),
                    on_edit: on_edit.clone(),
                    on_change_status: on_status_change.clone(),
                    on_delete: on_delete.clone(),
                }
            }
        }
    }
}

#[component]
fn HourlySlotRow(
    hour: i32,
    appointments: Vec<AppointmentResponse>,
    can_write: bool,
    can_delete: bool,
    on_slot_click: EventHandler<i32>,
    on_edit: EventHandler<AppointmentResponse>,
    on_change_status: EventHandler<AppointmentResponse>,
    on_delete: EventHandler<AppointmentResponse>,
) -> Element {
    let slot_apps: Vec<AppointmentResponse> = appointments
        .into_iter()
        .filter(|a| {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.scheduled_for) {
                let local_dt = dt.with_timezone(&chrono::Local);
                local_dt
                    .format("%H")
                    .to_string()
                    .parse::<i32>()
                    .unwrap_or(-1)
                    == hour
            } else {
                false
            }
        })
        .collect();

    let hour_formatted = format!("{:02}:00", hour);

    rsx! {
        div { class: "agenda-hour-slot-row",
            div { class: "agenda-hour-rail",
                div { class: "agenda-hour-time-badge", "{hour_formatted}" }
                div { class: "agenda-hour-rail-line" }
            }
            div { class: "agenda-hour-slot-content",
                if slot_apps.is_empty() {
                    div {
                        class: "agenda-slot-empty-lane",
                        onclick: move |_| on_slot_click.call(hour),
                        title: if can_write { "Clique para agendar às {hour_formatted}" } else { "" },
                        if can_write {
                            span { class: "agenda-slot-empty-hint",
                                IconPlus { size: 14, color: "currentColor".to_string() }
                                " Disponível • Clique para agendar"
                            }
                        }
                    }
                } else {
                    div { class: "agenda-cards-lane",
                        for app in slot_apps {
                            AppointmentCard {
                                key: "{app.id}",
                                appointment: app.clone(),
                                can_write,
                                can_delete,
                                on_edit: on_edit.clone(),
                                on_change_status: on_change_status.clone(),
                                on_delete: on_delete.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AppointmentCard(
    appointment: AppointmentResponse,
    can_write: bool,
    can_delete: bool,
    on_edit: EventHandler<AppointmentResponse>,
    on_change_status: EventHandler<AppointmentResponse>,
    on_delete: EventHandler<AppointmentResponse>,
) -> Element {
    let app_edit = appointment.clone();
    let app_status = appointment.clone();
    let app_del = appointment.clone();

    let time_str = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&appointment.scheduled_for)
    {
        let local_dt = dt.with_timezone(&chrono::Local);
        let end_dt = local_dt + chrono::Duration::minutes(appointment.duration_minutes as i64);
        format!(
            "{} - {} ({} min)",
            local_dt.format("%H:%M"),
            end_dt.format("%H:%M"),
            appointment.duration_minutes
        )
    } else {
        format!("{} min", appointment.duration_minutes)
    };

    let fin_badge = if let Some(amount_cents) = appointment.financial_amount_cents {
        let reais = amount_cents as f64 / 100.0;
        let is_income = appointment.financial_type.as_deref().unwrap_or("income") == "income";
        let prefix = if is_income { "+ R$" } else { "- R$" };
        let css_cls = if is_income {
            "app-fin-income"
        } else {
            "app-fin-expense"
        };
        Some((format!("{} {:.2}", prefix, reais), css_cls))
    } else {
        None
    };

    rsx! {
        div { class: "appointment-item-card {appointment.status.color_class()}",
            div { class: "app-card-left-col",
                div { class: "app-time-badge",
                    IconClock { size: 13, color: "currentColor".to_string() }
                    span { "{time_str}" }
                }
                span { class: "app-card-title", "{appointment.title}" }

                if let Some(ref p_name) = appointment.patient_name {
                    span { class: "app-patient-chip", "👤 {p_name}" }
                }

                if !appointment.assigned_users.is_empty() {
                    for user in &appointment.assigned_users {
                        span { class: "app-team-chip",
                            "👨‍⚕️ {user.user_name.as_deref().unwrap_or(&user.role_in_appointment)}"
                        }
                    }
                }

                if let Some((fin_text, fin_cls)) = fin_badge {
                    span { class: "app-fin-pill {fin_cls}", "{fin_text}" }
                }
            }

            div { class: "app-card-right-col",
                span { class: "{appointment.appointment_type.badge_class()}", "{appointment.appointment_type.label()}" }
                button {
                    class: "app-status-badge {appointment.status.color_class()}",
                    onclick: move |_| on_change_status.call(app_status.clone()),
                    title: "Clique para alterar status",
                    "{appointment.status.label()}"
                }
                div { class: "app-card-actions",
                    if can_write {
                        button {
                            class: "item-action-icon-btn",
                            onclick: move |_| on_edit.call(app_edit.clone()),
                            title: "Editar Agendamento",
                            IconEdit { size: 14, color: "currentColor".to_string() }
                        }
                    }
                    if can_delete {
                        button {
                            class: "item-action-icon-btn btn-danger-icon",
                            onclick: move |_| on_delete.call(app_del.clone()),
                            title: "Excluir Agendamento",
                            IconTrash { size: 14, color: "currentColor".to_string() }
                        }
                    }
                }
            }
        }
    }
}
