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
    on_success: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
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
            toast_msg.set(Some("Por favor, preencha o título do agendamento.".to_string()));
            return;
        }

        if assigned_users().is_empty() {
            toast_msg.set(Some("Selecione pelo menos um profissional responsável.".to_string()));
            return;
        }

        let scheduled_for = match chrono::NaiveDateTime::parse_from_str(
            &format!("{} {}:00", scheduled_date(), scheduled_time()),
            "%Y-%m-%d %H:%M:%S",
        ) {
            Ok(ndt) => {
                let local_dt = chrono::Local
                    .from_local_datetime(&ndt)
                    .single()
                    .unwrap_or_else(|| chrono::Local::now());
                local_dt.to_rfc3339()
            }
            Err(_) => {
                toast_msg.set(Some("Data ou horário inválido.".to_string()));
                return;
            }
        };

        let fin_amount_cents = if !financial_amount_str().trim().is_empty() {
            let clean_str = financial_amount_str().replace(',', ".");
            if let Ok(val_float) = clean_str.parse::<f64>() {
                Some((val_float * 100.0).round() as i64)
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
                    assigned_users: Some(assigned_users()),
                    consumed_items: Some(consumed_items()),
                };

                match update_appointment(&t, &a.id, &c, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        toast.set(Some("Agendamento atualizado com sucesso!".to_string()));
                        on_succ.call(());
                    }
                    Err(e) => {
                        toast.set(Some(format!("Erro ao atualizar agendamento: {}", e)));
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

                match create_appointment(&t, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        toast.set(Some("Agendamento criado com sucesso!".to_string()));
                        on_succ.call(());
                    }
                    Err(e) => {
                        toast.set(Some(format!("Erro ao criar agendamento: {}", e)));
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
            div { class: "action-modal stock-custom-modal", style: "max-width: 720px; max-height: 90vh; display: flex; flex-direction: column;",
                div { class: "settings-header",
                    div {
                        h2 { class: "settings-title", "{title_modal}" }
                        p { class: "text-muted font-xs mt-1", "Defina paciente, data, horário, profissionais e rateio financeiro." }
                    }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }

                div { class: "settings-content", style: "overflow-y: auto; gap: 18px; padding: 22px 26px;",
                    // 1. Título
                    div { class: "form-group",
                        label { "Título do Agendamento / Procedimento *" }
                        input {
                            class: "form-input",
                            placeholder: "Ex: Consulta Inicial & Avaliação Estética, Restauração...",
                            value: "{title}",
                            oninput: move |e| title.set(e.value())
                        }
                    }

                    // 2. Tipo de Atendimento & Duração Estimada
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Tipo de Atendimento" }
                            select {
                                class: "form-input",
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

                        div { class: "form-group",
                            label { "Duração Estimada" }
                            select {
                                class: "form-input",
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
                    }

                    // 3. Data do Atendimento & Horário de Início
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Data do Atendimento" }
                            input {
                                class: "form-input",
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

                        div { class: "form-group",
                            label { "Horário de Início (Padrão 24h)" }
                            select {
                                class: "form-input",
                                value: "{scheduled_time}",
                                onchange: move |e: FormEvent| scheduled_time.set(e.value()),
                                for t in &standard_times {
                                    option { key: "{t}", value: "{t}", "{t}" }
                                }
                            }
                        }
                    }

                    // 4. Paciente
                    div { class: "form-group",
                        label { "Paciente (Opcional)" }
                        select {
                            class: "form-input",
                            value: match patient_id() {
                                Some(ref pid) => pid.as_str(),
                                None => if !patient_name().is_empty() { "manual" } else { "" },
                            },
                            onchange: {
                                let pats = resources.patients.clone();
                                move |e: FormEvent| {
                                    let val = e.value();
                                    if val.is_empty() {
                                        patient_id.set(None);
                                        patient_name.set(String::new());
                                    } else if val == "manual" {
                                        patient_id.set(None);
                                    } else {
                                        if let Some(p) = pats.iter().find(|x| x.id == val) {
                                            patient_id.set(Some(p.id.clone()));
                                            patient_name.set(p.name.clone());
                                        }
                                    }
                                }
                            },
                            option { value: "", "Sem paciente vinculado" }
                            for pat in &resources.patients {
                                option { value: "{pat.id}", "{pat.name} {pat.extra_info.as_deref().unwrap_or(\"\")}" }
                            }
                            option { value: "manual", "Digitar nome de paciente não cadastrado..." }
                        }
                        if patient_id().is_none() && !patient_name().is_empty() {
                            input {
                                class: "form-input mt-2",
                                placeholder: "Nome do paciente avulso / novo...",
                                value: "{patient_name}",
                                oninput: move |e| patient_name.set(e.value())
                            }
                        }
                    }

                    // 5. Profissionais Responsáveis (Sem campo redundante de nome/cargo)
                    div { class: "form-group",
                        label { "Profissionais Responsáveis & Rateio *" }
                        if resources.team_members.is_empty() {
                            div { class: "empty-state-card py-3",
                                p { class: "text-muted font-xs", "Nenhum membro da equipe associado a esta unidade." }
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
                                                                        split_percentage: 100,
                                                                    });
                                                                }
                                                            } else {
                                                                curr.retain(|u| u.user_id != mid_chk);
                                                            }
                                                            assigned_users.set(curr);
                                                        }
                                                    }
                                                    span { class: "font-bold text-dark ml-1", "{member.name}" }
                                                    span { class: "{role_class} ml-2 font-xs", "{role_val}" }
                                                }

                                                if is_assigned {
                                                    div { class: "agenda-split-input-wrapper",
                                                        label { class: "text-muted font-xs", "Rateio:" }
                                                        input {
                                                            class: "form-input",
                                                            style: "width: 70px; height: 36px; text-align: center;",
                                                            r#type: "number",
                                                            min: "0",
                                                            max: "100",
                                                            value: "{split_val}",
                                                            oninput: move |e: FormEvent| {
                                                                if let Ok(v) = e.value().parse::<i32>() {
                                                                    let mut curr = assigned_users();
                                                                    if let Some(entry) = curr.iter_mut().find(|u| u.user_id == mid_split) {
                                                                        entry.split_percentage = v;
                                                                    }
                                                                    assigned_users.set(curr);
                                                                }
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

                    // 6. Financeiro & Faturamento Previsto
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Valor Previsto (R$)" }
                            div { class: "currency-input-wrapper",
                                span { class: "currency-prefix", "R$" }
                                input {
                                    class: "form-input currency-input-field",
                                    placeholder: "0,00",
                                    value: "{financial_amount_str}",
                                    oninput: move |e| financial_amount_str.set(e.value())
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "Tipo de Lançamento" }
                            select {
                                class: "form-input",
                                value: "{financial_type}",
                                onchange: move |e| financial_type.set(e.value()),
                                option { value: "income", "Receita (Entrada)" }
                                option { value: "expense", "Despesa (Saída)" }
                            }
                        }
                    }

                    // 7. Consumo de Materiais de Estoque
                    div { class: "form-group",
                        div { class: "stock-consumption-header",
                            label { class: "m-0 font-bold", "Consumo de Materiais de Estoque" }
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
                                span { "Adicionar Item" }
                            }
                        }

                        if consumed_items().is_empty() {
                            p { class: "text-muted font-xs m-0", "Nenhum material de estoque associado a este agendamento." }
                        } else {
                            div { class: "flex flex-col gap-2 mt-1",
                                for (idx, consumed) in consumed_items().iter().enumerate() {
                                    {
                                        let item_id_val = consumed.item_id.clone();
                                        let qty_val = consumed.quantity_planned;
                                        let inventory_items = resources.inventory_items.clone();

                                        rsx! {
                                            div { key: "{idx}", class: "stock-consumed-row",
                                                select {
                                                    class: "form-input",
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
                                                    class: "form-input",
                                                    style: "text-align: center;",
                                                    r#type: "number",
                                                    min: "1",
                                                    value: "{qty_val}",
                                                    oninput: move |e: FormEvent| {
                                                        if let Ok(q) = e.value().parse::<i32>() {
                                                            let mut curr = consumed_items();
                                                            if let Some(c) = curr.get_mut(idx) {
                                                                c.quantity_planned = q;
                                                                c.quantity_used = Some(q);
                                                            }
                                                            consumed_items.set(curr);
                                                        }
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
