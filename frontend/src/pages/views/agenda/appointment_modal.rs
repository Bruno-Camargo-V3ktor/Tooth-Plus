//! # Modal de Cadastro e Edição de Agendamentos (Frontend)
//!
//! Controla o formulário de marcação de consultas odontológicas, seleção de
//! profissionais responsáveis, divisões percentuais de repasse e materiais planejados.

use crate::api::{create_appointment, update_appointment};
use crate::components::icons::{IconCheck, IconPlus, IconTrash};
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, AppointmentType, AssignedUserDto, ConsumedItemDto,
    CreateAppointmentRequest, UpdateAppointmentRequest,
};

/// Converte enum `AppointmentType` em string.
fn type_to_str(t: &AppointmentType) -> &'static str {
    match t {
        AppointmentType::Consultation => "consultation",
        AppointmentType::Treatment => "treatment",
        AppointmentType::Surgery => "surgery",
        AppointmentType::Return => "return",
        AppointmentType::Meeting => "meeting",
        AppointmentType::Other => "other",
    }
}

/// Converte string em enum `AppointmentType`.
fn str_to_type(s: &str) -> AppointmentType {
    match s {
        "treatment" => AppointmentType::Treatment,
        "surgery" => AppointmentType::Surgery,
        "return" => AppointmentType::Return,
        "meeting" => AppointmentType::Meeting,
        "other" => AppointmentType::Other,
        _ => AppointmentType::Consultation,
    }
}

/// Modal de agendamento e edição de consultas.
#[component]
pub fn AppointmentModal(
    token: String,
    clinic_id: String,
    editing_appointment: Option<AppointmentResponse>,
    default_date: String,
    default_time: String,
    resources: AgendaResourcesResponse,
    is_open: Signal<bool>,
    on_success: EventHandler<String>,
    toast_msg: Signal<Option<String>>,
) -> Element {
    let initial_app = editing_appointment.clone();
    let is_editing = initial_app.is_some();
    let edit_id = initial_app.as_ref().map(|a| a.id.clone()).unwrap_or_default();

    let mut form_title = use_signal(|| initial_app.as_ref().map(|a| a.title.clone()).unwrap_or_default());
    let mut form_patient_id = use_signal(|| initial_app.as_ref().and_then(|a| a.patient_id.clone()).unwrap_or_default());
    let mut form_patient_name = use_signal(|| initial_app.as_ref().and_then(|a| a.patient_name.clone()).unwrap_or_default());
    let mut form_date = use_signal(|| {
        initial_app
            .as_ref()
            .map(|a| a.scheduled_for.chars().take(10).collect::<String>())
            .unwrap_or_else(|| default_date.clone())
    });
    let mut form_time = use_signal(|| {
        initial_app
            .as_ref()
            .map(|a| {
                if a.scheduled_for.len() >= 16 {
                    a.scheduled_for[11..16].to_string()
                } else {
                    "09:00".to_string()
                }
            })
            .unwrap_or_else(|| default_time.clone())
    });
    let mut form_duration = use_signal(|| initial_app.as_ref().map(|a| a.duration_minutes).unwrap_or(30));
    let mut form_type = use_signal(|| {
        initial_app
            .as_ref()
            .map(|a| type_to_str(&a.appointment_type).to_string())
            .unwrap_or_else(|| "consultation".into())
    });
    let mut form_financial_reais = use_signal(|| {
        initial_app
            .as_ref()
            .and_then(|a| a.financial_amount_cents)
            .map(|cents| format!("{:.2}", (cents as f64) / 100.0))
            .unwrap_or_else(|| "0,00".into())
    });
    let mut form_assigned_users = use_signal(|| {
        initial_app
            .as_ref()
            .map(|a| a.assigned_users.clone())
            .unwrap_or_else(|| {
                if let Some(first_member) = resources.team_members.first() {
                    vec![AssignedUserDto {
                        user_id: first_member.id.clone(),
                        user_name: Some(first_member.name.clone()),
                        role_in_appointment: "Dentista Principal".into(),
                        split_percentage: 100,
                    }]
                } else {
                    vec![]
                }
            })
    });
    let mut form_consumed_items = use_signal(|| {
        initial_app
            .as_ref()
            .map(|a| a.consumed_items.clone())
            .unwrap_or_default()
    });
    let mut is_submitting = use_signal(|| false);

    if !is_open() {
        return rsx! {};
    }

    let tok = token.clone();
    let cid = clinic_id.clone();
    let res_clone = resources.clone();

    let mut handle_submit = move |_| {
        let title = form_title().trim().to_string();
        if title.is_empty() {
            let mut toast = toast_msg;
            toast.set(Some("Informe o título do agendamento.".into()));
            return;
        }

        if form_assigned_users().is_empty() {
            let mut toast = toast_msg;
            toast.set(Some("Vincule ao menos um profissional responsável.".into()));
            return;
        }

        let datetime_str = format!("{}T{}:00Z", form_date(), form_time());
        let amount_clean = form_financial_reais().replace("R$", "").replace(".", "").replace(",", "").trim().to_string();
        let amount_cents = amount_clean.parse::<i64>().unwrap_or(0);

        let patient_id_opt = if form_patient_id().trim().is_empty() {
            None
        } else {
            Some(form_patient_id().trim().to_string())
        };

        let patient_name_opt = if form_patient_name().trim().is_empty() {
            None
        } else {
            Some(form_patient_name().trim().to_string())
        };

        let t = tok.clone();
        let c = cid.clone();
        let e_id = edit_id.clone();
        let mut open_sig = is_open;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;
        let on_succ = on_success.clone();

        sub_sig.set(true);
        spawn(async move {
            if is_editing {
                let req = UpdateAppointmentRequest {
                    patient_id: patient_id_opt,
                    patient_name: patient_name_opt,
                    title: Some(title),
                    scheduled_for: Some(datetime_str),
                    duration_minutes: Some(form_duration()),
                    appointment_type: Some(str_to_type(&form_type())),
                    financial_amount_cents: Some(amount_cents),
                    financial_type: Some(if amount_cents > 0 { "income".into() } else { "none".into() }),
                    assigned_users: Some(form_assigned_users()),
                    consumed_items: Some(form_consumed_items()),
                };
                match update_appointment(&t, &e_id, &c, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        toast.set(Some("Agendamento atualizado com sucesso!".into()));
                        on_succ.call("Atualizado".into());
                    }
                    Err(e) => {
                        toast.set(Some(format!("Erro ao atualizar: {}", e)));
                    }
                }
            } else {
                let req = CreateAppointmentRequest {
                    clinic_id: c,
                    patient_id: patient_id_opt,
                    patient_name: patient_name_opt,
                    title,
                    scheduled_for: datetime_str,
                    duration_minutes: form_duration(),
                    appointment_type: str_to_type(&form_type()),
                    financial_amount_cents: Some(amount_cents),
                    financial_type: Some(if amount_cents > 0 { "income".into() } else { "none".into() }),
                    assigned_users: form_assigned_users(),
                    consumed_items: form_consumed_items(),
                };
                match create_appointment(&t, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        toast.set(Some("Agendamento criado com sucesso!".into()));
                        on_succ.call("Criado".into());
                    }
                    Err(e) => {
                        toast.set(Some(format!("Erro ao criar: {}", e)));
                    }
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal modal-large",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", if is_editing { "Editar Agendamento" } else { "Novo Agendamento na Agenda" } }
                        p { class: "modal-subtitle", "Preencha os dados do agendamento, selecione o paciente e profissionais responsáveis." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }
                div { class: "modal-body scrollable",
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Título / Procedimento *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Consulta Inicial / Avaliação Ortodôntica",
                                value: "{form_title}",
                                oninput: move |e| form_title.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Tipo de Atendimento" }
                            select {
                                class: "form-input",
                                value: "{form_type}",
                                onchange: move |e| form_type.set(e.value()),
                                option { value: "consultation", "Consulta / Avaliação" }
                                option { value: "treatment", "Tratamento / Procedimento" }
                                option { value: "surgery", "Cirurgia Odontológica" }
                                option { value: "return", "Retorno / Revisão" }
                                option { value: "meeting", "Reunião / Outro" }
                            }
                        }
                    }

                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Paciente Cadastrado" }
                            select {
                                class: "form-input",
                                value: "{form_patient_id}",
                                onchange: move |e| {
                                    let val = e.value();
                                    form_patient_id.set(val.clone());
                                    if let Some(p) = res_clone.patients.iter().find(|p| p.id == val) {
                                        form_patient_name.set(p.name.clone());
                                    }
                                },
                                option { value: "", "Paciente não cadastrado / Avulso" }
                                for p in &res_clone.patients {
                                    option { value: "{p.id}", "{p.name} ({p.extra_info.as_deref().unwrap_or(\"\")})" }
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "Nome do Paciente Avulso" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Maria Santos",
                                value: "{form_patient_name}",
                                oninput: move |e| form_patient_name.set(e.value())
                            }
                        }
                    }

                    div { class: "form-grid-3",
                        div { class: "form-group",
                            label { "Data *" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                value: "{form_date}",
                                oninput: move |e| form_date.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Horário *" }
                            input {
                                class: "form-input",
                                r#type: "time",
                                value: "{form_time}",
                                oninput: move |e| form_time.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Duração (minutos)" }
                            select {
                                class: "form-input",
                                value: "{form_duration}",
                                onchange: move |e| form_duration.set(e.value().parse::<i32>().unwrap_or(30)),
                                option { value: "15", "15 minutos" }
                                option { value: "30", "30 minutos" }
                                option { value: "45", "45 minutos" }
                                option { value: "60", "1 hora (60 min)" }
                                option { value: "90", "1 hora e meia (90 min)" }
                                option { value: "120", "2 horas (120 min)" }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Valor Financeiro Previsto (R$)" }
                        input {
                            class: "form-input",
                            placeholder: "0,00",
                            value: "{form_financial_reais}",
                            oninput: move |e| form_financial_reais.set(e.value())
                        }
                    }

                    div { class: "form-section-title mt-4", "Profissionais Responsáveis" }
                    div { class: "team-assignment-list",
                        for (idx, user) in form_assigned_users().iter().enumerate() {
                            {
                                let u_id = user.user_id.clone();
                                let u_role = user.role_in_appointment.clone();
                                let u_split = user.split_percentage;

                                rsx! {
                                    div { key: "{u_id}", class: "form-grid-3 mb-2",
                                        select {
                                            class: "form-input",
                                            value: "{u_id}",
                                            onchange: move |e| {
                                                let mut users = form_assigned_users();
                                                if let Some(u) = users.get_mut(idx) {
                                                    u.user_id = e.value();
                                                }
                                                form_assigned_users.set(users);
                                            },
                                            for m in &res_clone.team_members {
                                                option { value: "{m.id}", "{m.name} ({m.extra_info.as_deref().unwrap_or(\"\")})" }
                                            }
                                        }
                                        input {
                                            class: "form-input",
                                            placeholder: "Papel (Ex: Dentista Principal)",
                                            value: "{u_role}",
                                            oninput: move |e| {
                                                let mut users = form_assigned_users();
                                                if let Some(u) = users.get_mut(idx) {
                                                    u.role_in_appointment = e.value();
                                                }
                                                form_assigned_users.set(users);
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            input {
                                                class: "form-input",
                                                r#type: "number",
                                                min: "0",
                                                max: "100",
                                                placeholder: "% Repasse",
                                                value: "{u_split}",
                                                oninput: move |e| {
                                                    let val = e.value().parse::<i32>().unwrap_or(0);
                                                    let mut users = form_assigned_users();
                                                    if let Some(u) = users.get_mut(idx) {
                                                        u.split_percentage = val;
                                                    }
                                                    form_assigned_users.set(users);
                                                }
                                            }
                                            button {
                                                class: "btn-icon text-danger",
                                                onclick: move |_| {
                                                    let mut users = form_assigned_users();
                                                    users.remove(idx);
                                                    form_assigned_users.set(users);
                                                },
                                                IconTrash { size: 14, color: "currentColor".to_string() }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "form-section-title mt-4", "Planejamento de Insumos / Estoque" }
                    div { class: "consumed-planning-list",
                        for (idx, item) in form_consumed_items().iter().enumerate() {
                            {
                                let item_id = item.item_id.clone();
                                let planned_qty = item.quantity_planned;

                                rsx! {
                                    div { key: "{item_id}", class: "form-grid-3 mb-2",
                                        select {
                                            class: "form-input",
                                            value: "{item_id}",
                                            onchange: move |e| {
                                                let mut items = form_consumed_items();
                                                if let Some(it) = items.get_mut(idx) {
                                                    it.item_id = e.value();
                                                }
                                                form_consumed_items.set(items);
                                            },
                                            for inv in &res_clone.inventory_items {
                                                option { value: "{inv.id}", "{inv.name} ({inv.extra_info.as_deref().unwrap_or(\"\")})" }
                                            }
                                        }
                                        input {
                                            class: "form-input",
                                            r#type: "number",
                                            min: "1",
                                            placeholder: "Qtd. Planejada",
                                            value: "{planned_qty}",
                                            oninput: move |e| {
                                                let val = e.value().parse::<i32>().unwrap_or(1);
                                                let mut items = form_consumed_items();
                                                if let Some(it) = items.get_mut(idx) {
                                                    it.quantity_planned = val;
                                                }
                                                form_consumed_items.set(items);
                                            }
                                        }
                                        button {
                                            class: "btn-icon text-danger",
                                            onclick: move |_| {
                                                let mut items = form_consumed_items();
                                                items.remove(idx);
                                                form_consumed_items.set(items);
                                            },
                                            IconTrash { size: 14, color: "currentColor".to_string() }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(first_inv) = res_clone.inventory_items.first() {
                            {
                                let f_id = first_inv.id.clone();
                                let f_name = first_inv.name.clone();
                                rsx! {
                                    button {
                                        class: "btn-secondary btn-sm mt-2",
                                        onclick: move |_| {
                                            let mut items = form_consumed_items();
                                            items.push(ConsumedItemDto {
                                                item_id: f_id.clone(),
                                                item_name: Some(f_name.clone()),
                                                quantity_planned: 1,
                                                quantity_used: None,
                                            });
                                            form_consumed_items.set(items);
                                        },
                                        IconPlus { size: 12, color: "currentColor".to_string() }
                                        span { "Adicionar Item de Estoque" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| { let mut o = is_open; o.set(false); }, "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        IconCheck { size: 16, color: "currentColor".to_string() }
                        span { if is_submitting() { "Salvando..." } else { "Salvar Agendamento" } }
                    }
                }
            }
        }
    }
}
