//! # Aba de Tratamentos Odontológicos, Orçamentos e Evolução do Paciente
//!
//! Controla os planos de tratamento / orçamentos e o histórico de procedimentos clínicos
//! realizados no paciente, com agendamento direto na Agenda, aprovação financeira e
//! registro de evolução clínica.

use super::treatment_plan_modal::TreatmentPlanModal;
use crate::api::{
    create_appointment, create_patient_treatment, delete_patient_treatment, delete_treatment_plan,
    update_patient_treatment, update_treatment_plan_status,
};
use crate::components::icons::{
    IconAlertTriangle, IconCalendar, IconCheck, IconClock, IconEdit, IconFilter, IconPlus,
    IconSearch, IconTooth, IconTrash,
};
use dioxus::prelude::*;
use shared::appointments::{AppointmentType, CreateAppointmentRequest};
use shared::patients::{
    CreatePatientTreatmentRequest, PatientTreatment, UpdatePatientTreatmentRequest,
};
use shared::treatments::{
    PatientTreatmentPlan, TreatmentPlanItem, TreatmentPlanStatus,
    UpdateTreatmentPlanStatusRequest,
};

fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;

    let reals_str = reals.to_string();
    let mut formatted_reals = String::new();
    let len = reals_str.len();
    for (i, ch) in reals_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_reals.push('.');
        }
        formatted_reals.push(ch);
    }

    if is_negative {
        format!("- R$ {},{:02}", formatted_reals, centavos)
    } else {
        format!("R$ {},{:02}", formatted_reals, centavos)
    }
}

fn plan_status_badge(status: &TreatmentPlanStatus) -> (&'static str, &'static str) {
    match status {
        TreatmentPlanStatus::Approved => ("badge-completed", "Aprovado"),
        TreatmentPlanStatus::InProgress => ("badge-active", "Em Andamento"),
        TreatmentPlanStatus::Draft => ("badge-pending", "Rascunho / Pendente"),
        TreatmentPlanStatus::Completed => ("badge-completed", "Concluído"),
        TreatmentPlanStatus::Canceled => ("badge-danger", "Cancelado"),
    }
}

#[component]
pub fn PatientTreatmentsTab(
    patient_id: String,
    patient_name: Option<String>,
    clinic_id: String,
    token: String,
    treatments: Vec<PatientTreatment>,
    treatment_plans: Vec<PatientTreatmentPlan>,
    can_write: bool,
    can_delete: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut active_subview = use_signal(|| "plans".to_string()); // "plans" or "treatments"
    let mut status_filter = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);

    // Modal state for treatment plan (Budget)
    let mut is_plan_modal_open = use_signal(|| false);
    let mut editing_plan = use_signal(|| None::<PatientTreatmentPlan>);

    // Modal state for scheduling appointment directly from plan
    let mut is_schedule_modal_open = use_signal(|| false);
    let mut schedule_target_plan = use_signal(|| None::<PatientTreatmentPlan>);
    let mut schedule_date = use_signal(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let mut schedule_time = use_signal(|| "09:00".to_string());
    let mut schedule_duration = use_signal(|| 45i32);
    let mut schedule_title = use_signal(String::new);
    let mut schedule_notes = use_signal(String::new);
    let mut is_scheduling = use_signal(|| false);

    // Modal state for single clinical procedure record
    let mut is_proc_modal_open = use_signal(|| false);
    let mut editing_proc = use_signal(|| None::<PatientTreatment>);
    let mut proc_name = use_signal(String::new);
    let mut proc_category = use_signal(|| "Dentística".to_string());
    let mut proc_tooth = use_signal(String::new);
    let mut proc_price_str = use_signal(|| "0.00".to_string());
    let mut proc_notes = use_signal(String::new);
    let mut proc_status = use_signal(|| "completed".to_string());
    let mut is_proc_saving = use_signal(|| false);

    // Delete confirmation state
    let mut delete_plan_target = use_signal(|| None::<PatientTreatmentPlan>);
    let mut delete_proc_target = use_signal(|| None::<PatientTreatment>);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut is_delete_proc_modal_open = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // KPIs dos Orçamentos
    let total_plans = treatment_plans.len();
    let total_budgeted_cents: i64 = treatment_plans.iter().map(|p| p.total_price_cents).sum();
    let approved_cents: i64 = treatment_plans
        .iter()
        .filter(|p| {
            p.status == TreatmentPlanStatus::Approved
                || p.status == TreatmentPlanStatus::InProgress
                || p.status == TreatmentPlanStatus::Completed
        })
        .map(|p| p.total_price_cents)
        .sum();
    let draft_count = treatment_plans
        .iter()
        .filter(|p| p.status == TreatmentPlanStatus::Draft)
        .count();

    // KPIs dos Procedimentos Realizados
    let total_treatments = treatments.len();
    let total_performed_cents: i64 = treatments.iter().map(|t| t.cost_cents).sum();
    let completed_proc_count = treatments
        .iter()
        .filter(|t| t.status == "completed" || t.status == "done")
        .count();

    // Filtro de pesquisa e status dos orçamentos
    let q = search_query().trim().to_lowercase();
    let filtered_plans: Vec<PatientTreatmentPlan> = treatment_plans
        .iter()
        .filter(|p| {
            let filter = status_filter();
            let matches_status = match filter.as_str() {
                "approved" => p.status == TreatmentPlanStatus::Approved,
                "in_progress" => p.status == TreatmentPlanStatus::InProgress,
                "draft" => p.status == TreatmentPlanStatus::Draft,
                "completed" => p.status == TreatmentPlanStatus::Completed,
                "canceled" => p.status == TreatmentPlanStatus::Canceled,
                _ => true,
            };

            let matches_search = if q.is_empty() {
                true
            } else {
                p.title.to_lowercase().contains(&q)
                    || p.notes.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || p.items.iter().any(|i| {
                        i.procedure_name.to_lowercase().contains(&q)
                            || i.tooth_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                            || i.dental_region.as_deref().unwrap_or("").to_lowercase().contains(&q)
                            || i.category.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    })
            };

            matches_status && matches_search
        })
        .cloned()
        .collect();

    // Filtro dos procedimentos realizados
    let filtered_treatments: Vec<PatientTreatment> = treatments
        .iter()
        .filter(|t| {
            if q.is_empty() {
                true
            } else {
                t.procedure_name.to_lowercase().contains(&q)
                    || t.procedure_category.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.tooth_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || t.clinical_notes.as_deref().unwrap_or("").to_lowercase().contains(&q)
            }
        })
        .cloned()
        .collect();

    // Handler de exclusão de orçamento
    let tok_del = token.clone();
    let pid_del = patient_id.clone();
    let cid_del = clinic_id.clone();
    let on_reload_del = reload_patient_details.clone();

    let mut handle_confirm_delete_plan = move |_| {
        let Some(ref target) = *delete_plan_target.read() else {
            return;
        };
        let target_id = target.id.clone();
        let t = tok_del.clone();
        let pid = pid_del.clone();
        let cid = cid_del.clone();
        let on_r = on_reload_del.clone();
        let mut is_del = is_deleting;
        let mut is_open = is_delete_modal_open;
        let mut target_sig = delete_plan_target;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_del.set(true);
        spawn(async move {
            let res = delete_treatment_plan(&t, &cid, &pid, &target_id).await;
            is_del.set(false);
            is_open.set(false);
            target_sig.set(None);
            match res {
                Ok(_) => {
                    toast.set(Some("Orçamento excluído com sucesso!".into()));
                    on_r.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir orçamento: {}", e)));
                }
            }
        });
    };

    // Handler de exclusão de procedimento clínico
    let tok_del_p = token.clone();
    let pid_del_p = patient_id.clone();
    let cid_del_p = clinic_id.clone();
    let on_reload_del_p = reload_patient_details.clone();

    let mut handle_confirm_delete_proc = move |_| {
        let Some(ref target) = *delete_proc_target.read() else {
            return;
        };
        let target_id = target.id.clone();
        let t = tok_del_p.clone();
        let pid = pid_del_p.clone();
        let cid = cid_del_p.clone();
        let on_r = on_reload_del_p.clone();
        let mut is_del = is_deleting;
        let mut is_open = is_delete_proc_modal_open;
        let mut target_sig = delete_proc_target;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_del.set(true);
        spawn(async move {
            let res = delete_patient_treatment(&t, &cid, &pid, &target_id).await;
            is_del.set(false);
            is_open.set(false);
            target_sig.set(None);
            match res {
                Ok(_) => {
                    toast.set(Some("Procedimento clínico excluído do histórico!".into()));
                    on_r.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir procedimento: {}", e)));
                }
            }
        });
    };

    // Handler de agendamento na agenda
    let tok_sched = token.clone();
    let cid_sched = clinic_id.clone();
    let pid_sched = patient_id.clone();
    let p_name_sched = patient_name.clone().unwrap_or_else(|| "Paciente".into());

    let mut handle_submit_schedule = move |_| {
        let title_val = schedule_title().trim().to_string();
        if title_val.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o título do agendamento.".into()));
            return;
        }

        let date_val = schedule_date().trim().to_string();
        let time_val = schedule_time().trim().to_string();
        if date_val.is_empty() || time_val.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe a data e horário da consulta.".into()));
            return;
        }

        let scheduled_for = format!("{}T{}:00Z", date_val, time_val);

        let t = tok_sched.clone();
        let cid = cid_sched.clone();
        let pid = pid_sched.clone();
        let pname = p_name_sched.clone();
        let notes_val = schedule_notes().trim().to_string();
        let dur = schedule_duration();

        let req = CreateAppointmentRequest {
            clinic_id: cid,
            patient_id: Some(pid),
            patient_name: Some(pname.clone()),
            title: title_val,
            scheduled_for,
            duration_minutes: dur,
            appointment_type: AppointmentType::Treatment,
            financial_amount_cents: None,
            financial_type: None,
            notes: if notes_val.is_empty() { None } else { Some(notes_val) },
            assigned_users: vec![],
            consumed_items: vec![],
            assigned_equipment: None,
        };

        let mut is_sched = is_scheduling;
        let mut is_open = is_schedule_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_sched.set(true);
        spawn(async move {
            let res = create_appointment(&t, req).await;
            is_sched.set(false);
            match res {
                Ok(_) => {
                    is_open.set(false);
                    toast.set(Some(format!("🎉 Agendamento criado com sucesso na Agenda para {}!", pname)));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao agendar: {}", e)));
                }
            }
        });
    };

    // Handler de salvar procedimento clínico avulso
    let tok_proc = token.clone();
    let cid_proc = clinic_id.clone();
    let pid_proc = patient_id.clone();
    let on_reload_proc = reload_patient_details.clone();

    let mut handle_save_proc = move |_| {
        let name = proc_name().trim().to_string();
        if name.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome do procedimento realizado.".into()));
            return;
        }

        let clean_cost = proc_price_str()
            .trim()
            .replace(',', ".")
            .replace("R$", "")
            .replace(' ', "");
        let cost_cents = if let Ok(val) = clean_cost.parse::<f64>() {
            (val * 100.0).round() as i64
        } else {
            0
        };

        let t = tok_proc.clone();
        let cid = cid_proc.clone();
        let pid = pid_proc.clone();
        let category = proc_category();
        let tooth = proc_tooth().trim().to_string();
        let notes = proc_notes().trim().to_string();
        let status = proc_status();
        let existing = editing_proc();
        let on_r = on_reload_proc.clone();

        let mut is_saving = is_proc_saving;
        let mut is_open = is_proc_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_saving.set(true);
        spawn(async move {
            let res = if let Some(ref p) = existing {
                let req = UpdatePatientTreatmentRequest {
                    clinic_id: cid,
                    dentist_user_id: None,
                    appointment_id: None,
                    document_id: None,
                    exam_id: None,
                    treatment_plan_id: None,
                    transaction_id: None,
                    procedure_category: Some(category),
                    procedure_name: name,
                    tooth_number: if tooth.is_empty() { None } else { Some(tooth) },
                    surfaces: None,
                    materials_used: None,
                    status,
                    cost_cents,
                    post_care_instructions: None,
                    clinical_notes: if notes.is_empty() { None } else { Some(notes) },
                    performed_at: Some(chrono::Utc::now().to_rfc3339()),
                };
                update_patient_treatment(&t, &pid, &p.id, req).await.map(|_| ())
            } else {
                let req = CreatePatientTreatmentRequest {
                    clinic_id: cid,
                    dentist_user_id: None,
                    appointment_id: None,
                    document_id: None,
                    exam_id: None,
                    treatment_plan_id: None,
                    transaction_id: None,
                    procedure_category: Some(category),
                    procedure_name: name,
                    tooth_number: if tooth.is_empty() { None } else { Some(tooth) },
                    surfaces: None,
                    materials_used: None,
                    status,
                    cost_cents,
                    post_care_instructions: None,
                    clinical_notes: if notes.is_empty() { None } else { Some(notes) },
                    performed_at: Some(chrono::Utc::now().to_rfc3339()),
                };
                create_patient_treatment(&t, &pid, req).await.map(|_| ())
            };

            is_saving.set(false);
            match res {
                Ok(_) => {
                    is_open.set(false);
                    toast.set(Some("Procedimento clínico registrado no prontuário!".into()));
                    on_r.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao salvar procedimento: {}", e)));
                }
            }
        });
    };

    let tok_app = token.clone();
    let pid_app = patient_id.clone();
    let cid_app = clinic_id.clone();
    let on_reload_app = reload_patient_details.clone();

    rsx! {
        div { class: "patient-treatments-container",
            // 1. Abas Limpas de Navegação no Prontuário
            div { class: "documents-tab-bar",
                button {
                    class: if active_subview() == "plans" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_subview.set("plans".to_string()),
                    IconTooth { size: 16, color: "currentColor".to_string() }
                    span { " Orçamentos & Planos de Tratamento ({total_plans})" }
                }
                button {
                    class: if active_subview() == "treatments" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_subview.set("treatments".to_string()),
                    IconCheck { size: 16, color: "currentColor".to_string() }
                    span { " Procedimentos Clínicos Realizados ({total_treatments})" }
                }
            }

            // 2. CONTEÚDO: ORÇAMENTOS & PLANOS
            if active_subview() == "plans" {
                // KPIs dos Orçamentos
                div { class: "agenda-kpi-row",
                    div { class: "agenda-kpi-card",
                        div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                            IconTooth { size: 16, color: "currentColor".to_string() }
                        }
                        div { class: "agenda-kpi-text-col",
                            span { class: "agenda-kpi-lbl", "Total Orçado" }
                            span { class: "agenda-kpi-sublbl", "{total_plans} orçamentos emitidos" }
                        }
                        div { class: "agenda-kpi-val font-mono", "{format_currency(total_budgeted_cents)}" }
                    }

                    div { class: "agenda-kpi-card",
                        div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                            IconCheck { size: 16, color: "currentColor".to_string() }
                        }
                        div { class: "agenda-kpi-text-col",
                            span { class: "agenda-kpi-lbl", "Aprovado / Em Execução" }
                            span { class: "agenda-kpi-sublbl", "Tratamentos ativos" }
                        }
                        div { class: "agenda-kpi-val kpi-completed font-mono", "{format_currency(approved_cents)}" }
                    }

                    div { class: "agenda-kpi-card",
                        div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                            IconClock { size: 16, color: "currentColor".to_string() }
                        }
                        div { class: "agenda-kpi-text-col",
                            span { class: "agenda-kpi-lbl", "Aguardando Aprovação" }
                            span { class: "agenda-kpi-sublbl", "{draft_count} rascunhos pendentes" }
                        }
                        div { class: "agenda-kpi-val kpi-pending", "{draft_count}" }
                    }
                }

                // Toolbar dos Orçamentos
                div { class: "view-toolbar",
                    div { class: "search-input-wrap",
                        IconSearch { size: 18, color: "#94a3b8".to_string() }
                        input {
                            r#type: "text",
                            class: "search-input",
                            placeholder: "Buscar por orçamento, dente ou procedimento...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }

                    div { class: "toolbar-filter-select-wrap",
                        IconFilter { size: 16, color: "#64748b".to_string() }
                        select {
                            class: "toolbar-specialty-select",
                            value: "{status_filter}",
                            onchange: move |e: FormEvent| status_filter.set(e.value()),
                            option { value: "all", "Todos os Status ({total_plans})" }
                            option { value: "approved", "Aprovados" }
                            option { value: "draft", "Rascunhos / Pendentes" }
                            option { value: "in_progress", "Em Andamento" }
                            option { value: "completed", "Concluídos" }
                            option { value: "canceled", "Cancelados" }
                        }
                    }

                    if can_write {
                        div { class: "toolbar-actions",
                            button {
                                class: "btn-primary",
                                onclick: move |_| {
                                    editing_plan.set(None);
                                    is_plan_modal_open.set(true);
                                },
                                IconPlus { size: 16, color: "currentColor".to_string() }
                                span { " Novo Orçamento" }
                            }
                        }
                    }
                }

                // Lista de Orçamentos
                if filtered_plans.is_empty() {
                    div { class: "empty-state-card",
                        IconTooth { size: 44, color: "#94a3b8".to_string() }
                        h3 { "Nenhum orçamento encontrado" }
                        p { "Crie um plano de tratamento selecionando procedimentos do catálogo ou digitando itens sob medida." }
                        if can_write {
                            button {
                                class: "btn-primary",
                                style: "margin-top: 14px;",
                                onclick: move |_| {
                                    editing_plan.set(None);
                                    is_plan_modal_open.set(true);
                                },
                                IconPlus { size: 16, color: "currentColor".to_string() }
                                span { " Criar Primeiro Orçamento" }
                            }
                        }
                    }
                } else {
                    div { class: "patient-plans-list-container",
                        for plan in filtered_plans {
                            {
                                let p_edit = plan.clone();
                                let p_del = plan.clone();
                                let p_approve = plan.clone();
                                let p_sched = plan.clone();
                                let (badge_cls, badge_text) = plan_status_badge(&plan.status);
                                let is_draft = plan.status == TreatmentPlanStatus::Draft;

                                let tok_app_inner = tok_app.clone();
                                let pid_app_inner = pid_app.clone();
                                let cid_app_inner = cid_app.clone();
                                let on_r_app_inner = on_reload_app.clone();

                                rsx! {
                                    div { key: "{plan.id}", class: "patient-plan-card-modern",
                                        // Header do Card
                                        div { class: "plan-card-header",
                                            div { class: "plan-card-header-left",
                                                span { class: "status-badge {badge_cls}", "{badge_text}" }
                                                h3 { class: "plan-title-text", "{plan.title}" }
                                                if let Some(ref s_date) = plan.planned_start_date {
                                                    span { class: "plan-date-tag",
                                                        IconCalendar { size: 13, color: "#64748b".to_string() }
                                                        "Início: {s_date}"
                                                    }
                                                }
                                            }

                                            // Ações Rápidas
                                            div { class: "plan-card-header-actions",
                                                // 1. Agendar na Agenda Direto pelo Prontuário
                                                button {
                                                    r#type: "button",
                                                    class: "btn-action-outline btn-schedule-action",
                                                    title: "Agendar procedimento deste orçamento na Agenda",
                                                    onclick: move |_| {
                                                        let pl = p_sched.clone();
                                                        schedule_target_plan.set(Some(pl.clone()));
                                                        schedule_title.set(pl.title.clone());
                                                        if let Some(ref dt) = pl.planned_start_date {
                                                            schedule_date.set(dt.clone());
                                                        }
                                                        schedule_notes.set(pl.notes.clone().unwrap_or_default());
                                                        is_schedule_modal_open.set(true);
                                                    },
                                                    IconCalendar { size: 14, color: "#0284c7".to_string() }
                                                    span { " Agendar" }
                                                }

                                                // 2. Aprovar Orçamento (com 1 clique -> gera financeiro)
                                                if is_draft && can_write {
                                                    button {
                                                        r#type: "button",
                                                        class: "btn-action-outline btn-approve-action",
                                                        title: "Aprovar orçamento e emitir lançamento no financeiro",
                                                        onclick: move |_| {
                                                            let t = tok_app_inner.clone();
                                                            let pid = pid_app_inner.clone();
                                                            let cid = cid_app_inner.clone();
                                                            let pl_id = p_approve.id.clone();
                                                            let on_r = on_r_app_inner.clone();
                                                            let mut toast = toast_msg;
                                                            let mut err_sig = error_toast;

                                                            spawn(async move {
                                                                let req = UpdateTreatmentPlanStatusRequest {
                                                                    clinic_id: cid,
                                                                    status: TreatmentPlanStatus::Approved,
                                                                };
                                                                match update_treatment_plan_status(&t, &pid, &pl_id, req).await {
                                                                    Ok(_) => {
                                                                        toast.set(Some("Orçamento aprovado e lançado no Financeiro!".into()));
                                                                        on_r.call(());
                                                                    }
                                                                    Err(e) => {
                                                                        err_sig.set(Some(format!("Erro ao aprovar orçamento: {}", e)));
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        IconCheck { size: 14, color: "#10b981".to_string() }
                                                        span { " Aprovar" }
                                                    }
                                                }

                                                // 3. Editar Orçamento
                                                if can_write {
                                                    button {
                                                        r#type: "button",
                                                        class: "stock-action-icon-btn",
                                                        title: "Editar Orçamento",
                                                        onclick: move |_| {
                                                            editing_plan.set(Some(p_edit.clone()));
                                                            is_plan_modal_open.set(true);
                                                        },
                                                        IconEdit { size: 15, color: "currentColor".to_string() }
                                                    }
                                                }

                                                // 4. Excluir Orçamento
                                                if can_delete {
                                                    button {
                                                        r#type: "button",
                                                        class: "stock-action-icon-btn btn-danger-icon",
                                                        title: "Excluir Orçamento",
                                                        onclick: move |_| {
                                                            delete_plan_target.set(Some(p_del.clone()));
                                                            is_delete_modal_open.set(true);
                                                        },
                                                        IconTrash { size: 15, color: "currentColor".to_string() }
                                                    }
                                                }
                                            }
                                        }

                                        // Corpo do Card
                                        div { class: "plan-card-body",
                                            div { class: "plan-items-table-clean",
                                                for (idx, item) in plan.items.iter().enumerate() {
                                                    {
                                                        rsx! {
                                                            div { key: "{idx}_{item.procedure_name}", class: "plan-item-row-clean",
                                                                div { class: "plan-item-left",
                                                                    if let Some(ref tooth) = item.tooth_number {
                                                                        if !tooth.is_empty() {
                                                                            span { class: "item-tooth-pill", "🦷 {tooth}" }
                                                                        }
                                                                    }
                                                                    span { class: "item-name-bold", "{item.procedure_name}" }
                                                                    if let Some(ref cat) = item.category {
                                                                        span { class: "item-category-tag", "{cat}" }
                                                                    }
                                                                }

                                                                div { class: "plan-item-right",
                                                                    if let Some(ref notes) = item.clinical_notes {
                                                                        if !notes.is_empty() {
                                                                            span { class: "item-notes-text", "{notes}" }
                                                                        }
                                                                    }
                                                                    span { class: "item-price-text font-mono", "{format_currency(item.price_cents)}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if let Some(ref notes) = plan.notes {
                                                if !notes.trim().is_empty() {
                                                    div { class: "plan-notes-banner",
                                                        strong { "📝 Observações / Condições: " }
                                                        span { "{notes}" }
                                                    }
                                                }
                                            }
                                        }

                                        // Rodapé do Card
                                        div { class: "plan-card-footer",
                                            div { class: "plan-footer-count",
                                                "{plan.items.len()} procedimento(s) planejado(s)"
                                            }
                                            div { class: "plan-footer-total-wrap",
                                                span { class: "total-label", "Total do Orçamento:" }
                                                span { class: "total-val font-mono", "{format_currency(plan.total_price_cents)}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // 3. CONTEÚDO: PROCEDIMENTOS CLÍNICOS REALIZADOS (HISTÓRICO)
                div { class: "agenda-kpi-row",
                    div { class: "agenda-kpi-card",
                        div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                            IconCheck { size: 16, color: "currentColor".to_string() }
                        }
                        div { class: "agenda-kpi-text-col",
                            span { class: "agenda-kpi-lbl", "Procedimentos Realizados" }
                            span { class: "agenda-kpi-sublbl", "{completed_proc_count} executados" }
                        }
                        div { class: "agenda-kpi-val kpi-completed", "{total_treatments}" }
                    }

                    div { class: "agenda-kpi-card",
                        div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                            IconTooth { size: 16, color: "currentColor".to_string() }
                        }
                        div { class: "agenda-kpi-text-col",
                            span { class: "agenda-kpi-lbl", "Total em Procedimentos" }
                            span { class: "agenda-kpi-sublbl", "Histórico acumulado" }
                        }
                        div { class: "agenda-kpi-val font-mono", "{format_currency(total_performed_cents)}" }
                    }
                }

                // Toolbar dos Procedimentos Realizados
                div { class: "view-toolbar",
                    div { class: "search-input-wrap",
                        IconSearch { size: 18, color: "#94a3b8".to_string() }
                        input {
                            r#type: "text",
                            class: "search-input",
                            placeholder: "Buscar no histórico de procedimentos...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }

                    if can_write {
                        div { class: "toolbar-actions",
                            button {
                                class: "btn-primary",
                                onclick: move |_| {
                                    editing_proc.set(None);
                                    proc_name.set(String::new());
                                    proc_category.set("Dentística".to_string());
                                    proc_tooth.set(String::new());
                                    proc_price_str.set("0.00".to_string());
                                    proc_notes.set(String::new());
                                    is_proc_modal_open.set(true);
                                },
                                IconPlus { size: 16, color: "currentColor".to_string() }
                                span { " Registrar Procedimento" }
                            }
                        }
                    }
                }

                // Lista de Procedimentos Realizados
                if filtered_treatments.is_empty() {
                    div { class: "empty-state-card",
                        IconCheck { size: 44, color: "#94a3b8".to_string() }
                        h3 { "Nenhum procedimento registrado ainda" }
                        p { "Registre procedimentos executados, restaurações, cirurgias e evoluções clínicas do paciente." }
                        if can_write {
                            button {
                                class: "btn-primary",
                                style: "margin-top: 14px;",
                                onclick: move |_| {
                                    editing_proc.set(None);
                                    proc_name.set(String::new());
                                    proc_category.set("Dentística".to_string());
                                    proc_tooth.set(String::new());
                                    proc_price_str.set("0.00".to_string());
                                    proc_notes.set(String::new());
                                    is_proc_modal_open.set(true);
                                },
                                IconPlus { size: 16, color: "currentColor".to_string() }
                                span { " Registrar Primeiro Procedimento" }
                            }
                        }
                    }
                } else {
                    div { class: "table-container",
                        table { class: "modern-table",
                            thead {
                                tr {
                                    th { "Data" }
                                    th { "Procedimento" }
                                    th { "Especialidade" }
                                    th { "Dente / Região" }
                                    th { "Valor (R$)" }
                                    th { "Status" }
                                    th { "Notas Clínicas" }
                                    th { class: "text-right", "Ações" }
                                }
                            }
                            tbody {
                                for t_item in filtered_treatments {
                                    {
                                        let t_edit = t_item.clone();
                                        let t_del = t_item.clone();
                                        let date_display = t_item.performed_at.as_deref().unwrap_or(&t_item.created_at).chars().take(10).collect::<String>();
                                        rsx! {
                                            tr { key: "{t_item.id}",
                                                td { class: "font-mono text-sm", "{date_display}" }
                                                td { strong { "{t_item.procedure_name}" } }
                                                td { span { class: "badge-outline", "{t_item.procedure_category.as_deref().unwrap_or(\"Geral\")}" } }
                                                td {
                                                    if let Some(ref tooth) = t_item.tooth_number {
                                                        span { class: "item-tooth-pill", "🦷 {tooth}" }
                                                    } else {
                                                        span { class: "text-muted", "-" }
                                                    }
                                                }
                                                td { class: "font-mono font-bold", "{format_currency(t_item.cost_cents)}" }
                                                td {
                                                    if t_item.status == "completed" || t_item.status == "done" {
                                                        span { class: "badge-completed", "✓ Concluído" }
                                                    } else {
                                                        span { class: "badge-active", "{t_item.status}" }
                                                    }
                                                }
                                                td { class: "text-sm text-muted", "{t_item.clinical_notes.as_deref().unwrap_or(\"-\")}" }
                                                td { class: "text-right",
                                                    div { class: "table-actions-row",
                                                        if can_write {
                                                            button {
                                                                class: "btn-action-icon",
                                                                title: "Editar Procedimento",
                                                                onclick: move |_| {
                                                                    editing_proc.set(Some(t_edit.clone()));
                                                                    proc_name.set(t_edit.procedure_name.clone());
                                                                    proc_category.set(t_edit.procedure_category.clone().unwrap_or_else(|| "Dentística".into()));
                                                                    proc_tooth.set(t_edit.tooth_number.clone().unwrap_or_default());
                                                                    proc_price_str.set(format!("{:.2}", t_edit.cost_cents as f64 / 100.0));
                                                                    proc_notes.set(t_edit.clinical_notes.clone().unwrap_or_default());
                                                                    is_proc_modal_open.set(true);
                                                                },
                                                                IconEdit { size: 15, color: "#475569".to_string() }
                                                            }
                                                        }
                                                        if can_delete {
                                                            button {
                                                                class: "btn-action-icon btn-danger-icon",
                                                                title: "Excluir Procedimento",
                                                                onclick: move |_| {
                                                                    delete_proc_target.set(Some(t_del.clone()));
                                                                    is_delete_proc_modal_open.set(true);
                                                                },
                                                                IconTrash { size: 15, color: "#dc2626".to_string() }
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

            // Modal de Elaboração / Edição de Orçamento
            if is_plan_modal_open() {
                TreatmentPlanModal {
                    patient_id: patient_id.clone(),
                    clinic_id: clinic_id.clone(),
                    token: token.clone(),
                    editing_plan: editing_plan(),
                    on_close: move |_| is_plan_modal_open.set(false),
                    on_saved: move |_| {
                        is_plan_modal_open.set(false);
                        reload_patient_details.call(());
                    },
                    toast_msg,
                    error_toast,
                }
            }

            // Modal de Agendamento Direto pelo Orçamento
            if is_schedule_modal_open() {
                div { class: "modal-overlay",
                    onclick: move |_| is_schedule_modal_open.set(false),
                    div { class: "modal-card action-modal",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            div { class: "modal-header-left",
                                div { class: "stock-header-icon-box",
                                    IconCalendar { size: 20, color: "#0284c7".to_string() }
                                }
                                div { class: "header-text-col",
                                    h2 { class: "modal-title", "Agendar Procedimento na Agenda" }
                                    p { class: "modal-subtitle", "Defina a data e horário para a realização deste plano de tratamento." }
                                }
                            }
                            button { class: "modal-close-btn", onclick: move |_| is_schedule_modal_open.set(false), "✕" }
                        }

                        div { class: "modal-body",
                            div { class: "input-group-wrapper",
                                label { "Paciente" }
                                input {
                                    class: "modern-input-field",
                                    disabled: true,
                                    value: "{patient_name.as_deref().unwrap_or(\"Paciente\")}",
                                }
                            }

                            div { class: "input-group-wrapper",
                                label { "Procedimento / Título do Agendamento *" }
                                input {
                                    class: "modern-input-field",
                                    value: "{schedule_title}",
                                    oninput: move |e| schedule_title.set(e.value()),
                                }
                            }

                            div { class: "form-row-grid-3",
                                div { class: "input-group-wrapper",
                                    label { "Data da Consulta *" }
                                    input {
                                        r#type: "date",
                                        class: "modern-input-field font-mono",
                                        value: "{schedule_date}",
                                        oninput: move |e| schedule_date.set(e.value()),
                                    }
                                }
                                div { class: "input-group-wrapper",
                                    label { "Horário *" }
                                    input {
                                        r#type: "time",
                                        class: "modern-input-field font-mono",
                                        value: "{schedule_time}",
                                        oninput: move |e| schedule_time.set(e.value()),
                                    }
                                }
                                div { class: "input-group-wrapper",
                                    label { "Duração (minutos)" }
                                    input {
                                        r#type: "number",
                                        class: "modern-input-field font-mono",
                                        value: "{schedule_duration}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<i32>() {
                                                schedule_duration.set(v);
                                            }
                                        },
                                    }
                                }
                            }

                            div { class: "input-group-wrapper",
                                label { "Instruções / Notas para a Consulta" }
                                textarea {
                                    class: "modern-input-field",
                                    rows: "2",
                                    placeholder: "Ex: Separar kit cirúrgico e anestésicos...",
                                    value: "{schedule_notes}",
                                    oninput: move |e| schedule_notes.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: move |_| is_schedule_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary",
                                disabled: is_scheduling(),
                                onclick: handle_submit_schedule,
                                IconCalendar { size: 16, color: "#ffffff".to_string() }
                                span { if is_scheduling() { "Agendando..." } else { "Confirmar Agendamento" } }
                            }
                        }
                    }
                }
            }

            // Modal de Registro / Edição de Procedimento Clínico Realizado
            if is_proc_modal_open() {
                div { class: "modal-overlay",
                    onclick: move |_| is_proc_modal_open.set(false),
                    div { class: "modal-card action-modal",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            div { class: "modal-header-left",
                                div { class: "stock-header-icon-box",
                                    IconTooth { size: 20, color: "#0284c7".to_string() }
                                }
                                div { class: "header-text-col",
                                    h2 { class: "modal-title",
                                        if editing_proc().is_some() { "Editar Procedimento Clínico" } else { "Registrar Procedimento Clínico" }
                                    }
                                    p { class: "modal-subtitle", "Histórico de evolução e procedimentos odontológicos executados." }
                                }
                            }
                            button { class: "modal-close-btn", onclick: move |_| is_proc_modal_open.set(false), "✕" }
                        }

                        div { class: "modal-body",
                            div { class: "form-row-grid-2",
                                div { class: "input-group-wrapper",
                                    label { "Nome do Procedimento *" }
                                    input {
                                        class: "modern-input-field",
                                        placeholder: "Ex: Restauração Resina, Profilaxia...",
                                        value: "{proc_name}",
                                        oninput: move |e| proc_name.set(e.value()),
                                    }
                                }
                                div { class: "input-group-wrapper",
                                    label { "Especialidade" }
                                    select {
                                        class: "modern-input-field modern-select",
                                        value: "{proc_category}",
                                        onchange: move |e: FormEvent| proc_category.set(e.value()),
                                        option { value: "Dentística", "Dentística" }
                                        option { value: "Cirurgia", "Cirurgia" }
                                        option { value: "Endodontia", "Endodontia" }
                                        option { value: "Ortodontia", "Ortodontia" }
                                        option { value: "Periodontia", "Periodontia" }
                                        option { value: "Prótese", "Prótese" }
                                        option { value: "Estética", "Estética" }
                                        option { value: "Implantodontia", "Implantodontia" }
                                        option { value: "Odontopediatria", "Odontopediatria" }
                                        option { value: "Geral", "Geral" }
                                    }
                                }
                            }

                            div { class: "form-row-grid-3",
                                div { class: "input-group-wrapper",
                                    label { "Dente / Região" }
                                    input {
                                        class: "modern-input-field",
                                        placeholder: "Ex: 21, 38 ou Arcada",
                                        value: "{proc_tooth}",
                                        oninput: move |e| proc_tooth.set(e.value()),
                                    }
                                }
                                div { class: "input-group-wrapper",
                                    label { "Valor Cobrado (R$)" }
                                    input {
                                        class: "modern-input-field font-mono",
                                        placeholder: "0.00",
                                        value: "{proc_price_str}",
                                        oninput: move |e| proc_price_str.set(e.value()),
                                    }
                                }
                                div { class: "input-group-wrapper",
                                    label { "Status da Execução" }
                                    select {
                                        class: "modern-input-field modern-select",
                                        value: "{proc_status}",
                                        onchange: move |e: FormEvent| proc_status.set(e.value()),
                                        option { value: "completed", "Concluído" }
                                        option { value: "in_progress", "Em Andamento" }
                                    }
                                }
                            }

                            div { class: "input-group-wrapper",
                                label { "Notas Clínicas / Evolução" }
                                textarea {
                                    class: "modern-input-field",
                                    rows: "2",
                                    placeholder: "Detalhes da técnica, materiais empregados e observações clínicas...",
                                    value: "{proc_notes}",
                                    oninput: move |e| proc_notes.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: move |_| is_proc_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary",
                                disabled: is_proc_saving(),
                                onclick: handle_save_proc,
                                IconCheck { size: 16, color: "#ffffff".to_string() }
                                span { if is_proc_saving() { "Salvando..." } else { "Salvar Procedimento" } }
                            }
                        }
                    }
                }
            }

            // Modal de Confirmação de Exclusão de Orçamento
            if is_delete_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal modal-delete-confirm",
                        div { class: "settings-header",
                            div { class: "modal-header-left",
                                div { class: "stock-header-icon-box header-icon-danger",
                                    IconTrash { size: 20, color: "#dc2626".to_string() }
                                }
                                div {
                                    h2 { class: "settings-title", "Excluir Orçamento" }
                                    p { class: "settings-subtitle", "Esta ação removerá este orçamento do prontuário do paciente." }
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_delete_modal_open.set(false), "×" }
                        }

                        div { class: "settings-content",
                            div { class: "delete-confirm-box",
                                if let Some(ref target) = *delete_plan_target.read() {
                                    p {
                                        "Você está prestes a excluir o orçamento "
                                        strong { "\"{target.title}\"" }
                                        " no valor de "
                                        strong { "{format_currency(target.total_price_cents)}" }
                                        "."
                                    }
                                }
                            }
                        }

                        div { class: "modal-actions",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: move |_| is_delete_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "button",
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: handle_confirm_delete_plan,
                                if is_deleting() { "Excluindo..." } else { "Sim, Excluir" }
                            }
                        }
                    }
                }
            }

            // Modal de Confirmação de Exclusão de Procedimento Realizado
            if is_delete_proc_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal modal-delete-confirm",
                        div { class: "settings-header",
                            div { class: "modal-header-left",
                                div { class: "stock-header-icon-box header-icon-danger",
                                    IconTrash { size: 20, color: "#dc2626".to_string() }
                                }
                                div {
                                    h2 { class: "settings-title", "Excluir Procedimento" }
                                    p { class: "settings-subtitle", "Esta ação removerá o procedimento do histórico do paciente." }
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_delete_proc_modal_open.set(false), "×" }
                        }

                        div { class: "settings-content",
                            div { class: "delete-confirm-box",
                                if let Some(ref target) = *delete_proc_target.read() {
                                    p {
                                        "Você está prestes a excluir o procedimento "
                                        strong { "\"{target.procedure_name}\"" }
                                        "."
                                    }
                                }
                            }
                        }

                        div { class: "modal-actions",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: move |_| is_delete_proc_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "button",
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: handle_confirm_delete_proc,
                                if is_deleting() { "Excluindo..." } else { "Sim, Excluir" }
                            }
                        }
                    }
                }
            }
        }
    }
}
