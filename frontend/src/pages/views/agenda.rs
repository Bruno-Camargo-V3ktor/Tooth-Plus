use crate::api;
use crate::components::icons::{
    IconBox, IconCalendar, IconCheck, IconChevronLeft, IconChevronRight, IconClock, IconEdit,
    IconFinance, IconPlus, IconSearch, IconTrash, IconUsers,
};
use crate::components::ui_blocks::ActionModal;
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use chrono::Datelike;
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, AppointmentStatus, AppointmentType,
    AssignedUserDto, ConsumedItemDto, CreateAppointmentRequest, UpdateAppointmentRequest,
    UpdateAppointmentStatusRequest,
};

#[component]
pub fn AgendaView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "appointments:read");
    let can_write = permissions::has_permission(&sess, &clinic, "appointments:write");
    let can_delete =
        permissions::has_permission(&sess, &clinic, "appointments:delete") || can_write;
    let can_finance = permissions::has_permission(&sess, &clinic, "appointments:finance");

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let now_date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut selected_date = use_signal(|| now_date_str);
    let mut selected_time = use_signal(|| "09:00".to_string());
    let mut search_query = use_signal(|| String::new());
    let mut filter_member = use_signal(|| "all".to_string());
    let mut filter_status = use_signal(|| "all".to_string());
    let mut filter_type = use_signal(|| "all".to_string());

    let mut is_form_modal_open = use_signal(|| false);
    let mut is_status_modal_open = use_signal(|| false);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut selected_appointment = use_signal(|| None::<AppointmentResponse>);
    let mut toast_msg = use_signal(|| None::<String>);

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();

    let mut appointments_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        let d = selected_date();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                vec![]
            } else {
                api::fetch_appointments(&t, &cid, Some(&d), None, None)
                    .await
                    .unwrap_or_default()
            }
        }
    });

    let tok_res_meta = token.clone();
    let cid_res_meta = clinic_id.clone();
    let resources_resource = use_resource(move || {
        let t = tok_res_meta.clone();
        let cid = cid_res_meta.clone();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                AgendaResourcesResponse {
                    team_members: vec![],
                    patients: vec![],
                    inventory_items: vec![],
                }
            } else {
                api::fetch_agenda_resources(&t, &cid)
                    .await
                    .unwrap_or(AgendaResourcesResponse {
                        team_members: vec![],
                        patients: vec![],
                        inventory_items: vec![],
                    })
            }
        }
    });

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar a agenda desta unidade." }
            }
        };
    }

    let all_appointments = appointments_resource().unwrap_or_default();
    let agenda_resources = resources_resource().unwrap_or(AgendaResourcesResponse {
        team_members: vec![],
        patients: vec![],
        inventory_items: vec![],
    });

    let s_query = search_query().to_lowercase();
    let f_member = filter_member();
    let f_status = filter_status();
    let f_type = filter_type();

    let filtered_appointments: Vec<AppointmentResponse> = all_appointments
        .into_iter()
        .filter(|app| {
            if !s_query.is_empty() {
                let matches_title = app.title.to_lowercase().contains(&s_query);
                let matches_patient = app
                    .patient_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&s_query);
                if !matches_title && !matches_patient {
                    return false;
                }
            }

            if f_member != "all" {
                let has_member = app.assigned_users.iter().any(|u| {
                    u.user_id == f_member || u.user_id.ends_with(&format!(":{}", f_member))
                });
                if !has_member {
                    return false;
                }
            }

            if f_status != "all" {
                let status_matches = match f_status.as_str() {
                    "pending" => app.status == AppointmentStatus::Pending,
                    "confirmed" => app.status == AppointmentStatus::Confirmed,
                    "in_progress" => app.status == AppointmentStatus::InProgress,
                    "completed" => app.status == AppointmentStatus::Completed,
                    "canceled" => app.status == AppointmentStatus::Canceled,
                    "no_show" => app.status == AppointmentStatus::NoShow,
                    _ => true,
                };
                if !status_matches {
                    return false;
                }
            }

            if f_type != "all" {
                let type_matches = match f_type.as_str() {
                    "consultation" => app.appointment_type == AppointmentType::Consultation,
                    "treatment" => app.appointment_type == AppointmentType::Treatment,
                    "surgery" => app.appointment_type == AppointmentType::Surgery,
                    "return" => app.appointment_type == AppointmentType::Return,
                    "meeting" => app.appointment_type == AppointmentType::Meeting,
                    "other" => app.appointment_type == AppointmentType::Other,
                    _ => true,
                };
                if !type_matches {
                    return false;
                }
            }

            true
        })
        .collect();

    let total_count = filtered_appointments.len();
    let completed_count = filtered_appointments
        .iter()
        .filter(|a| a.status == AppointmentStatus::Completed)
        .count();
    let in_progress_count = filtered_appointments
        .iter()
        .filter(|a| a.status == AppointmentStatus::InProgress)
        .count();
    let pending_count = filtered_appointments
        .iter()
        .filter(|a| {
            a.status == AppointmentStatus::Pending || a.status == AppointmentStatus::Confirmed
        })
        .count();

    let handle_prev_day = move |_| {
        if let Ok(curr) = chrono::NaiveDate::parse_from_str(&selected_date(), "%Y-%m-%d") {
            let prev = curr - chrono::Duration::days(1);
            selected_date.set(prev.format("%Y-%m-%d").to_string());
        }
    };

    let handle_next_day = move |_| {
        if let Ok(curr) = chrono::NaiveDate::parse_from_str(&selected_date(), "%Y-%m-%d") {
            let next = curr + chrono::Duration::days(1);
            selected_date.set(next.format("%Y-%m-%d").to_string());
        }
    };

    let handle_today = move |_| {
        selected_date.set(chrono::Local::now().format("%Y-%m-%d").to_string());
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
        div { class: "agenda-page-container",

            if let Some(msg) = toast_msg() {
                div { class: "toast-error",
                    span { "{msg}" }
                    button { class: "toast-close-btn", onclick: move |_| toast_msg.set(None), "×" }
                }
            }

            div { class: "agenda-kpi-row",
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconCalendar { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Total no Dia" }
                    }
                    div { class: "agenda-kpi-val", "{total_count}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconClock { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Pendentes" }
                    }
                    div { class: "agenda-kpi-val kpi-pending", "{pending_count}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                        IconUsers { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Em Atendimento" }
                    }
                    div { class: "agenda-kpi-val kpi-progress", "{in_progress_count}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconCheck { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Concluídos" }
                    }
                    div { class: "agenda-kpi-val kpi-completed", "{completed_count}" }
                }
            }

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
                        for member in &agenda_resources.team_members {
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
                            onclick: move |_| {
                                selected_appointment.set(None);
                                selected_time.set("09:00".to_string());
                                is_form_modal_open.set(true);
                            },
                            IconPlus { size: 16, color: "currentColor".to_string() }
                            "Novo Agendamento"
                        }
                    }
                }
            }

            div { class: "agenda-grid-card",
                if filtered_appointments.is_empty() {
                    div { class: "agenda-empty-day-state",
                        div { class: "agenda-empty-icon-circle",
                            IconCalendar { size: 36, color: "var(--clinic-primary, #00a0e4)" }
                        }
                        h3 { class: "agenda-empty-title", "Nenhum agendamento neste dia" }
                        p { class: "agenda-empty-desc", "Utilize o botão acima ou clique em qualquer horário para agendar um novo procedimento." }
                        if can_write {
                            button {
                                class: "btn-primary",
                                onclick: move |_| {
                                    selected_appointment.set(None);
                                    selected_time.set("09:00".to_string());
                                    is_form_modal_open.set(true);
                                },
                                IconPlus { size: 16, color: "currentColor".to_string() }
                                "Agendar Horário"
                            }
                        }
                    }
                } else {
                    div { class: "agenda-timeline-view",
                        for hour in 7..=20 {
                            HourlySlotRow {
                                key: "{hour}",
                                hour,
                                appointments: filtered_appointments.clone(),
                                can_write,
                                can_delete,
                                can_finance,
                                on_slot_click: move |h| {
                                    if can_write {
                                        selected_appointment.set(None);
                                        selected_time.set(format!("{:02}:00", h));
                                        is_form_modal_open.set(true);
                                    }
                                },
                                on_edit: move |app| {
                                    selected_appointment.set(Some(app));
                                    is_form_modal_open.set(true);
                                },
                                on_change_status: move |app| {
                                    selected_appointment.set(Some(app));
                                    is_status_modal_open.set(true);
                                },
                                on_delete: move |app| {
                                    selected_appointment.set(Some(app));
                                    is_delete_modal_open.set(true);
                                },
                            }
                        }
                    }
                }
            }

            if is_form_modal_open() {
                AppointmentFormModal {
                    is_open: is_form_modal_open,
                    appointment: selected_appointment,
                    clinic_id: clinic_id.clone(),
                    token: token.clone(),
                    default_date: selected_date(),
                    default_time: selected_time(),
                    resources: agenda_resources.clone(),
                    can_finance,
                    on_success: move |_| {
                        is_form_modal_open.set(false);
                        appointments_resource.restart();
                    },
                    on_error: move |err| toast_msg.set(Some(err)),
                }
            }

            if is_status_modal_open() {
                StatusChangeModal {
                    is_open: is_status_modal_open,
                    appointment: selected_appointment,
                    clinic_id: clinic_id.clone(),
                    token: token.clone(),
                    on_success: move |_| {
                        is_status_modal_open.set(false);
                        appointments_resource.restart();
                    },
                    on_error: move |err| toast_msg.set(Some(err)),
                }
            }

            if is_delete_modal_open() {
                DeleteAppointmentModal {
                    is_open: is_delete_modal_open,
                    appointment: selected_appointment,
                    clinic_id: clinic_id.clone(),
                    token: token.clone(),
                    on_success: move |_| {
                        is_delete_modal_open.set(false);
                        appointments_resource.restart();
                    },
                    on_error: move |err| toast_msg.set(Some(err)),
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
    can_finance: bool,
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
                                "Disponível • Clique para agendar"
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
                                can_finance,
                                on_edit: on_edit,
                                on_change_status: on_change_status,
                                on_delete: on_delete,
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
    can_finance: bool,
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

#[component]
fn AppointmentFormModal(
    is_open: Signal<bool>,
    appointment: Signal<Option<AppointmentResponse>>,
    clinic_id: String,
    token: String,
    default_date: String,
    default_time: String,
    resources: AgendaResourcesResponse,
    can_finance: bool,
    on_success: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    let editing_app = appointment();
    let is_edit = editing_app.is_some();
    let title_modal = if is_edit {
        "Editar Agendamento"
    } else {
        "Novo Agendamento"
    };

    let mut title = use_signal(|| String::new());
    let mut app_type = use_signal(|| AppointmentType::Consultation);
    let mut date_val = use_signal(|| default_date.clone());
    let mut time_val = use_signal(|| default_time.clone());
    let mut duration_minutes = use_signal(|| 30);
    let mut patient_name = use_signal(|| String::new());
    let mut patient_id = use_signal(|| None::<String>);

    let mut assigned_users = use_signal(|| Vec::<AssignedUserDto>::new());
    let mut consumed_items = use_signal(|| Vec::<ConsumedItemDto>::new());

    let mut financial_amount_str = use_signal(|| String::new());
    let mut financial_type = use_signal(|| "income".to_string());
    let mut is_submitting = use_signal(|| false);

    let first_member_opt = resources.team_members.first().cloned();
    use_effect(use_reactive(&editing_app, move |opt_a| {
        if let Some(a) = opt_a {
            title.set(a.title);
            app_type.set(a.appointment_type);
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.scheduled_for) {
                let local_dt = dt.with_timezone(&chrono::Local);
                date_val.set(local_dt.format("%Y-%m-%d").to_string());
                time_val.set(local_dt.format("%H:%M").to_string());
            }
            duration_minutes.set(a.duration_minutes);
            patient_name.set(a.patient_name.unwrap_or_default());
            patient_id.set(a.patient_id);
            assigned_users.set(a.assigned_users);
            consumed_items.set(a.consumed_items);
            if let Some(cents) = a.financial_amount_cents {
                financial_amount_str.set(format!("{:.2}", cents as f64 / 100.0));
            } else {
                financial_amount_str.set(String::new());
            }
            financial_type.set(a.financial_type.unwrap_or_else(|| "income".to_string()));
        } else {
            title.set(String::new());
            app_type.set(AppointmentType::Consultation);
            date_val.set(default_date.clone());
            time_val.set(default_time.clone());
            duration_minutes.set(30);
            patient_name.set(String::new());
            patient_id.set(None);
            if let Some(first_member) = first_member_opt.clone() {
                assigned_users.set(vec![AssignedUserDto {
                    user_id: first_member.id.clone(),
                    user_name: Some(first_member.name.clone()),
                    role_in_appointment: first_member
                        .extra_info
                        .clone()
                        .unwrap_or_else(|| "Dentista Principal".to_string()),
                    split_percentage: if can_finance { 100 } else { 0 },
                }]);
            } else {
                assigned_users.set(Vec::new());
            }
            consumed_items.set(Vec::new());
            financial_amount_str.set(String::new());
            financial_type.set("income".to_string());
        }
    }));

    let tok_submit = token.clone();
    let cid_submit = clinic_id.clone();

    let handle_submit = move |_| {
        if title().trim().is_empty() {
            on_error.call("Informe o título do agendamento.".to_string());
            return;
        }

        if assigned_users().is_empty() {
            on_error.call("Selecione ao menos um profissional responsável.".to_string());
            return;
        }

        let combined_dt_str = format!("{}T{}:00", date_val(), time_val());
        let scheduled_for =
            match chrono::NaiveDateTime::parse_from_str(&combined_dt_str, "%Y-%m-%dT%H:%M:%S") {
                Ok(ndt) => {
                    let local_res = ndt.and_local_timezone(chrono::Local);
                    if let Some(local_dt) = local_res.latest() {
                        local_dt.with_timezone(&chrono::Utc).to_rfc3339()
                    } else {
                        on_error.call("Horário inválido.".to_string());
                        return;
                    }
                }
                Err(_) => {
                    on_error.call("Data e horário inválidos.".to_string());
                    return;
                }
            };

        let fin_amount_cents = if !financial_amount_str().trim().is_empty() {
            if let Ok(val) = financial_amount_str().replace(',', ".").parse::<f64>() {
                Some((val * 100.0).round() as i64)
            } else {
                None
            }
        } else {
            None
        };

        is_submitting.set(true);
        let t = tok_submit.clone();
        let cid = cid_submit.clone();
        let a_opt = appointment();

        spawn(async move {
            if let Some(a) = a_opt {
                let req = UpdateAppointmentRequest {
                    title: Some(title()),
                    scheduled_for: Some(scheduled_for),
                    duration_minutes: Some(duration_minutes()),
                    appointment_type: Some(app_type()),
                    patient_id: patient_id(),
                    patient_name: if patient_name().trim().is_empty() {
                        None
                    } else {
                        Some(patient_name())
                    },
                    financial_amount_cents: fin_amount_cents,
                    financial_type: if fin_amount_cents.is_some() {
                        Some(financial_type())
                    } else {
                        None
                    },
                    assigned_users: Some(assigned_users()),
                    consumed_items: Some(consumed_items()),
                };

                match api::update_appointment(&t, &a.id, &cid, req).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
            } else {
                let req = CreateAppointmentRequest {
                    clinic_id: cid.clone(),
                    patient_id: patient_id(),
                    patient_name: if patient_name().trim().is_empty() {
                        None
                    } else {
                        Some(patient_name())
                    },
                    title: title(),
                    scheduled_for,
                    duration_minutes: duration_minutes(),
                    appointment_type: app_type(),
                    financial_amount_cents: fin_amount_cents,
                    financial_type: if fin_amount_cents.is_some() {
                        Some(financial_type())
                    } else {
                        None
                    },
                    assigned_users: assigned_users(),
                    consumed_items: consumed_items(),
                };

                match api::create_appointment(&t, req).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
            }
            is_submitting.set(false);
        });
    };

    rsx! {
        ActionModal {
            is_open: is_open(),
            title: title_modal.to_string(),
            on_close: move |_| is_open.set(false),

            div { class: "form-grid",
                div { class: "input-group-wrapper full-width",
                    label { "Título do Agendamento / Procedimento *" }
                    input {
                        class: "modern-input-field",
                        placeholder: "Ex: Avaliação de Implante, Clareamento, Retorno",
                        value: "{title}",
                        oninput: move |e| title.set(e.value())
                    }
                }

                div { class: "input-group-wrapper",
                    label { "Tipo de Atendimento" }
                    select {
                        class: "modern-input-field modern-select",
                        value: match app_type() {
                            AppointmentType::Consultation => "consultation",
                            AppointmentType::Treatment => "treatment",
                            AppointmentType::Surgery => "surgery",
                            AppointmentType::Return => "return",
                            AppointmentType::Meeting => "meeting",
                            AppointmentType::Other => "other",
                        },
                        onchange: move |e: FormEvent| {
                            app_type.set(match e.value().as_str() {
                                "treatment" => AppointmentType::Treatment,
                                "surgery" => AppointmentType::Surgery,
                                "return" => AppointmentType::Return,
                                "meeting" => AppointmentType::Meeting,
                                "other" => AppointmentType::Other,
                                _ => AppointmentType::Consultation,
                            });
                        },
                        option { value: "consultation", "Consulta" }
                        option { value: "treatment", "Tratamento" }
                        option { value: "surgery", "Cirurgia" }
                        option { value: "return", "Retorno" }
                        option { value: "meeting", "Reunião" }
                        option { value: "other", "Outro" }
                    }
                }

                div { class: "input-group-wrapper",
                    label { "Duração Estimada" }
                    select {
                        class: "modern-input-field modern-select",
                        value: "{duration_minutes}",
                        onchange: move |e: FormEvent| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                duration_minutes.set(v);
                            }
                        },
                        option { value: "15", "15 minutos" }
                        option { value: "30", "30 minutos" }
                        option { value: "45", "45 minutos" }
                        option { value: "60", "1 hora (60 min)" }
                        option { value: "90", "1 hora e 30 min" }
                        option { value: "120", "2 horas" }
                    }
                }

                div { class: "input-group-wrapper",
                    label { "Data do Atendimento" }
                    input {
                        class: "modern-input-field",
                        r#type: "date",
                        value: "{date_val}",
                        oninput: move |e| date_val.set(e.value())
                    }
                    div { class: "agenda-quick-dates-row",
                        button {
                            class: "agenda-quick-date-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                date_val.set(today);
                            },
                            "Hoje"
                        }
                        button {
                            class: "agenda-quick-date-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let tomorrow = (chrono::Utc::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
                                date_val.set(tomorrow);
                            },
                            "Amanhã"
                        }
                        button {
                            class: "agenda-quick-date-btn",
                            r#type: "button",
                            onclick: move |_| {
                                let next_week = (chrono::Utc::now() + chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
                                date_val.set(next_week);
                            },
                            "+7 dias"
                        }
                    }
                }

                div { class: "input-group-wrapper",
                    label { "Horário de Início (Padrão 24h)" }
                    select {
                        class: "modern-input-field modern-select",
                        value: "{time_val}",
                        onchange: move |e: FormEvent| {
                            time_val.set(e.value());
                        },
                        option { value: "07:00", "07:00 (Manhã)" }
                        option { value: "07:30", "07:30" }
                        option { value: "08:00", "08:00" }
                        option { value: "08:30", "08:30" }
                        option { value: "09:00", "09:00" }
                        option { value: "09:30", "09:30" }
                        option { value: "10:00", "10:00" }
                        option { value: "10:30", "10:30" }
                        option { value: "11:00", "11:00" }
                        option { value: "11:30", "11:30" }
                        option { value: "12:00", "12:00 (Almoço)" }
                        option { value: "12:30", "12:30" }
                        option { value: "13:00", "13:00 (Tarde)" }
                        option { value: "13:30", "13:30" }
                        option { value: "14:00", "14:00" }
                        option { value: "14:30", "14:30" }
                        option { value: "15:00", "15:00" }
                        option { value: "15:30", "15:30" }
                        option { value: "16:00", "16:00" }
                        option { value: "16:30", "16:30" }
                        option { value: "17:00", "17:00" }
                        option { value: "17:30", "17:30" }
                        option { value: "18:00", "18:00 (Noite)" }
                        option { value: "18:30", "18:30" }
                        option { value: "19:00", "19:00" }
                        option { value: "19:30", "19:30" }
                        option { value: "20:00", "20:00" }
                    }
                }

                div { class: "input-group-wrapper full-width",
                    label { "Paciente (Opcional)" }
                    div { class: "agenda-patient-picker-container",
                        if !resources.patients.is_empty() {
                            select {
                                class: "modern-input-field modern-select",
                                onchange: move |e: FormEvent| {
                                    let val = e.value();
                                    if val == "custom" || val.is_empty() {
                                        patient_id.set(None);
                                    } else {
                                        if let Some(p) = resources.patients.iter().find(|x| x.id == val) {
                                            patient_id.set(Some(p.id.clone()));
                                            patient_name.set(p.name.clone());
                                        }
                                    }
                                },
                                option { value: "custom", "Selecione um paciente cadastrado ou digite abaixo..." }
                                for pat in &resources.patients {
                                    option { value: "{pat.id}", "{pat.name} {pat.extra_info.as_deref().unwrap_or(\"\")}" }
                                }
                            }
                        }
                        input {
                            class: "modern-input-field",
                            placeholder: "Nome do paciente (ou selecione acima)",
                            value: "{patient_name}",
                            oninput: move |e| {
                                patient_name.set(e.value());
                                patient_id.set(None);
                            }
                        }
                    }
                }

                div { class: "input-group-wrapper full-width",
                    h4 { class: "form-section-title", "Profissionais Responsáveis *" }
                    if resources.team_members.is_empty() {
                        div { class: "empty-helper-banner",
                            "Nenhum membro da equipe associado a esta unidade. Cadastre membros no módulo de Usuários."
                        }
                    } else {
                        div { class: "agenda-assignment-box",
                            for member in &resources.team_members {
                                {
                                    let mid = member.id.clone();
                                    let mname = member.name.clone();
                                    let current_assigned = assigned_users();
                                    let existing_entry = current_assigned.iter().find(|u| u.user_id == mid);
                                    let is_assigned = existing_entry.is_some();
                                    let split_val = existing_entry.map(|u| u.split_percentage).unwrap_or(0);
                                    let role_val = existing_entry.map(|u| u.role_in_appointment.clone()).unwrap_or_else(|| member.extra_info.clone().unwrap_or_else(|| "Dentista".to_string()));

                                    let mid_chk = mid.clone();
                                    let mname_chk = mname.clone();
                                    let role_chk = role_val.clone();
                                    let mid_role = mid.clone();
                                    let mid_split = mid.clone();

                                    rsx! {
                                        div { key: "{member.id}", class: "agenda-member-assign-row",
                                            label { class: "perm-checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: is_assigned,
                                                    onchange: move |e: FormEvent| {
                                                        let mut curr = assigned_users();
                                                        if e.checked() {
                                                            if !curr.iter().any(|u| u.user_id == mid_chk) {
                                                                curr.push(AssignedUserDto {
                                                                    user_id: mid_chk.clone(),
                                                                    user_name: Some(mname_chk.clone()),
                                                                    role_in_appointment: role_chk.clone(),
                                                                    split_percentage: if can_finance { 100 } else { 0 },
                                                                });
                                                            }
                                                        } else {
                                                            curr.retain(|u| u.user_id != mid_chk);
                                                        }
                                                        assigned_users.set(curr);
                                                    }
                                                }
                                                span { class: "member-assign-name", "{member.name}" }
                                            }

                                            if is_assigned {
                                                div { class: "agenda-assign-extras",
                                                    input {
                                                        class: "agenda-role-input",
                                                        placeholder: "Função no atendimento",
                                                        value: "{role_val}",
                                                        oninput: move |e: FormEvent| {
                                                            let mut curr = assigned_users();
                                                            if let Some(entry) = curr.iter_mut().find(|u| u.user_id == mid_role) {
                                                                entry.role_in_appointment = e.value();
                                                            }
                                                            assigned_users.set(curr);
                                                        }
                                                    }
                                                    if can_finance {
                                                        div { class: "agenda-split-input-wrapper",
                                                            label { "Rateio:" }
                                                            input {
                                                                class: "agenda-split-num-input",
                                                                r#type: "number",
                                                                min: "0",
                                                                max: "100",
                                                                value: "{split_val}",
                                                                oninput: move |e: FormEvent| {
                                                                    let mut curr = assigned_users();
                                                                    if let Some(entry) = curr.iter_mut().find(|u| u.user_id == mid_split) {
                                                                        if let Ok(v) = e.value().parse::<i32>() {
                                                                            entry.split_percentage = v.clamp(0, 100);
                                                                        }
                                                                    }
                                                                    assigned_users.set(curr);
                                                                }
                                                            }
                                                            span { "%" }
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

                if !resources.inventory_items.is_empty() {
                    div { class: "input-group-wrapper full-width",
                        h4 { class: "form-section-title", "Consumo de Materiais de Estoque" }
                        div { class: "agenda-inventory-box",
                            for item in &resources.inventory_items {
                                {
                                    let i_id = item.id.clone();
                                    let i_name = item.name.clone();
                                    let current_items = consumed_items();
                                    let existing_item = current_items.iter().find(|i| i.item_id == i_id);
                                    let is_consumed = existing_item.is_some();
                                    let qty_planned = existing_item.map(|i| i.quantity_planned).unwrap_or(1);

                                    let i_id_chk = i_id.clone();
                                    let i_name_chk = i_name.clone();
                                    let i_id_qty = i_id.clone();

                                    rsx! {
                                        div { key: "{item.id}", class: "agenda-item-consume-row",
                                            label { class: "perm-checkbox-item",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: is_consumed,
                                                    onchange: move |e: FormEvent| {
                                                        let mut curr = consumed_items();
                                                        if e.checked() {
                                                            if !curr.iter().any(|i| i.item_id == i_id_chk) {
                                                                curr.push(ConsumedItemDto {
                                                                    item_id: i_id_chk.clone(),
                                                                    item_name: Some(i_name_chk.clone()),
                                                                    quantity_planned: 1,
                                                                    quantity_used: None,
                                                                });
                                                            }
                                                        } else {
                                                            curr.retain(|i| i.item_id != i_id_chk);
                                                        }
                                                        consumed_items.set(curr);
                                                    }
                                                }
                                                span { "{item.name} ({item.extra_info.as_deref().unwrap_or(\"un\")})" }
                                            }

                                            if is_consumed {
                                                div { class: "agenda-consume-qty-wrapper",
                                                    label { "Qtd. Prevista:" }
                                                    input {
                                                        class: "agenda-split-num-input",
                                                        r#type: "number",
                                                        min: "1",
                                                        value: "{qty_planned}",
                                                        oninput: move |e: FormEvent| {
                                                            let mut curr = consumed_items();
                                                            if let Some(entry) = curr.iter_mut().find(|i| i.item_id == i_id_qty) {
                                                                if let Ok(v) = e.value().parse::<i32>() {
                                                                    entry.quantity_planned = v.max(1);
                                                                }
                                                            }
                                                            consumed_items.set(curr);
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

                div { class: "input-group-wrapper",
                    label { "Valor Financeiro (R$ - Opcional)" }
                    input {
                        class: "modern-input-field",
                        placeholder: "0,00",
                        value: "{financial_amount_str}",
                        oninput: move |e| financial_amount_str.set(e.value())
                    }
                }

                div { class: "input-group-wrapper",
                    label { "Tipo Financeiro" }
                    select {
                        class: "modern-input-field modern-select",
                        value: "{financial_type}",
                        onchange: move |e: FormEvent| financial_type.set(e.value()),
                        option { value: "income", "Entrada (Receita)" }
                        option { value: "expense", "Saída (Despesa)" }
                    }
                }
            }

            div { class: "modal-footer-actions",
                button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                button {
                    class: "btn-primary",
                    disabled: is_submitting(),
                    onclick: handle_submit,
                    if is_submitting() { "Salvando..." } else { "Salvar Agendamento" }
                }
            }
        }
    }
}

#[component]
fn StatusChangeModal(
    is_open: Signal<bool>,
    appointment: Signal<Option<AppointmentResponse>>,
    clinic_id: String,
    token: String,
    on_success: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    let app = appointment();
    let mut new_status = use_signal(|| AppointmentStatus::Confirmed);
    let mut reason = use_signal(|| String::new());
    let mut consumed_items = use_signal(|| Vec::<ConsumedItemDto>::new());
    let mut is_submitting = use_signal(|| false);

    use_effect(use_reactive(&app, move |opt_a| {
        if let Some(a) = opt_a {
            new_status.set(a.status);
            reason.set(a.cancellation_reason.unwrap_or_default());
            consumed_items.set(a.consumed_items);
        }
    }));

    let handle_status_submit = move |_| {
        if let Some(a) = app.clone() {
            is_submitting.set(true);
            let t = token.clone();
            let cid = clinic_id.clone();
            let aid = a.id.clone();
            let st = new_status();
            let r = if st == AppointmentStatus::Canceled && !reason().trim().is_empty() {
                Some(reason())
            } else {
                None
            };
            let items = if st == AppointmentStatus::Completed {
                Some(consumed_items())
            } else {
                None
            };

            spawn(async move {
                let req = UpdateAppointmentStatusRequest {
                    status: st,
                    cancellation_reason: r,
                    consumed_items: items,
                };
                match api::update_appointment_status(&t, &aid, &cid, req).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
                is_submitting.set(false);
            });
        }
    };

    rsx! {
        ActionModal {
            is_open: is_open(),
            title: "Alterar Status do Agendamento".to_string(),
            on_close: move |_| is_open.set(false),

            div { class: "form-grid",
                div { class: "input-group-wrapper full-width",
                    label { "Novo Status" }
                    select {
                        class: "modern-input-field modern-select",
                        value: match new_status() {
                            AppointmentStatus::Pending => "pending",
                            AppointmentStatus::Confirmed => "confirmed",
                            AppointmentStatus::InProgress => "in_progress",
                            AppointmentStatus::Completed => "completed",
                            AppointmentStatus::Canceled => "canceled",
                            AppointmentStatus::NoShow => "no_show",
                        },
                        onchange: move |e: FormEvent| {
                            new_status.set(match e.value().as_str() {
                                "confirmed" => AppointmentStatus::Confirmed,
                                "in_progress" => AppointmentStatus::InProgress,
                                "completed" => AppointmentStatus::Completed,
                                "canceled" => AppointmentStatus::Canceled,
                                "no_show" => AppointmentStatus::NoShow,
                                _ => AppointmentStatus::Pending,
                            });
                        },
                        option { value: "pending", "Pendente" }
                        option { value: "confirmed", "Confirmado" }
                        option { value: "in_progress", "Em Atendimento" }
                        option { value: "completed", "Concluído (Baixa Estoque + Repasse)" }
                        option { value: "canceled", "Cancelado" }
                        option { value: "no_show", "Não Compareceu" }
                    }
                }

                if new_status() == AppointmentStatus::Canceled {
                    div { class: "input-group-wrapper full-width",
                        label { "Motivo do Cancelamento" }
                        input {
                            class: "modern-input-field",
                            placeholder: "Ex: Paciente solicitou reagendamento, Imprevisto médico",
                            value: "{reason}",
                            oninput: move |e| reason.set(e.value())
                        }
                    }
                }

                if new_status() == AppointmentStatus::Completed && !consumed_items().is_empty() {
                    div { class: "input-group-wrapper full-width",
                        h4 { class: "form-section-title", "Confirmar Quantidades Usadas de Estoque" }
                        p { class: "app-modal-hint", "Ao marcar como Concluído, os itens abaixo serão baixados do estoque automaticamente." }
                        div { class: "agenda-inventory-box",
                            for (idx, item) in consumed_items().iter().enumerate() {
                                {
                                    let qty_val = item.quantity_used.unwrap_or(item.quantity_planned);
                                    rsx! {
                                        div { key: "{item.item_id}", class: "agenda-item-consume-row",
                                            span { "{item.item_name.as_deref().unwrap_or(&item.item_id)}" }
                                            div { class: "agenda-consume-qty-wrapper",
                                                label { "Qtd. Efetiva:" }
                                                input {
                                                    class: "agenda-split-num-input",
                                                    r#type: "number",
                                                    min: "0",
                                                    value: "{qty_val}",
                                                    oninput: move |e: FormEvent| {
                                                        let mut curr = consumed_items();
                                                        if let Some(entry) = curr.get_mut(idx) {
                                                            if let Ok(v) = e.value().parse::<i32>() {
                                                                entry.quantity_used = Some(v.max(0));
                                                            }
                                                        }
                                                        consumed_items.set(curr);
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

            div { class: "modal-footer-actions",
                button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                button {
                    class: "btn-primary",
                    disabled: is_submitting(),
                    onclick: handle_status_submit,
                    if is_submitting() { "Atualizando..." } else { "Confirmar Status" }
                }
            }
        }
    }
}

#[component]
fn DeleteAppointmentModal(
    is_open: Signal<bool>,
    appointment: Signal<Option<AppointmentResponse>>,
    clinic_id: String,
    token: String,
    on_success: EventHandler<()>,
    on_error: EventHandler<String>,
) -> Element {
    let mut is_deleting = use_signal(|| false);

    let handle_delete = move |_| {
        if let Some(a) = appointment() {
            is_deleting.set(true);
            let t = token.clone();
            let cid = clinic_id.clone();
            let aid = a.id.clone();
            spawn(async move {
                match api::delete_appointment(&t, &aid, &cid).await {
                    Ok(_) => on_success.call(()),
                    Err(e) => on_error.call(e),
                }
                is_deleting.set(false);
            });
        }
    };

    rsx! {
        ActionModal {
            is_open: is_open(),
            title: "Excluir Agendamento".to_string(),
            on_close: move |_| is_open.set(false),

            div {
                p { class: "delete-modal-text", "Tem certeza que deseja excluir permanentemente este agendamento da agenda?" }
                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-danger",
                        disabled: is_deleting(),
                        onclick: handle_delete,
                        if is_deleting() { "Excluindo..." } else { "Sim, Excluir" }
                    }
                }
            }
        }
    }
}
