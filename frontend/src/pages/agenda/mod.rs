//! # Página de Agenda (Tooth Plus V2)
//!
//! Grade semanal interativa com eventos coloridos por status, popover de detalhes,
//! modal de nova consulta/compromisso e integração com o modal de cadastro de paciente.

use crate::api::{ActiveClinicState, AppointmentsApi, SessionState};
use crate::components::patient_form_modal::PatientFormModal;
use crate::components::toast::{ToastState, ToastVariant};
use dioxus::prelude::*;
use shared::appointments::{
    AppointmentResponse, AppointmentStatus, AppointmentType, AssignedUserDto,
    CreateAppointmentRequest,
};

const STYLE: Asset = asset!("/src/pages/agenda/style.css");

// ─── HELPERS DE DATA/HORA ───────────────────────────────────────────────────

fn today_iso() -> String {
    // Obtém a data de hoje via JS
    js_sys::eval("new Date().toISOString().split('T')[0]")
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "2026-08-25".to_string())
}

fn parse_date(iso: &str) -> (i32, u32, u32) {
    let parts: Vec<&str> = iso.split('-').collect();
    let year = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(2026);
    let month = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let day = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(25);
    (year, month, day)
}

// Dias no mês (ignora anos bissextos para mês 2 → usa 28)
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if year % 4 == 0 { 29 } else { 28 },
        _ => 30,
    }
}

// Adiciona `days` dias a uma data ISO
fn add_days(iso: &str, days: i32) -> String {
    let (mut y, mut m, mut d) = parse_date(iso);
    let mut total = d as i32 + days;
    while total < 1 {
        m -= 1;
        if m == 0 { m = 12; y -= 1; }
        total += days_in_month(y, m) as i32;
    }
    while total > days_in_month(y, m) as i32 {
        total -= days_in_month(y, m) as i32;
        m += 1;
        if m > 12 { m = 1; y += 1; }
    }
    format!("{:04}-{:02}-{:02}", y, m, total as u32)
}

// Retorna segunda-feira da semana de `iso`
fn week_start(iso: &str) -> String {
    let js_code = format!(
        "(function(){{ var d=new Date('{}T12:00:00'); var day=d.getDay(); var diff=day===0?-6:1-day; d.setDate(d.getDate()+diff); return d.toISOString().split('T')[0]; }})()",
        iso
    );
    js_sys::eval(&js_code)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| iso.to_string())
}

fn format_display_date(iso: &str) -> String {
    let (y, m, d) = parse_date(iso);
    format!("{}/{}/{}", d, m, y)
}

fn week_month_label(start: &str) -> String {
    let months = ["Jan","Fev","Mar","Abr","Mai","Jun","Jul","Ago","Set","Out","Nov","Dez"];
    let (y, m, _) = parse_date(start);
    let end = add_days(start, 6);
    let (y2, m2, _) = parse_date(&end);
    if m == m2 {
        format!("{} {}", months[(m-1) as usize], y)
    } else {
        format!("{}/{} {}", months[(m-1) as usize], months[(m2-1) as usize], if y == y2 { y.to_string() } else { format!("{}/{}", y, y2) })
    }
}

fn day_name_pt(iso: &str) -> &'static str {
    let js = format!("new Date('{}T12:00:00').getDay()", iso);
    let dow = js_sys::eval(&js).ok().and_then(|v| v.as_f64()).unwrap_or(1.0) as u8;
    match dow {
        0 => "Dom", 1 => "Seg", 2 => "Ter", 3 => "Qua", 4 => "Qui", 5 => "Sex", 6 => "Sáb", _ => "?"
    }
}

fn appointment_status_label(status: &AppointmentStatus) -> &'static str {
    match status {
        AppointmentStatus::Confirmed         => "Confirmada",
        AppointmentStatus::Completed         => "Finalizada",
        AppointmentStatus::InProgress        => "Em Atendimento",
        AppointmentStatus::Pending           => "Pendente",
        AppointmentStatus::Canceled          => "Cancelada",
        AppointmentStatus::CanceledByDoctor  => "Cancelada (Médico)",
        AppointmentStatus::CanceledByPatient => "Cancelada (Paciente)",
        AppointmentStatus::NoShow            => "Não Compareceu",
    }
}

fn appointment_status_css(status: &AppointmentStatus) -> &'static str {
    match status {
        AppointmentStatus::Confirmed         => "status-confirmed",
        AppointmentStatus::Completed         => "status-finalized",
        AppointmentStatus::InProgress        => "status-waiting",
        AppointmentStatus::Pending           => "status-pending",
        AppointmentStatus::Canceled
        | AppointmentStatus::CanceledByDoctor
        | AppointmentStatus::CanceledByPatient
        | AppointmentStatus::NoShow          => "status-cancelled",
    }
}

// Extrai hora e minuto de uma string "YYYY-MM-DDTHH:MM:SSZ"
fn extract_hhmm(dt: &str) -> (u32, u32) {
    let time_part = dt.split('T').nth(1).unwrap_or("00:00");
    let parts: Vec<&str> = time_part.split(':').collect();
    let h: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let m: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    (h, m)
}

// Extrai a data "YYYY-MM-DD" de um datetime
fn extract_date(dt: &str) -> String {
    dt.split('T').next().unwrap_or(dt).to_string()
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase()
}

// ─── PÁGINA PRINCIPAL ────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum CalendarMode {
    Week,
    Day,
}

#[derive(Clone, Debug)]
struct PopoverPos {
    x: f64,
    y: f64,
    appointment_id: String,
}

#[component]
pub fn AgendaView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let today = today_iso();
    let clinic_id = active_clinic.read().as_ref().map(|c| c.clinic_id.clone()).unwrap_or_default();

    let mut current_date = use_signal(|| today.clone());
    let mut view_mode = use_signal(|| CalendarMode::Week);
    let mut selected_professional = use_signal(|| "all".to_string());
    let mut appointments = use_signal(|| Vec::<AppointmentResponse>::new());
    let mut professionals = use_signal(|| Vec::<(String, String)>::new()); // (id, name)
    let mut is_loading = use_signal(|| true);
    let mut show_new_modal = use_signal(|| false);
    let mut show_patient_modal = use_signal(|| false);
    let mut popover = use_signal(|| Option::<PopoverPos>::None);
    // Data/hora pré-preenchida quando clica numa célula vazia
    let mut prefill_date = use_signal(|| String::new());
    let mut prefill_hour = use_signal(|| String::new());

    // Clone toast antes do use_effect para evitar move
    let toast_for_effect = toast.clone();
    let toast_for_del = toast.clone();
    let toast_for_save = toast.clone();
    let toast_for_status = toast.clone();
    let toast_for_patient = toast.clone();

    // Carrega agendamentos e profissionais
    let clinic_id_for_effect = clinic_id.clone();
    use_effect(move || {
        let cid = clinic_id_for_effect.clone();
        let mut toast_clone = toast_for_effect.clone();
        spawn(async move {
            match AppointmentsApi::list_appointments(&cid, None).await {
                Ok(apps) => { appointments.set(apps); }
                Err(e) => {
                    let _ = &e; web_sys::console::log_1(&format!("Error: {}", e).into());
                    toast_clone.show(format!("Erro ao carregar agenda: {}", e), ToastVariant::Error);
                }
            }
            match AppointmentsApi::get_agenda_resources(&cid).await {
                Ok(res) => {
                    let profs: Vec<(String, String)> = res.team_members.iter()
                        .map(|m| (m.id.clone(), m.name.clone()))
                        .collect();
                    professionals.set(profs);
                }
                Err(_) => {}
            }
            is_loading.set(false);
        });
    });

    // Calcula semana atual
    let week_start_date = week_start(&current_date.read());
    let week_days: Vec<String> = (0..7).map(|i| add_days(&week_start_date, i)).collect();
    let period_label = week_month_label(&week_start_date);

    // Filtra appointments
    let prof_filter = selected_professional.read().clone();
    let filtered_appointments: Vec<AppointmentResponse> = appointments.read().iter()
        .filter(|a| {
            if prof_filter == "all" { return true; }
            a.assigned_users.iter().any(|u| u.user_id == prof_filter)
        })
        .cloned()
        .collect();

    // Horas exibidas (08h - 20h)
    let hours: Vec<u32> = (8..=20).collect();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "agenda-page",

            // ─── TOOLBAR ───────────────────────────────────────────────────
            div { class: "agenda-toolbar",
                div { class: "agenda-toolbar-left",
                    // Filtro por profissional
                    select {
                        class: "agenda-select-prof",
                        value: "{selected_professional}",
                        onchange: move |e| selected_professional.set(e.value()),
                        option { value: "all", "Todos os profissionais" }
                        for (id, name) in professionals.read().iter() {
                            option { value: "{id}", "{name}" }
                        }
                    }

                    // Botão HOJE
                    button {
                        class: "agenda-today-btn",
                        onclick: move |_| current_date.set(today.clone()),
                        "HOJE"
                    }

                    // Navegação semana
                    div { class: "agenda-nav-group",
                        button {
                            class: "agenda-nav-btn",
                            onclick: move |_| {
                                let new_date = add_days(&current_date.read(), -7);
                                current_date.set(new_date);
                            },
                            "‹"
                        }
                        button {
                            class: "agenda-nav-btn",
                            onclick: move |_| {
                                let new_date = add_days(&current_date.read(), 7);
                                current_date.set(new_date);
                            },
                            "›"
                        }
                    }

                    span { class: "agenda-current-period", "{period_label}" }
                }

                div { class: "agenda-toolbar-right",
                    // Seletor de data
                    input {
                        class: "agenda-date-input",
                        r#type: "date",
                        value: "{current_date}",
                        onchange: move |e| current_date.set(e.value()),
                    }

                    // Toggle Semana/Dia
                    div { class: "agenda-view-toggle",
                        button {
                            class: if *view_mode.read() == CalendarMode::Week { "agenda-view-btn active" } else { "agenda-view-btn" },
                            onclick: move |_| view_mode.set(CalendarMode::Week),
                            "Semana"
                        }
                        button {
                            class: if *view_mode.read() == CalendarMode::Day { "agenda-view-btn active" } else { "agenda-view-btn" },
                            onclick: move |_| view_mode.set(CalendarMode::Day),
                            "Dia"
                        }
                    }

                    // Botão + novo
                    button {
                        class: "agenda-new-btn",
                        title: "Nova consulta ou compromisso",
                        onclick: move |_| {
                            prefill_date.set(current_date.read().clone());
                            prefill_hour.set("09:00".to_string());
                            show_new_modal.set(true);
                        },
                        "+"
                    }
                }
            }

            // ─── GRADE ─────────────────────────────────────────────────────
            div { class: "agenda-body",
                div { class: "agenda-grid-scroll",
                    if is_loading() {
                        div { class: "empty-state",
                            div { class: "empty-state-icon", "📅" }
                            p { class: "empty-state-title", "Carregando agenda..." }
                        }
                    } else {
                        {
                            let display_days = if *view_mode.read() == CalendarMode::Week {
                                week_days.clone()
                            } else {
                                vec![current_date.read().clone()]
                            };
                            let grid_cols = if *view_mode.read() == CalendarMode::Week { 8 } else { 2 };
                            let today_ref = today_iso();

                            rsx! {
                                div {
                                    class: if *view_mode.read() == CalendarMode::Week { "agenda-grid" } else { "agenda-grid agenda-grid-day" },
                                    style: format!("grid-template-columns: 60px repeat({}, minmax({}px, 1fr));",
                                        grid_cols - 1,
                                        if *view_mode.read() == CalendarMode::Week { 110 } else { 200 }
                                    ),

                                    // Cabeçalho
                                    div { class: "agenda-header-time-spacer" }
                                    for day_iso in display_days.iter() {
                                        {
                                            let is_today = *day_iso == today_ref;
                                            let (_, _, day_num) = parse_date(day_iso);
                                            let day_name = day_name_pt(day_iso);
                                            rsx! {
                                                div {
                                                    key: "hdr-{day_iso}",
                                                    class: if is_today { "agenda-header-day today-col" } else { "agenda-header-day" },
                                                    div { class: "agenda-header-day-name", "{day_name}" }
                                                    div {
                                                        class: if is_today { "agenda-header-day-num today-num" } else { "agenda-header-day-num" },
                                                        "{day_num}"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Linhas de hora
                                    for &hour in hours.iter() {
                                        // Label de hora
                                        div {
                                            key: "tl-{hour}",
                                            class: "agenda-time-label",
                                            { let lbl = format!("{:02}h00", hour); lbl }
                                        }

                                        // Células por dia
                                        for day_iso in display_days.iter() {
                                            {
                                                let is_today = *day_iso == today_ref;
                                                let day_iso_clone = day_iso.clone();

                                                // Encontra appointments nesta célula
                                                let cell_apps: Vec<AppointmentResponse> = filtered_appointments.iter()
                                                    .filter(|a| {
                                                        let app_date = extract_date(&a.scheduled_for);
                                                        let (app_h, _) = extract_hhmm(&a.scheduled_for);
                                                        app_date == *day_iso && app_h == hour
                                                    })
                                                    .cloned()
                                                    .collect();

                                                let h_clone = hour;
                                                let d_clone = day_iso.clone();
                                                let mut pf_date = prefill_date.clone();
                                                let mut pf_hour = prefill_hour.clone();
                                                let mut show_modal = show_new_modal.clone();

                                                rsx! {
                                                    div {
                                                        key: "cell-{day_iso}-{hour}",
                                                        class: if is_today { "agenda-cell today-col" } else { "agenda-cell" },
                                                        onclick: move |_| {
                                                            pf_date.set(d_clone.clone());
                                                            pf_hour.set(format!("{:02}:00", h_clone));
                                                            show_modal.set(true);
                                                        },

                                                        for (idx, app) in cell_apps.iter().enumerate() {
                                                            {
                                                                let app = app.clone();
                                                                let app_id = app.id.clone();
                                                                let (_, app_min) = extract_hhmm(&app.scheduled_for);
                                                                let dur: u32 = app.duration_minutes as u32;
                                                                let top_offset = (app_min as f64 / 60.0) * 60.0;
                                                                let height = (dur as f64 / 60.0 * 60.0_f64).max(22.0) - 4.0;
                                                                let left_pct = idx as f64 * 50.0;
                                                                let status_css = appointment_status_css(&app.status);

                                                                let patient_name = app.patient_name.clone()
                                                                    .unwrap_or_else(|| app.title.clone());
                                                                let dentist = app.assigned_users.first()
                                                                    .map(|u| format!("Dr(a). {}", u.user_name.as_deref().unwrap_or("").split_whitespace().last().unwrap_or("")))
                                                                    .unwrap_or_default();
                                                                let (ah, am) = extract_hhmm(&app.scheduled_for);
                                                                let end_min = am + dur;
                                                                let end_h = ah + end_min / 60;
                                                                let end_m = end_min % 60;
                                                                let time_str = format!("{:02}h{:02} - {:02}h{:02}", ah, am, end_h, end_m);

                                                                let mut pop_signal = popover.clone();

                                                                rsx! {
                                                                    div {
                                                                        key: "{app_id}",
                                                                        class: "event-card {status_css}",
                                                                        style: format!(
                                                                            "top: {}px; height: {}px; left: calc({}% + 2px); right: 2px; z-index: {};",
                                                                            top_offset, height, left_pct, 5 + idx
                                                                        ),
                                                                        onclick: move |e| {
                                                                            e.stop_propagation();
                                                                            let rect = e.client_coordinates();
                                                                            pop_signal.set(Some(PopoverPos {
                                                                                x: rect.x as f64,
                                                                                y: rect.y as f64,
                                                                                appointment_id: app_id.clone(),
                                                                            }));
                                                                        },
                                                                        div { class: "event-time", "{time_str}" }
                                                                        div { class: "event-name", "{patient_name}" }
                                                                        if !dentist.is_empty() {
                                                                            div { class: "event-dentist", "{dentist}" }
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
                        }
                    }
                }
            }

            // ─── POPOVER DE DETALHES ────────────────────────────────────────
            if let Some(pop) = popover.read().clone() {
                {
                    let found_app = appointments.read().iter()
                        .find(|a| a.id == pop.appointment_id)
                        .cloned();

                    if let Some(app) = found_app {
                        let (ah, am) = extract_hhmm(&app.scheduled_for);
                        let end_min = am + app.duration_minutes as u32;
                        let end_h = ah + end_min / 60;
                        let end_m = end_min % 60;
                        let day_display = format_display_date(&extract_date(&app.scheduled_for));
                        let time_display = format!("{:02}:{:02} - {:02}:{:02}", ah, am, end_h, end_m);
                        let patient_name = app.patient_name.clone().unwrap_or_else(|| app.title.clone());
                        let dentist_name = app.assigned_users.first().map(|u| u.user_name.clone().unwrap_or_default()).unwrap_or_default();
                        let dentist_init = initials(&dentist_name);
                        let day_name = {
                            let d = extract_date(&app.scheduled_for);
                            let nm = day_name_pt(&d);
                            let (_, m, dy) = parse_date(&d);
                            let months = ["jan","fev","mar","abr","mai","jun","jul","ago","set","out","nov","dez"];
                            format!("{}, {} {}.", nm, dy, months[(m-1) as usize])
                        };
                        let notes_text = app.notes.clone().unwrap_or_default();

                        // Calcula posição do popover (evita sair da tela)
                        let pop_x = (pop.x + 10.0).min(window_width() - 340.0);
                        let pop_y = pop.y.min(window_height() - 360.0);

                        let mut pop_close = popover.clone();
                        let app_id_delete = app.id.clone();
                        let mut apps_signal = appointments.clone();
                        let mut toast_del = toast_for_del.clone();
                        let app_id_edit = app.id.clone();
                        let mut pop_close2 = popover.clone();

                        rsx! {
                            div {
                                class: "event-popover-overlay",
                                onclick: move |_| pop_close.set(None),
                            }
                            div {
                                class: "event-popover",
                                style: format!("left: {}px; top: {}px;", pop_x, pop_y),

                                div { class: "popover-header",
                                    div { class: "popover-avatar", "👤" }
                                    div { class: "popover-patient-info",
                                        div { class: "popover-patient-name", "{patient_name}" }
                                        div { class: "popover-datetime",
                                            "{day_name} • {time_display}"
                                        }
                                    }
                                    div { class: "popover-actions",
                                        button {
                                            class: "popover-action-btn",
                                            title: "Editar",
                                            onclick: move |_| {
                                                pop_close2.set(None);
                                                // TODO: abrir modal de edição
                                            },
                                            "✏"
                                        }
                                        button {
                                            class: "popover-action-btn danger",
                                            title: "Excluir",
                                            onclick: move |_| {
                                                let aid = app_id_delete.clone();
                                                let mut apps = apps_signal.clone();
                                                let mut toast_c = toast_del.clone();
                                                let mut pop = pop_close.clone();
                                                spawn(async move {
                                                    match AppointmentsApi::delete_appointment(&aid).await {
                                                        Ok(_) => {
                                                            apps.write().retain(|a| a.id != aid);
                                                            toast_c.show("Consulta removida.", ToastVariant::Info);
                                                            pop.set(None);
                                                        }
                                                        Err(e) => {
                                                            web_sys::console::error_1(&e.clone().into());
                                                            toast_c.show(format!("Erro: {}", e), ToastVariant::Error);
                                                        }
                                                    }
                                                });
                                            },
                                            "✕"
                                        }
                                    }
                                }

                                div { class: "popover-body",
                                    // Status dropdown
                                    {
                                        let app_id_s = app.id.clone();
                                        let mut apps_s = appointments.clone();
                                        let mut toast_s = toast_for_status.clone();
                                        let current_status = appointment_status_label(&app.status);
                                        rsx! {
                                            select {
                                                class: "popover-status-select",
                                                value: "{current_status}",
                                                onchange: move |e| {
                                                    let new_status = match e.value().as_str() {
                                                        "Confirmada" => AppointmentStatus::Confirmed,
                                                        "Finalizada" => AppointmentStatus::Completed,
                                                        "Aguardando" => AppointmentStatus::InProgress,
                                                        "Pendente"   => AppointmentStatus::Pending,
                                                        "Cancelada"  => AppointmentStatus::Canceled,
                                                        _            => AppointmentStatus::Confirmed,
                                                    };
                                                    let aid = app_id_s.clone();
                                                    let mut apps = apps_s.clone();
                                                    let mut t = toast_s.clone();
                                                    spawn(async move {
                                                        use shared::appointments::UpdateAppointmentStatusRequest;
                                                        let req = UpdateAppointmentStatusRequest {
                                                            status: new_status,
                                                            cancellation_reason: None,
                                                            consumed_items: None,
                                                        };
                                                        match AppointmentsApi::update_appointment_status(&aid, req).await {
                                                            Ok(updated) => {
                                                                let mut lock = apps.write();
                                                                if let Some(a) = lock.iter_mut().find(|a| a.id == aid) {
                                                                    *a = updated;
                                                                }
                                                                t.show("Status atualizado.", ToastVariant::Success);
                                                            }
                                                            Err(e) => { t.show(format!("Erro: {}", e), ToastVariant::Error); }
                                                        }
                                                    });
                                                },
                                                option { "Confirmada" }
                                                option { "Aguardando" }
                                                option { "Finalizada" }
                                                option { "Pendente" }
                                                option { "Cancelada" }
                                            }
                                        }
                                    }

                                    // WhatsApp
                                    button { class: "popover-whatsapp-btn",
                                        "💬 Conversar por WhatsApp Web"
                                    }

                                    hr { class: "popover-divider" }

                                    // Dentista
                                    if !dentist_name.is_empty() {
                                        div { class: "popover-dentist-row",
                                            div { class: "popover-dentist-badge", "{dentist_init}" }
                                            span { class: "popover-dentist-name", "Dr(a). {dentist_name}" }
                                        }
                                    }

                                    // Observação
                                    if !notes_text.is_empty() {
                                        div { class: "popover-notes",
                                            "📝 {notes_text}"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }

            // ─── MODAL NOVA CONSULTA/COMPROMISSO ───────────────────────────
            if *show_new_modal.read() {
                AppointmentFormModal {
                    clinic_id: clinic_id.clone(),
                    prefill_date: prefill_date.read().clone(),
                    prefill_hour: prefill_hour.read().clone(),
                    on_close: move |_| show_new_modal.set(false),
                    on_open_new_patient: move |_| {
                        show_new_modal.set(false);
                        show_patient_modal.set(true);
                    },
                    on_saved: {
                        let cid = clinic_id.clone();
                        let mut apps = appointments.clone();
                        let mut toast_saved = toast_for_save.clone();
                        move |new_app: AppointmentResponse| {
                            apps.write().push(new_app);
                            toast_saved.show("Consulta agendada com sucesso!", ToastVariant::Success);
                            show_new_modal.set(false);
                        }
                    },
                }
            }

            // ─── MODAL NOVO PACIENTE ────────────────────────────────────────
            if *show_patient_modal.read() {
                PatientFormModal {
                    on_save: move |patient_id: String| {
                        show_patient_modal.set(false);
                        show_new_modal.set(true);
                        toast_for_patient.clone().show(format!("Paciente criado. ID: {}", patient_id), ToastVariant::Success);
                    },
                    on_close: move |_| {
                        show_patient_modal.set(false);
                        show_new_modal.set(true);
                    },
                }
            }
        }
    }
}

fn window_width() -> f64 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(1280.0)
}

fn window_height() -> f64 {
    web_sys::window()
        .and_then(|w| w.inner_height().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0)
}

// ─── MODAL DE AGENDAMENTO ─────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum AppointmentTab {
    Consulta,
    Compromisso,
}

#[derive(Props, Clone, PartialEq)]
struct AppointmentFormModalProps {
    clinic_id: String,
    prefill_date: String,
    prefill_hour: String,
    on_close: EventHandler<()>,
    on_open_new_patient: EventHandler<()>,
    on_saved: EventHandler<AppointmentResponse>,
}

#[component]
fn AppointmentFormModal(props: AppointmentFormModalProps) -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let mut tab = use_signal(|| AppointmentTab::Consulta);

    // Consulta fields
    let mut patient_search = use_signal(|| String::new());
    let mut patient_id = use_signal(|| Option::<String>::None);
    let mut patient_name_selected = use_signal(|| String::new());
    let mut show_autocomplete = use_signal(|| false);
    let mut patient_error = use_signal(|| false);

    // Shared fields
    let mut professional_id = use_signal(|| String::new());
    let mut consult_date = use_signal(|| props.prefill_date.clone());
    let mut start_time = use_signal(|| props.prefill_hour.clone());
    let mut duration = use_signal(|| 30i32);
    let mut notes = use_signal(|| String::new());
    let mut send_reminder = use_signal(|| false);

    // Compromisso fields
    let mut title_comp = use_signal(|| String::new());
    let mut desc_comp = use_signal(|| String::new());
    let mut end_time_comp = use_signal(|| {
        let h: u32 = props.prefill_hour.split(':').next().and_then(|s| s.parse().ok()).unwrap_or(9);
        format!("{:02}:30", h)
    });
    let mut all_day = use_signal(|| false);
    let mut title_error = use_signal(|| false);

    let mut is_saving = use_signal(|| false);

    // Profissionais disponíveis
    let professionals = use_signal(|| {
        if let Ok(db) = crate::api::mock_db::DB.lock() {
            db.users.iter()
                .map(|u| (u.id.clone(), u.full_name.clone()))
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    // Pacientes para autocomplete
    let patients_list = use_signal(|| {
        if let Ok(db) = crate::api::mock_db::DB.lock() {
            db.patients.iter()
                .map(|p| (p.id.clone(), p.full_name.clone(), p.phone.clone()))
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    let filtered_patients: Vec<(String, String, String)> = {
        let q = patient_search.read().to_lowercase();
        if q.is_empty() {
            vec![]
        } else {
            patients_list.read().iter()
                .filter(|(_, name, _)| name.to_lowercase().contains(&q))
                .take(6)
                .cloned()
                .collect()
        }
    };

    let on_close = props.on_close.clone();
    let on_saved = props.on_saved.clone();
    let on_open_new_patient = props.on_open_new_patient.clone();
    let clinic_id = props.clinic_id.clone();

    let handle_save = move |_| {
        match *tab.read() {
            AppointmentTab::Consulta => {
                if patient_id.read().is_none() {
                    patient_error.set(true);
                    return;
                }
                patient_error.set(false);
            }
            AppointmentTab::Compromisso => {
                if title_comp.read().trim().is_empty() {
                    title_error.set(true);
                    return;
                }
                title_error.set(false);
            }
        }

        is_saving.set(true);

        let cid = clinic_id.clone();
        let date_val = consult_date.read().clone();
        let time_val = start_time.read().clone();
        let scheduled = format!("{}T{}:00Z", date_val, time_val);
        let dur_val = *duration.read();
        let notes_val = if notes.read().is_empty() { None } else { Some(notes.read().clone()) };
        let pid = patient_id.read().clone();
        let pname = if patient_name_selected.read().is_empty() { None } else { Some(patient_name_selected.read().clone()) };
        let prof_id = professional_id.read().clone();

        let (app_type, title_val) = match *tab.read() {
            AppointmentTab::Consulta => (
                AppointmentType::Consultation,
                patient_name_selected.read().clone(),
            ),
            AppointmentTab::Compromisso => (
                AppointmentType::Meeting,
                title_comp.read().clone(),
            ),
        };

        let assigned: Vec<AssignedUserDto> = if !prof_id.is_empty() {
            let name = professionals.read().iter()
                .find(|(id, _)| id == &prof_id)
                .map(|(_, n)| n.clone())
                .unwrap_or_default();
            vec![AssignedUserDto { user_id: prof_id, user_name: Some(name), role_in_appointment: "dentist".into(), split_percentage: 100 }]
        } else { vec![] };

        let mut toast_save = toast.clone();
        let on_saved_clone = on_saved.clone();

        spawn(async move {
            let req = CreateAppointmentRequest {
                clinic_id: cid,
                patient_id: pid,
                patient_name: pname,
                treatment_id: None,
                treatment_plan_id: None,
                title: title_val,
                scheduled_for: scheduled,
                duration_minutes: dur_val,
                appointment_type: app_type,
                financial_amount_cents: None,
                financial_type: None,
                notes: notes_val,
                assigned_users: assigned,
                consumed_items: vec![],
                assigned_equipment: None,
            };
            match AppointmentsApi::create_appointment(req).await {
                Ok(app) => { on_saved_clone.call(app); }
                Err(e) => {
                    web_sys::console::error_1(&e.clone().into());
                    toast_save.show(format!("Erro ao agendar: {}", e), ToastVariant::Error);
                }
            }
            is_saving.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| {
                on_close.call(());
            },

            div { class: "modal-box", onclick: move |e| e.stop_propagation(),

                // Header com tabs
                div { class: "appointment-modal-header-full",
                    div { class: "appointment-modal-type-tabs",
                        button {
                            class: if *tab.read() == AppointmentTab::Consulta { "appt-type-tab type-consulta active" } else { "appt-type-tab type-consulta" },
                            onclick: move |_| tab.set(AppointmentTab::Consulta),
                            "Consulta"
                        }
                        button {
                            class: if *tab.read() == AppointmentTab::Compromisso { "appt-type-tab type-compromisso active" } else { "appt-type-tab type-compromisso" },
                            onclick: move |_| tab.set(AppointmentTab::Compromisso),
                            "Compromisso"
                        }
                    }
                    button { class: "modal-close-btn", onclick: move |_| on_close.call(()), "✕" }
                }

                div { class: "modal-body",

                    match *tab.read() {
                        AppointmentTab::Consulta => rsx! {
                            // Busca de paciente
                            div { class: "form-field",
                                label { class: "form-label", "Paciente *" }
                                div { class: "patient-search-wrap",
                                    input {
                                        class: if *patient_error.read() { "patient-search-input input-error" } else { "patient-search-input" },
                                        r#type: "text",
                                        placeholder: "Buscar paciente...",
                                        value: "{patient_search}",
                                        oninput: move |e| {
                                            patient_search.set(e.value());
                                            patient_id.set(None);
                                            patient_name_selected.set(String::new());
                                            show_autocomplete.set(!e.value().is_empty());
                                            if !e.value().is_empty() { patient_error.set(false); }
                                        },
                                        onfocus: move |_| {
                                            if !patient_search.read().is_empty() {
                                                show_autocomplete.set(true);
                                            }
                                        },
                                    }

                                    if *show_autocomplete.read() && !filtered_patients.is_empty() {
                                        div { class: "autocomplete-list",
                                            for (pid_opt, pname, pphone) in filtered_patients.iter() {
                                                {
                                                    let pid_v = pid_opt.clone();
                                                    let pname_v = pname.clone();
                                                    let pphone_v = pphone.clone();
                                                    let mut ps = patient_search.clone();
                                                    let mut pi = patient_id.clone();
                                                    let mut pns = patient_name_selected.clone();
                                                    let mut sac = show_autocomplete.clone();
                                                    rsx! {
                                                        div {
                                                            key: "{pid_v}",
                                                            class: "autocomplete-item",
                                                            onmousedown: move |_| {
                                                                ps.set(pname_v.clone());
                                                                pi.set(Some(pid_v.clone()));
                                                                pns.set(pname_v.clone());
                                                                sac.set(false);
                                                            },
                                                            span { class: "autocomplete-item-name", "{pname_v}" }
                                                            span { class: "autocomplete-item-sub", "{pphone_v}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                if *patient_error.read() {
                                    span { class: "field-required-msg", "Este campo é obrigatório" }
                                }

                                button {
                                    class: "new-patient-link",
                                    onclick: move |_| on_open_new_patient.call(()),
                                    "Cadastrar novo paciente"
                                }
                            }

                            // Profissional
                            div { class: "form-field",
                                label { class: "form-label", "Profissional *" }
                                select {
                                    class: "form-select",
                                    value: "{professional_id}",
                                    onchange: move |e| professional_id.set(e.value()),
                                    option { value: "", "Selecionar profissional" }
                                    for (pid_u, pname_u) in professionals.read().iter() {
                                        option { value: "{pid_u}", "{pname_u}" }
                                    }
                                }
                            }

                            // Data + Hora + Duração
                            div { class: "form-row-3 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Data da consulta *" }
                                    input { class: "form-input", r#type: "date", value: "{consult_date}",
                                        onchange: move |e| consult_date.set(e.value()) }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Hora de início *" }
                                    input { class: "form-input", r#type: "time", value: "{start_time}",
                                        onchange: move |e| start_time.set(e.value()) }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Duração (min) *" }
                                    input { class: "form-input", r#type: "number", min: "10", max: "480", step: "5",
                                        value: "{duration}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<i32>() { duration.set(v); }
                                        }
                                    }
                                }
                            }

                            // Observação
                            div { class: "form-field",
                                label { class: "form-label", "Observação" }
                                textarea { class: "form-textarea", rows: "3",
                                    placeholder: "Observações sobre a consulta...",
                                    value: "{notes}",
                                    oninput: move |e| notes.set(e.value()),
                                    maxlength: "500",
                                }
                                div { class: "char-counter", "{notes.read().len()} / 500" }
                            }

                            // Enviar confirmação
                            label { class: "form-checkbox-wrap",
                                input { r#type: "checkbox", checked: "{send_reminder}",
                                    onchange: move |e| send_reminder.set(e.checked()) }
                                "Enviar confirmação e lembrete de consulta automático"
                            }
                        },

                        AppointmentTab::Compromisso => rsx! {
                            // Título
                            div { class: "form-field",
                                label { class: "form-label", "Título do compromisso *" }
                                input {
                                    class: if *title_error.read() { "form-input input-error" } else { "form-input" },
                                    r#type: "text",
                                    placeholder: "Ex: Reunião de equipe",
                                    value: "{title_comp}",
                                    maxlength: "255",
                                    oninput: move |e| {
                                        title_comp.set(e.value());
                                        if !e.value().is_empty() { title_error.set(false); }
                                    },
                                }
                                div { class: "char-counter", "{title_comp.read().len()} / 255" }
                                if *title_error.read() {
                                    span { class: "field-required-msg", "Este campo é obrigatório" }
                                }
                            }

                            // Descrição
                            div { class: "form-field",
                                label { class: "form-label", "Descrição" }
                                textarea { class: "form-textarea", rows: "3",
                                    placeholder: "Detalhes do compromisso...",
                                    value: "{desc_comp}",
                                    maxlength: "500",
                                    oninput: move |e| desc_comp.set(e.value()),
                                }
                                div { class: "char-counter", "{desc_comp.read().len()} / 500" }
                            }

                            // Agenda de (profissional)
                            div { class: "form-field",
                                label { class: "form-label", "Agenda de *" }
                                select {
                                    class: "form-select",
                                    value: "{professional_id}",
                                    onchange: move |e| professional_id.set(e.value()),
                                    option { value: "", "Selecionar profissional" }
                                    for (pid_u, pname_u) in professionals.read().iter() {
                                        option { value: "{pid_u}", "{pname_u}" }
                                    }
                                }
                            }

                            // Data e hora
                            div {
                                style: "display: flex; flex-direction: column; gap: 10px;",
                                label { class: "form-label", "Data e hora" }

                                label { class: "form-checkbox-wrap",
                                    input { r#type: "checkbox", checked: "{all_day}",
                                        onchange: move |e| all_day.set(e.checked()) }
                                    "Dia inteiro"
                                }

                                if !*all_day.read() {
                                    div { class: "form-row-2 form-row",
                                        div { class: "form-field",
                                            label { class: "form-label", "Começa em *" }
                                            input { class: "form-input", r#type: "date", value: "{consult_date}",
                                                onchange: move |e| consult_date.set(e.value()) }
                                        }
                                        div { class: "form-field",
                                            label { class: "form-label", "Horário início *" }
                                            input { class: "form-input", r#type: "time", value: "{start_time}",
                                                onchange: move |e| start_time.set(e.value()) }
                                        }
                                    }
                                    div { class: "form-row-2 form-row",
                                        div { class: "form-field",
                                            label { class: "form-label", "Termina em *" }
                                            input { class: "form-input", r#type: "date", value: "{consult_date}",
                                                onchange: move |e| consult_date.set(e.value()) }
                                        }
                                        div { class: "form-field",
                                            label { class: "form-label", "Horário fim *" }
                                            input { class: "form-input", r#type: "time", value: "{end_time_comp}",
                                                onchange: move |e| end_time_comp.set(e.value()) }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                div { class: "modal-footer",
                    button { class: "btn-modal-ghost", onclick: move |_| on_close.call(()), "FECHAR" }
                    button {
                        class: "btn-modal-primary",
                        disabled: *is_saving.read(),
                        onclick: handle_save,
                        if *is_saving.read() { "Salvando..." } else { "MARCAR" }
                    }
                }
            }
        }
    }
}
