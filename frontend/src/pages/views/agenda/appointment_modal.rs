//! # Modal de Criação e Edição de Agendamentos (Frontend)
//!
//! Exibe o formulário completo de agendamento de consultas com seleção de
//! paciente, procedimentos, profissionais responsáveis, materiais de estoque e financeiro.

use crate::api::{create_appointment, update_appointment};
use crate::components::icons::{IconPlus, IconTrash, IconUsers};
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, AppointmentType, AssignedUserDto,
    ConsumedItemDto, CreateAppointmentRequest, UpdateAppointmentRequest,
};

#[component]
pub fn AppointmentModal(
    token: String,
    clinic_id: String,
    editing_appointment: Option<AppointmentResponse>,
    default_date: String,
    default_time: String,
    resources: AgendaResourcesResponse,
    is_open: Signal<bool>,
    can_finance: bool,
    on_success: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {

    if !is_open() {
        return rsx! {};
    }

    let is_edit = editing_appointment.is_some();
    let title_modal = if is_edit {
        "Editar Agendamento"
    } else {
        "Novo Agendamento"
    };

    let mut title = use_signal(|| {
        editing_appointment
            .as_ref()
            .map(|a| a.title.clone())
            .unwrap_or_default()
    });

    let mut app_type = use_signal(|| {
        editing_appointment
            .as_ref()
            .map(|a| a.appointment_type)
            .unwrap_or(AppointmentType::Consultation)
    });

    let mut scheduled_date = use_signal(|| {
        if let Some(ref a) = editing_appointment {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.scheduled_for) {
                return dt.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
            }
        }
        if !default_date.is_empty() {
            default_date.clone()
        } else {
            chrono::Local::now().format("%Y-%m-%d").to_string()
        }
    });

    let mut scheduled_time = use_signal(|| {
        if let Some(ref a) = editing_appointment {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&a.scheduled_for) {
                return dt.with_timezone(&chrono::Local).format("%H:%M").to_string();
            }
        }
        if !default_time.is_empty() {
            default_time.clone()
        } else {
            "09:00".to_string()
        }
    });

    let mut duration_minutes = use_signal(|| {
        editing_appointment
            .as_ref()
            .map(|a| a.duration_minutes)
            .unwrap_or(30)
    });

    let mut patient_id = use_signal(|| {
        editing_appointment
            .as_ref()
            .and_then(|a| a.patient_id.clone())
    });

    let mut patient_name = use_signal(|| {
        editing_appointment
            .as_ref()
            .and_then(|a| a.patient_name.clone())
            .unwrap_or_default()
    });

    let mut selected_treatment_id = use_signal(|| {
        editing_appointment
            .as_ref()
            .and_then(|a| a.treatment_id.clone())
    });

    let mut assigned_users = use_signal(|| {
        if let Some(ref a) = editing_appointment {
            a.assigned_users.clone()
        } else if let Some(first_member) = resources.team_members.first() {
            vec![AssignedUserDto {
                user_id: first_member.id.clone(),
                user_name: Some(first_member.name.clone()),
                role_in_appointment: first_member
                    .extra_info
                    .clone()
                    .unwrap_or_else(|| "Dentista".to_string()),
                split_percentage: 100,
            }]
        } else {
            vec![]
        }
    });

    let mut consumed_items = use_signal(|| {
        editing_appointment
            .as_ref()
            .map(|a| a.consumed_items.clone())
            .unwrap_or_default()
    });

    let mut notes = use_signal(|| {
        editing_appointment
            .as_ref()
            .and_then(|a| a.notes.clone())
            .unwrap_or_default()
    });

    let mut assigned_equipment = use_signal(|| {
        editing_appointment
            .as_ref()
            .map(|a| a.assigned_equipment.clone())
            .unwrap_or_default()
    });

    let mut financial_amount_str = use_signal(|| {
        if let Some(ref a) = editing_appointment {
            if let Some(cents) = a.financial_amount_cents {
                return format!("{:.2}", cents as f64 / 100.0);
            }
        }
        String::new()
    });

    let mut financial_type = use_signal(|| {
        editing_appointment
            .as_ref()
            .and_then(|a| a.financial_type.clone())
            .unwrap_or_else(|| "income".to_string())
    });

    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();
    let editing_app_clone = editing_appointment.clone();

    let mut handle_submit = move |_| {
        if title().trim().is_empty() {
            let mut err = error_toast;
            err.set(Some("Por favor, preencha o título do agendamento.".to_string()));
            return;
        }

        if assigned_users().is_empty() {
            let mut err = error_toast;
            err.set(Some("Selecione pelo menos um profissional responsável.".to_string()));
            return;
        }

        let scheduled_for = match chrono::NaiveDateTime::parse_from_str(
            &format!("{} {}:00", scheduled_date(), scheduled_time()),
            "%Y-%m-%d %H:%M:%S",
        ) {
            Ok(ndt) => {
                let local_dt = chrono::Local.from_local_datetime(&ndt).single();
                if let Some(ldt) = local_dt {
                    ldt.to_utc().to_rfc3339()
                } else {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc)
                        .to_rfc3339()
                }
            }
            Err(_) => {
                let mut err = error_toast;
                err.set(Some("Data ou horário inválido.".to_string()));
                return;
            }
        };

        let fin_amount_cents = if can_finance {
            if let Ok(val) = financial_amount_str().replace(',', ".").parse::<f64>() {
                if val > 0.0 {
                    Some((val * 100.0).round() as i64)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };


        is_submitting.set(true);
        let t = tok.clone();
        let c = cid.clone();
        let edit_app = editing_app_clone.clone();
        let mut open_sig = is_open;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let on_succ = on_success.clone();

        spawn(async move {
            if let Some(ref a) = edit_app {
                let req = UpdateAppointmentRequest {
                    patient_id: patient_id(),
                    patient_name: if patient_name().trim().is_empty() {
                        None
                    } else {
                        Some(patient_name())
                    },
                    treatment_id: selected_treatment_id(),
                    treatment_plan_id: None,
                    title: Some(title()),
                    scheduled_for: Some(scheduled_for),
                    duration_minutes: Some(duration_minutes()),
                    appointment_type: Some(app_type()),
                    financial_amount_cents: fin_amount_cents,
                    financial_type: if fin_amount_cents.is_some() {
                        Some(financial_type())
                    } else {
                        None
                    },
                    notes: if notes().trim().is_empty() { None } else { Some(notes()) },
                    assigned_users: Some(assigned_users()),
                    consumed_items: Some(consumed_items()),
                    assigned_equipment: Some(assigned_equipment()),
                };

                match update_appointment(&t, &a.id, &c, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        toast.set(Some("Agendamento atualizado!".to_string()));
                        on_succ.call(());
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao atualizar agendamento: {}", e)));
                    }
                }
            } else {
                let req = CreateAppointmentRequest {
                    clinic_id: c.clone(),
                    patient_id: patient_id(),
                    patient_name: if patient_name().trim().is_empty() {
                        None
                    } else {
                        Some(patient_name())
                    },
                    treatment_id: selected_treatment_id(),
                    treatment_plan_id: None,
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
                    notes: if notes().trim().is_empty() { None } else { Some(notes()) },
                    assigned_users: assigned_users(),
                    consumed_items: consumed_items(),
                    assigned_equipment: Some(assigned_equipment()),
                };

                match create_appointment(&t, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        toast.set(Some("Agendamento criado!".to_string()));
                        on_succ.call(());
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao criar agendamento: {}", e)));
                    }
                }
            }
            sub_sig.set(false);
        });
    };

    use chrono::TimeZone;

    // Horários disponíveis com inserção do horário atual se customizado
    let mut standard_times: Vec<String> = Vec::new();
    for h in 7..=21 {
        for m in [0, 15, 30, 45] {
            standard_times.push(format!("{:02}:{:02}", h, m));
        }
    }
    let cur_time = scheduled_time();
    if !cur_time.is_empty() && !standard_times.contains(&cur_time) {
        standard_times.push(cur_time.clone());
        standard_times.sort();
    }

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal stock-custom-modal",
                div { class: "settings-header",
                    h2 { class: "settings-title", "{title_modal}" }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }

                div { class: "settings-content",
                    div { class: "form-grid",
                        // 1. Título do Agendamento
                        div { class: "input-group-wrapper full-width",
                            label { "Título do Agendamento / Procedimento *" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: Consulta Inicial & Avaliação Estética, Restauração...",
                                value: "{title}",
                                oninput: move |e| title.set(e.value())
                            }
                        }

                        // 2. Tipo de Atendimento & Duração Estimada
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
                                option { value: "60", "1 hora" }
                                option { value: "90", "1 hora e 30 min" }
                                option { value: "120", "2 horas" }
                                option { value: "180", "3 horas" }
                            }
                        }

                        // 3. Data do Atendimento & Horário de Início
                        div { class: "input-group-wrapper",
                            label { "Data do Atendimento" }
                            input {
                                class: "modern-input-field",
                                r#type: "date",
                                value: "{scheduled_date}",
                                oninput: move |e| scheduled_date.set(e.value())
                            }
                            div { class: "date-quick-buttons mt-1",
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        scheduled_date.set(chrono::Local::now().format("%Y-%m-%d").to_string());
                                    },
                                    "Hoje"
                                }
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        let tomorrow = chrono::Local::now() + chrono::Duration::days(1);
                                        scheduled_date.set(tomorrow.format("%Y-%m-%d").to_string());
                                    },
                                    "Amanhã"
                                }
                                button {
                                    r#type: "button",
                                    class: "quick-date-btn",
                                    onclick: move |_| {
                                        let next_week = chrono::Local::now() + chrono::Duration::days(7);
                                        scheduled_date.set(next_week.format("%Y-%m-%d").to_string());
                                    },
                                    "+7 dias"
                                }
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Horário de Início (Padrão 24h)" }
                            select {
                                class: "modern-input-field modern-select",
                                value: "{scheduled_time}",
                                onchange: move |e: FormEvent| scheduled_time.set(e.value()),
                                for t in &standard_times {
                                    option { key: "{t}", value: "{t}", "{t}" }
                                }
                            }
                        }

                        // 4. Seletor de Procedimento Pendente do Prontuário (Destaque Principal)
                        div { class: "input-group-wrapper full-width", style: "background: #f0fdf4; border: 1px solid #86efac; border-radius: 8px; padding: 12px; margin-bottom: 4px;",
                            label { class: "font-semibold text-emerald-900 flex items-center gap-1", "Procedimento Pendente do Prontuário" }
                            span { class: "text-xs text-emerald-700 block mb-2",
                                "Selecione um procedimento pendente para auto-preencher o paciente, título, valor e sincronizar o status."
                            }
                            select {
                                class: "modern-input-field modern-select",
                                value: selected_treatment_id().unwrap_or_default(),
                                onchange: move |e: FormEvent| {
                                    let v = e.value();
                                    if v.is_empty() {
                                        selected_treatment_id.set(None);
                                    } else {
                                        if let Some(treat) = resources.pending_treatments.iter().find(|t| t.id == v) {
                                            selected_treatment_id.set(Some(treat.id.clone()));
                                            patient_id.set(Some(treat.patient_id.clone()));
                                            patient_name.set(treat.patient_name.clone());
                                            title.set(format!("{} - {}", treat.procedure_name, treat.patient_name));
                                            app_type.set(AppointmentType::Treatment);
                                            if can_finance && treat.cost_cents > 0 {
                                                financial_amount_str.set(format!("{:.2}", (treat.cost_cents as f64) / 100.0));
                                            }
                                        }
                                    }
                                },
                                option { value: "", "Nenhum (Consulta Geral / Avulsa)" }
                                for t in &resources.pending_treatments {
                                    option { key: "{t.id}", value: "{t.id}",
                                        "[{t.patient_name}] {t.procedure_name}"
                                        if let Some(ref d) = t.tooth_number { " (Dente {d})" }
                                        " - R$ {(t.cost_cents as f64) / 100.0:.2}"
                                    }
                                }
                            }
                        }

                        // 5. Paciente Vinculado
                        div { class: "input-group-wrapper full-width",
                            label { "Paciente Vinculado" }
                            select {
                                class: "modern-input-field modern-select",
                                value: match patient_id() {
                                    Some(ref pid) => pid.as_str(),
                                    None => if !patient_name().is_empty() { "manual" } else { "" },
                                },
                                onchange: move |e: FormEvent| {
                                    let val = e.value();
                                    let pats = resources.patients.clone();
                                    if val.is_empty() {
                                        patient_id.set(None);
                                        patient_name.set(String::new());
                                    } else if val == "manual" {
                                        patient_id.set(None);
                                    } else {
                                        patient_id.set(Some(val.clone()));
                                        if let Some(p) = pats.iter().find(|x| x.id == val) {
                                            patient_name.set(p.name.clone());
                                        }
                                    }
                                },
                                option { value: "", "Sem paciente vinculado" }
                                option { value: "manual", "Digitar nome avulso..." }
                                for p in &resources.patients {
                                    option { key: "{p.id}", value: "{p.id}", "{p.name} {p.extra_info.as_deref().unwrap_or(\"\")}" }
                                }
                            }

                            if patient_id().is_none() && (!patient_name().is_empty() || patient_id().is_none()) {
                                input {
                                    class: "modern-input-field mt-1",
                                    placeholder: "Nome do paciente avulso",
                                    value: "{patient_name}",
                                    oninput: move |e| patient_name.set(e.value())
                                }
                            }
                        }

                        // 5. Profissionais Responsáveis & Rateio
                        div { class: "input-group-wrapper full-width",
                            label { "Profissionais Responsáveis & Rateio *" }
                            div { class: "agenda-resource-box",
                                if resources.team_members.is_empty() {
                                    div { class: "resource-empty-state",
                                        span { "Nenhum membro da equipe associado a esta unidade." }
                                    }
                                } else {
                                    for member in &resources.team_members {
                                        {
                                            let mid = member.id.clone();
                                            let mname = member.name.clone();
                                            let current_assigned = assigned_users();
                                            let existing_entry = current_assigned.iter().find(|u| u.user_id == mid);
                                            let is_assigned = existing_entry.is_some();
                                            let split_val = existing_entry.map(|u| u.split_percentage).unwrap_or(100);
                                            let role_val = existing_entry.map(|u| u.role_in_appointment.clone()).unwrap_or_else(|| member.extra_info.clone().unwrap_or_else(|| "Dentista".to_string()));

                                            let mid_chk = mid.clone();
                                            let mname_chk = mname.clone();
                                            let role_chk = role_val.clone();
                                            let mid_split = mid.clone();

                                            let role_class = match role_val.to_lowercase().as_str() {
                                                "admin" => "role-badge role-admin",
                                                _ => "role-badge role-dentist",
                                            };

                                            rsx! {
                                                div { key: "{member.id}", class: "agenda-member-assign-row",
                                                    label { class: "agenda-member-checkbox-label",
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
                                                                            split_percentage: 100,
                                                                        });
                                                                    }
                                                                } else {
                                                                    curr.retain(|u| u.user_id != mid_chk);
                                                                }
                                                                assigned_users.set(curr);
                                                            }
                                                        }
                                                        span { class: "agenda-member-name", "{member.name}" }
                                                        span { class: "{role_class} ml-2 font-xs flex-shrink-0", "{role_val}" }
                                                    }

                                                    if is_assigned && can_finance {
                                                        div { class: "agenda-split-input-wrapper",

                                                            label { class: "text-muted font-xs", "Rateio:" }
                                                            input {
                                                                class: "modern-input-field font-mono",
                                                                style: "width: 52px; height: 32px; text-align: center; padding: 2px 4px;",
                                                                r#type: "text",
                                                                inputmode: "numeric",
                                                                maxlength: "3",
                                                                placeholder: "100",
                                                                value: "{split_val}",
                                                                oninput: move |e: FormEvent| {
                                                                    let clean: String = e.value().chars().filter(|c| c.is_ascii_digit()).collect();
                                                                    let v = if clean.is_empty() {
                                                                        0
                                                                    } else {
                                                                        clean.parse::<i32>().unwrap_or(0).clamp(0, 100)
                                                                    };
                                                                    let mut curr = assigned_users();
                                                                    if let Some(entry) = curr.iter_mut().find(|u| u.user_id == mid_split) {
                                                                        entry.split_percentage = v;
                                                                    }
                                                                    assigned_users.set(curr);
                                                                }
                                                            }
                                                            span { class: "font-bold font-xs text-muted", "%" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 6. Equipamentos Odontológicos Alocados
                        div { class: "input-group-wrapper full-width",
                            div { class: "resource-section-header",
                                label { "Equipamentos Alocados" }
                                button {
                                    class: "btn-secondary btn-sm flex items-center gap-1",
                                    r#type: "button",
                                    onclick: move |_| {
                                        if let Some(first_eq) = resources.equipment_items.first() {
                                            let mut curr = assigned_equipment();
                                            curr.push(first_eq.name.clone());
                                            assigned_equipment.set(curr);
                                        }
                                    },
                                    IconPlus { size: 13, color: "currentColor".to_string() }
                                    span { "Adicionar Equipamento" }
                                }
                            }

                            div { class: "agenda-resource-box",
                                if assigned_equipment().is_empty() {
                                    div { class: "resource-empty-state",
                                        span { "Nenhum equipamento alocado para este agendamento." }
                                    }
                                } else {
                                    for (idx, eq_name) in assigned_equipment().iter().enumerate() {
                                        {
                                            let eq_name_val = eq_name.clone();

                                            rsx! {
                                                div { key: "{idx}", class: "stock-equipment-row",
                                                    select {
                                                        class: "modern-input-field modern-select",
                                                        value: "{eq_name_val}",
                                                        onchange: move |e: FormEvent| {
                                                            let new_name = e.value();
                                                            let mut curr = assigned_equipment();
                                                            if let Some(c) = curr.get_mut(idx) {
                                                                *c = new_name;
                                                            }
                                                            assigned_equipment.set(curr);
                                                        },
                                                        for item in &resources.equipment_items {
                                                            option { value: "{item.name}", "{item.name} {item.extra_info.as_deref().unwrap_or(\"\")}" }
                                                        }
                                                    }
                                                    button {
                                                        class: "btn-action-icon btn-action-danger",
                                                        r#type: "button",
                                                        title: "Remover equipamento",
                                                        onclick: move |_| {
                                                            let mut curr = assigned_equipment();
                                                            curr.remove(idx);
                                                            assigned_equipment.set(curr);
                                                        },
                                                        IconTrash { size: 14, color: "#ef4444".to_string() }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 7. Consumo de Materiais de Estoque
                        div { class: "input-group-wrapper full-width",
                            div { class: "resource-section-header",
                                label { "Consumo de Materiais de Estoque" }
                                button {
                                    class: "btn-secondary btn-sm flex items-center gap-1",
                                    r#type: "button",
                                    onclick: move |_| {
                                        if let Some(first_item) = resources.inventory_items.first() {
                                            let mut curr = consumed_items();
                                            curr.push(ConsumedItemDto {
                                                item_id: first_item.id.clone(),
                                                item_name: Some(first_item.name.clone()),
                                                quantity_planned: 1,
                                                quantity_used: Some(1),
                                            });
                                            consumed_items.set(curr);
                                        }
                                    },
                                    IconPlus { size: 13, color: "currentColor".to_string() }
                                    span { "Adicionar Material" }
                                }
                            }

                            div { class: "agenda-resource-box",
                                if consumed_items().is_empty() {
                                    div { class: "resource-empty-state",
                                        span { "Nenhum material de estoque associado a este agendamento." }
                                    }
                                } else {
                                    for (idx, consumed) in consumed_items().iter().enumerate() {
                                        {
                                            let item_id_val = consumed.item_id.clone();
                                            let qty_val = consumed.quantity_planned;
                                            let inventory_items = resources.inventory_items.clone();

                                            rsx! {
                                                div { key: "{idx}", class: "stock-consumed-row",
                                                    select {
                                                        class: "modern-input-field modern-select",
                                                        value: "{item_id_val}",
                                                        onchange: move |e: FormEvent| {
                                                            let new_id = e.value();
                                                            let mut curr = consumed_items();
                                                            if let Some(c) = curr.get_mut(idx) {
                                                                c.item_id = new_id.clone();
                                                                if let Some(s) = inventory_items.iter().find(|x| x.id == new_id) {
                                                                    c.item_name = Some(s.name.clone());
                                                                }
                                                            }
                                                            consumed_items.set(curr);
                                                        },
                                                        for item in &resources.inventory_items {
                                                            option { value: "{item.id}", "{item.name} {item.extra_info.as_deref().unwrap_or(\"\")}" }
                                                        }
                                                    }
                                                    input {
                                                        class: "modern-input-field font-mono",
                                                        style: "text-align: center; padding: 2px 4px;",
                                                        r#type: "text",
                                                        inputmode: "numeric",
                                                        maxlength: "4",
                                                        value: "{qty_val}",
                                                        oninput: move |e: FormEvent| {
                                                            let clean: String = e.value().chars().filter(|c| c.is_ascii_digit()).collect();
                                                            let q = if clean.is_empty() {
                                                                1
                                                            } else {
                                                                clean.parse::<i32>().unwrap_or(1).max(1)
                                                            };
                                                            let mut curr = consumed_items();
                                                            if let Some(c) = curr.get_mut(idx) {
                                                                c.quantity_planned = q;
                                                                c.quantity_used = Some(q);
                                                            }
                                                            consumed_items.set(curr);
                                                        }
                                                    }
                                                    button {
                                                        class: "btn-action-icon btn-action-danger",
                                                        r#type: "button",
                                                        title: "Remover item",
                                                        onclick: move |_| {
                                                            let mut curr = consumed_items();
                                                            curr.remove(idx);
                                                            consumed_items.set(curr);
                                                        },
                                                        IconTrash { size: 14, color: "#ef4444".to_string() }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 8. Financeiro & Faturamento Previsto
                        if can_finance {
                            div { class: "input-group-wrapper",
                                label { "Valor Previsto (R$)" }
                                div { class: "currency-input-wrapper",
                                    span { class: "currency-prefix", "R$" }
                                    input {
                                        class: "modern-input-field font-mono currency-input-field",
                                        placeholder: "0,00",
                                        value: "{financial_amount_str}",
                                        oninput: move |e| financial_amount_str.set(e.value())
                                    }
                                }
                            }

                            div { class: "input-group-wrapper",
                                label { "Tipo de Lançamento" }
                                select {
                                    class: "modern-input-field modern-select",
                                    value: "{financial_type}",
                                    onchange: move |e: FormEvent| financial_type.set(e.value()),
                                    option { value: "income", "Receita (Entrada)" }
                                    option { value: "expense", "Despesa (Saída)" }
                                }
                            }
                        }


                        // 9. Observações Clínicas & Recomendações
                        div { class: "input-group-wrapper full-width",
                            label { "Observações Clínicas & Recomendações Pré-Consulta" }
                            textarea {
                                class: "modern-input-field modern-textarea",
                                rows: "3",
                                placeholder: "Ex: Paciente com sensibilidade; necessita de profilaxia antes do procedimento...",
                                value: "{notes}",
                                oninput: move |e| notes.set(e.value())
                            }
                        }
                    }
                }

                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        if is_submitting() { "Salvando..." } else { "Salvar Agendamento" }
                    }
                }
            }
        }
    }
}
