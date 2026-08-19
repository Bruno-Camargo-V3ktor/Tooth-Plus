//! # Visualizações de Calendário e Linha do Tempo Contínua (Agenda Frontend)
//!
//! Controla a Timeline Diária com posicionamento proporcional ao horário e duração,
//! resolução automática de colisões/sobreposições, faixa horária dinâmica e cards completos.

use crate::components::icons::{
    IconCalendar, IconChevronLeft, IconChevronRight, IconClock, IconEdit, IconPlus,
    IconSearch, IconTooth, IconTrash, IconUsers,
};
use chrono::Datelike;
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, AppointmentStatus, AppointmentType,
};

const DEFAULT_START_HOUR: i32 = 7;
const DEFAULT_END_HOUR: i32 = 21;
const HOUR_HEIGHT_PX: f64 = 100.0;
const TIMELINE_TOP_PADDING: f64 = 24.0;
const CARD_WIDTH_PX: f64 = 440.0;
const CARD_GAP_PX: f64 = 14.0;

/// Formata a data ISO (YYYY-MM-DD) para exibição com dia da semana em português.
fn format_day_full(date_str: &str) -> String {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let weekday = match d.weekday() {
            chrono::Weekday::Mon => "Segunda-feira",
            chrono::Weekday::Tue => "Terça-feira",
            chrono::Weekday::Wed => "Quarta-feira",
            chrono::Weekday::Thu => "Quinta-feira",
            chrono::Weekday::Fri => "Sexta-feira",
            chrono::Weekday::Sat => "Sábado",
            chrono::Weekday::Sun => "Domingo",
        };
        format!("{}, {} de {}", weekday, d.format("%d"), match d.month() {
            1 => "janeiro", 2 => "fevereiro", 3 => "março", 4 => "abril",
            5 => "maio", 6 => "junho", 7 => "julho", 8 => "agosto",
            9 => "setembro", 10 => "outubro", 11 => "novembro", _ => "dezembro",
        })
    } else {
        date_str.to_string()
    }
}

/// Barra de ferramentas da agenda com navegação de datas e filtros integrados.
#[component]
pub fn AgendaToolbar(
    selected_date: Signal<String>,
    search_query: Signal<String>,
    mut filter_member: Signal<String>,
    mut filter_status: Signal<String>,
    mut filter_type: Signal<String>,
    resources: AgendaResourcesResponse,
    can_write: bool,
    on_new_appointment: EventHandler<()>,
) -> Element {
    let team_members = resources.team_members.clone();
    let mut date_sig = selected_date;

    let today_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let is_today = selected_date() == today_str;

    let handle_prev_day = move |_| {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&date_sig(), "%Y-%m-%d") {
            let prev = d - chrono::Duration::days(1);
            date_sig.set(prev.format("%Y-%m-%d").to_string());
        }
    };

    let handle_next_day = move |_| {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&date_sig(), "%Y-%m-%d") {
            let next = d + chrono::Duration::days(1);
            date_sig.set(next.format("%Y-%m-%d").to_string());
        }
    };

    let handle_today = move |_| {
        date_sig.set(chrono::Local::now().format("%Y-%m-%d").to_string());
    };

    let full_day_label = format_day_full(&selected_date());

    rsx! {
        div { class: "agenda-controls-wrapper mb-3",
            // Linha 1: Navegador de Datas com distribuição harmoniosa
            div { class: "agenda-nav-bar",
                div { class: "date-nav-group",
                    button {
                        class: "btn-secondary btn-icon-only",
                        title: "Dia anterior",
                        onclick: handle_prev_day,
                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                    }
                    button {
                        class: "btn-secondary btn-icon-only",
                        title: "Próximo dia",
                        onclick: handle_next_day,
                        IconChevronRight { size: 16, color: "currentColor".to_string() }
                    }

                    div { class: "date-picker-input-wrap",
                        input {
                            r#type: "date",
                            class: "native-date-input",
                            value: "{selected_date}",
                            oninput: move |e: FormEvent| date_sig.set(e.value())
                        }
                    }

                    span { class: "date-display-badge",
                        IconCalendar { size: 15, color: "currentColor".to_string() }
                        span { "{full_day_label}" }
                        if is_today {
                            span { class: "today-indicator-pill", "Hoje" }
                        }
                    }

                    if !is_today {
                        button {
                            class: "btn-today-return",
                            title: "Voltar para a data de hoje",
                            onclick: handle_today,
                            IconCalendar { size: 14, color: "currentColor".to_string() }
                            span { "Voltar para Hoje" }
                        }
                    }
                }
            }

            // Linha 2: Toolbar de Busca, Filtros e Ação
            div { class: "view-toolbar",
                div { class: "search-input-wrap flex-1", style: "max-width: 340px;",
                    IconSearch { size: 16, color: "#94a3b8".to_string() }
                    input {
                        class: "search-input-field",
                        placeholder: "Filtrar por título ou paciente...",
                        value: "{search_query}",
                        oninput: move |e: FormEvent| search_query.set(e.value())
                    }
                }

                div { class: "agenda-filters-group",
                    // 1. Filtro por Profissional
                    select {
                        class: "fin-cat-select",
                        value: "{filter_member}",
                        onchange: move |e: FormEvent| filter_member.set(e.value()),
                        option { value: "all", "Todos os Profissionais" }
                        for member in &team_members {
                            option { value: "{member.id}", "{member.name}" }
                        }
                    }

                    // 2. Filtro por Status
                    select {
                        class: "fin-cat-select",
                        value: "{filter_status}",
                        onchange: move |e: FormEvent| filter_status.set(e.value()),
                        option { value: "all", "Todos os Status" }
                        option { value: "pending", "Pendente" }
                        option { value: "confirmed", "Confirmado" }
                        option { value: "in_progress", "Em Atendimento" }
                        option { value: "completed", "Concluído" }
                        option { value: "canceled_by_doctor", "Cancelado (Doutor)" }
                        option { value: "canceled_by_patient", "Cancelado (Paciente)" }
                        option { value: "no_show", "Não Compareceu" }
                    }

                    // 3. Filtro por Tipo
                    select {
                        class: "fin-cat-select",
                        value: "{filter_type}",
                        onchange: move |e: FormEvent| filter_type.set(e.value()),
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
                            IconPlus { size: 16, color: "#ffffff".to_string() }
                            span { " Novo Agendamento" }
                        }
                    }
                }
            }
        }
    }
}

/// Estrutura para agendamento posicionado proporcionalmente no tempo.
#[derive(Clone, PartialEq)]
struct PositionedAppointment {
    app: AppointmentResponse,
    top_px: f64,
    height_px: f64,
    left_px: f64,
    width_px: f64,
    total_cols: usize,
}

/// Calcula o intervalo horário dinâmico (com base em 7h-21h e expandido se houver agendamentos fora do padrão).
fn compute_dynamic_hour_bounds(appointments: &[AppointmentResponse]) -> (i32, i32) {
    let mut min_hour = DEFAULT_START_HOUR;
    let mut max_hour = DEFAULT_END_HOUR;

    for a in appointments {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.scheduled_for) {
            let local_dt = dt.with_timezone(&chrono::Local);
            let start_h = local_dt.format("%H").to_string().parse::<i32>().unwrap_or(DEFAULT_START_HOUR);
            let start_m = local_dt.format("%M").to_string().parse::<i32>().unwrap_or(0);
            
            let end_total_minutes = start_h * 60 + start_m + a.duration_minutes;
            let end_h_ceil = (end_total_minutes + 59) / 60; // Arredondado para cima

            if start_h < min_hour {
                min_hour = start_h; // Arredondado para baixo
            }
            if end_h_ceil > max_hour {
                max_hour = end_h_ceil;
            }
        }
    }

    (min_hour.clamp(0, 23), max_hour.clamp(min_hour + 1, 24))
}

/// Calcula o posicionamento e resolução de colisões para os agendamentos do dia.
fn compute_timeline_layout(
    appointments: &[AppointmentResponse],
    start_hour: i32,
    hour_height_px: f64,
) -> Vec<PositionedAppointment> {
    let scale = hour_height_px / 60.0;

    struct RawItem {
        app: AppointmentResponse,
        start_min: i32,
        end_min: i32,
        real_end_min: i32,
    }

    let mut items: Vec<RawItem> = appointments
        .iter()
        .map(|a| {
            let (h, m) = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.scheduled_for) {
                let local_dt = dt.with_timezone(&chrono::Local);
                (
                    local_dt.format("%H").to_string().parse::<i32>().unwrap_or(start_hour),
                    local_dt.format("%M").to_string().parse::<i32>().unwrap_or(0),
                )
            } else {
                (start_hour, 0)
            };

            let start_min = (h - start_hour) * 60 + m;
            let real_dur = a.duration_minutes.max(15);
            let visual_dur = a.duration_minutes.max(50);
            let end_min = start_min + visual_dur;
            let real_end_min = start_min + real_dur;
            RawItem {
                app: a.clone(),
                start_min,
                end_min,
                real_end_min,
            }
        })
        .collect();

    // Ordenar por horário de início crescente e duração decrescente
    items.sort_by(|a, b| a.start_min.cmp(&b.start_min).then_with(|| b.end_min.cmp(&a.end_min)));

    // Agrupar em clusters de sobreposição
    let mut clusters: Vec<Vec<RawItem>> = Vec::new();
    let mut current_cluster: Vec<RawItem> = Vec::new();
    let mut cluster_max_end = -1;

    for item in items {
        if current_cluster.is_empty() {
            cluster_max_end = item.end_min;
            current_cluster.push(item);
        } else if item.start_min < cluster_max_end {
            cluster_max_end = cluster_max_end.max(item.end_min);
            current_cluster.push(item);
        } else {
            clusters.push(std::mem::take(&mut current_cluster));
            cluster_max_end = item.end_min;
            current_cluster.push(item);
        }
    }
    if !current_cluster.is_empty() {
        clusters.push(current_cluster);
    }

    let mut result: Vec<PositionedAppointment> = Vec::new();

    for cluster in clusters {
        let mut column_ends: Vec<i32> = Vec::new();
        struct Placed {
            app: AppointmentResponse,
            start_min: i32,
            end_min: i32,
            real_end_min: i32,
            col: usize,
        }
        let mut placed_items: Vec<Placed> = Vec::new();

        for item in cluster {
            let mut assigned_col = None;
            for (col_idx, &end_time) in column_ends.iter().enumerate() {
                if end_time <= item.start_min {
                    assigned_col = Some(col_idx);
                    break;
                }
            }

            let col = match assigned_col {
                Some(c) => {
                    column_ends[c] = item.end_min;
                    c
                }
                None => {
                    column_ends.push(item.end_min);
                    column_ends.len() - 1
                }
            };

            placed_items.push(Placed {
                app: item.app,
                start_min: item.start_min,
                end_min: item.end_min,
                real_end_min: item.real_end_min,
                col,
            });
        }

        let total_cols = column_ends.len().max(1);

        for p in placed_items {
            let top_px = TIMELINE_TOP_PADDING + (p.start_min as f64 * scale).max(0.0);
            let actual_dur_px = (p.real_end_min - p.start_min) as f64 * scale;
            let height_px = (actual_dur_px - 4.0).max(85.0);
            let left_px = 12.0 + (p.col as f64 * (CARD_WIDTH_PX + CARD_GAP_PX));
            let width_px = CARD_WIDTH_PX;

            result.push(PositionedAppointment {
                app: p.app,
                top_px,
                height_px,
                left_px,
                width_px,
                total_cols,
            });
        }
    }

    result
}

/// Visualização da grade contínua proporcional no tempo com layout inspirado no card de estoque.
#[component]
pub fn DayTimelineView(
    appointments: Vec<AppointmentResponse>,
    can_write: bool,
    can_delete: bool,
    can_finance: bool,
    on_slot_click: EventHandler<i32>,
    on_edit: EventHandler<AppointmentResponse>,
    on_status_change: EventHandler<AppointmentResponse>,
    on_delete: EventHandler<AppointmentResponse>,
) -> Element {
    let (start_hour, end_hour) = compute_dynamic_hour_bounds(&appointments);
    let total_hours = end_hour - start_hour;
    let total_height = TIMELINE_TOP_PADDING + (total_hours as f64) * HOUR_HEIGHT_PX + 40.0;

    let positioned_apps = compute_timeline_layout(&appointments, start_hour, HOUR_HEIGHT_PX);
    let max_cols = positioned_apps.iter().map(|p| p.total_cols).max().unwrap_or(1);
    let min_canvas_width = 12.0 + (max_cols as f64 * (CARD_WIDTH_PX + CARD_GAP_PX)) + 24.0;
    let min_grid_width = min_canvas_width + 85.0;

    rsx! {
        div { class: "timeline-calendar-wrapper",
            div { class: "timeline-grid-body", style: "height: {total_height}px; min-height: {total_height}px; min-width: {min_grid_width}px;",
                // 1. Coluna de Horários (Gutter) Dinâmica com alinhamento na linha
                div { class: "timeline-time-gutter", style: "height: {total_height}px;",
                    for h in start_hour..=end_hour {
                        {
                            let h_label = format!("{:02}:00", h);
                            let top_pos = TIMELINE_TOP_PADDING + ((h - start_hour) as f64) * HOUR_HEIGHT_PX;
                            rsx! {
                                div {
                                    key: "{h}",
                                    class: "timeline-gutter-hour",
                                    style: "top: {top_pos}px;",
                                    span { "{h_label}" }
                                }
                            }
                        }
                    }
                }

                // 2. Área do Canvas da Timeline
                div {
                    class: "timeline-canvas-container",
                    style: "height: {total_height}px; min-width: {min_canvas_width}px;",
                    onclick: move |e: MouseEvent| {
                        if can_write {
                            let coords = e.element_coordinates();
                            let y = coords.y;
                            let clicked_hour = start_hour + ((y - TIMELINE_TOP_PADDING).max(0.0) / HOUR_HEIGHT_PX).floor() as i32;
                            let clamped_hour = clicked_hour.clamp(start_hour, end_hour);
                            on_slot_click.call(clamped_hour);
                        }
                    },

                    // Renderização de cada Agendamento Proporcional
                    for item in positioned_apps {
                        {
                            let app = item.app.clone();
                            let app_edit = app.clone();
                            let app_status = app.clone();
                            let app_del = app.clone();

                            let top_style = format!("{:.1}px", item.top_px);
                            let height_style = format!("{:.1}px", item.height_px);
                            let left_style = format!("{:.1}px", item.left_px);
                            let width_style = format!("{:.1}px", item.width_px);

                            let single_card_class = if item.total_cols == 1 { "single-column-card" } else { "multi-column-card" };

                            let type_class = match app.appointment_type {
                                AppointmentType::Consultation => "type-consultation",
                                AppointmentType::Treatment => "type-treatment",
                                AppointmentType::Surgery => "type-surgery",
                                AppointmentType::Return => "type-return",
                                AppointmentType::Meeting => "type-meeting",
                                AppointmentType::Other => "type-other",
                            };

                            let type_tag_class = match app.appointment_type {
                                AppointmentType::Consultation => "tag-consultation",
                                AppointmentType::Treatment => "tag-treatment",
                                AppointmentType::Surgery => "tag-surgery",
                                AppointmentType::Return => "tag-return",
                                AppointmentType::Meeting => "tag-meeting",
                                AppointmentType::Other => "tag-other",
                            };

                            let time_label = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&app.scheduled_for) {
                                let local_dt = dt.with_timezone(&chrono::Local);
                                let end_dt = local_dt + chrono::Duration::minutes(app.duration_minutes as i64);
                                format!("{} - {} ({}m)", local_dt.format("%H:%M"), end_dt.format("%H:%M"), app.duration_minutes)
                            } else {
                                format!("{} min", app.duration_minutes)
                            };

                            let fin_badge = if can_finance {
                                if let Some(amount_cents) = app.financial_amount_cents {
                                    let reais = amount_cents as f64 / 100.0;
                                    let prefix = if app.financial_type.as_deref().unwrap_or("income") == "income" { "+ R$" } else { "- R$" };
                                    Some(format!("{} {:.2}", prefix, reais))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };


                            rsx! {
                                div {
                                    key: "{app.id}",
                                    class: "timeline-app-card {type_class} {single_card_class}",
                                    style: "top: {top_style}; min-height: {height_style}; left: {left_style}; width: {width_style};",
                                    onclick: move |e| e.stop_propagation(),

                                    // Header no Estilo do Estoque: Tags à esquerda + Ações à direita
                                    div { class: "timeline-card-header",
                                        div { class: "stock-badges-group",
                                            span { class: "stock-tag {type_tag_class}", "{app.appointment_type.label()}" }
                                            button {
                                                class: "app-status-badge {app.status.color_class()}",
                                                r#type: "button",
                                                onclick: move |_| on_status_change.call(app_status.clone()),
                                                title: "Clique para alterar status",
                                                "{app.status.label()}"
                                            }
                                        }

                                        div { class: "card-header-actions",
                                            if can_write {
                                                button {
                                                    class: "btn-action-icon",
                                                    style: "width: 26px; height: 26px;",
                                                    onclick: move |_| on_edit.call(app_edit.clone()),
                                                    title: "Editar Agendamento",
                                                    IconEdit { size: 13, color: "#475569".to_string() }
                                                }
                                            }
                                            if can_delete {
                                                button {
                                                    class: "btn-action-icon btn-action-danger",
                                                    style: "width: 26px; height: 26px;",
                                                    onclick: move |_| on_delete.call(app_del.clone()),
                                                    title: "Excluir Agendamento",
                                                    IconTrash { size: 13, color: "#ef4444".to_string() }
                                                }
                                            }
                                        }
                                    }

                                    // Corpo do Card: Título, Paciente, Doutor e Pills
                                    div { class: "timeline-card-body",
                                        h4 { class: "timeline-card-title", "{app.title}" }

                                        div { class: "timeline-card-parties",
                                            if let Some(ref p_name) = app.patient_name {
                                                span { class: "timeline-chip-patient",
                                                    IconUsers { size: 12, color: "#2563eb".to_string() }
                                                    span { "Paciente: {p_name}" }
                                                }
                                            }

                                            if !app.assigned_users.is_empty() {
                                                for user in &app.assigned_users {
                                                    span { class: "timeline-chip-doctor",
                                                        IconTooth { size: 12, color: "#059669".to_string() }
                                                        span { "Dr(a): {user.user_name.as_deref().unwrap_or(&user.role_in_appointment)}" }
                                                    }
                                                }
                                            }
                                        }

                                        // Linha de Metadados / Pills (Horário, Equipamentos, Finanças)
                                        div { class: "timeline-card-meta-row",
                                            span { class: "timeline-meta-pill pill-time",
                                                IconClock { size: 11, color: "#475569".to_string() }
                                                span { "{time_label}" }
                                            }

                                            if !app.assigned_equipment.is_empty() {
                                                span { class: "timeline-meta-pill pill-equip",
                                                    "🛠️ "
                                                    for (idx, eq) in app.assigned_equipment.iter().enumerate() {
                                                        if idx > 0 { ", " }
                                                        "{eq}"
                                                    }
                                                }
                                            }

                                            if let Some(ref fin_text) = fin_badge {
                                                span { class: "timeline-meta-pill pill-fin", "{fin_text}" }
                                            }
                                        }

                                        // Alerta de Cancelamento (pelo Doutor ou Paciente) com Observações
                                        if app.status.is_canceled() {
                                            div {
                                                class: if app.status == AppointmentStatus::CanceledByDoctor {
                                                    "timeline-cancel-alert cancel-by-doc"
                                                } else {
                                                    "timeline-cancel-alert cancel-by-pat"
                                                },
                                                strong {
                                                    if app.status == AppointmentStatus::CanceledByDoctor {
                                                        "⚠️ Cancelado pelo Doutor / Clínica: "
                                                    } else {
                                                        "⚠️ Cancelado pelo Paciente: "
                                                    }
                                                }
                                                span { "{app.cancellation_reason.as_deref().unwrap_or(\"Sem observações informadas.\")}" }
                                            }
                                        }

                                        // Observações Clínicas
                                        if let Some(ref notes_text) = app.notes {
                                            if !notes_text.trim().is_empty() {
                                                p { class: "timeline-card-notes",
                                                    span { class: "font-semibold", "📝 Obs: " }
                                                    span { "{notes_text}" }
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
