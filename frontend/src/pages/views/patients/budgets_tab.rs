//! # Aba de Orçamentos e Planos de Tratamento do Paciente (Frontend)
//!
//! Gerencia os orçamentos (Rascunho e Aprovado), controle de pendências financeiras,
//! amortização e pagamentos parciais com método obrigatório, e conversão automática
//! de itens em procedimentos clínicos no Prontuário.

use super::treatment_plan_modal::TreatmentPlanModal;
use crate::api::{delete_treatment_plan, pay_treatment_plan, update_treatment_plan_status};
use crate::components::icons::{
    IconCheck, IconClock, IconEdit, IconPlus, IconSearch, IconTooth, IconTrash,
};
use dioxus::prelude::*;
use shared::finance::RegisterPaymentRequest;
use shared::treatments::{
    PatientTreatmentPlan, TreatmentPlanStatus, UpdateTreatmentPlanStatusRequest,
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

#[component]
pub fn PatientBudgetsTab(
    patient_id: String,
    patient_name: Option<String>,
    clinic_id: String,
    token: String,
    treatment_plans: Vec<PatientTreatmentPlan>,
    can_write: bool,
    can_delete: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut status_filter = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);

    // Modal state for treatment plan (Budget)
    let mut is_plan_modal_open = use_signal(|| false);
    let mut editing_plan = use_signal(|| None::<PatientTreatmentPlan>);

    // Modal state for registering payment
    let mut is_pay_modal_open = use_signal(|| false);
    let mut pay_target_plan = use_signal(|| None::<PatientTreatmentPlan>);
    let mut pay_amount_input = use_signal(String::new);
    let mut pay_method = use_signal(|| "Pix".to_string());
    let mut pay_notes = use_signal(String::new);
    let mut is_paying = use_signal(|| false);

    // Delete confirmation state
    let mut delete_plan_target = use_signal(|| None::<PatientTreatmentPlan>);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Approving state
    let mut is_approving = use_signal(|| false);

    // KPIs dos Orçamentos
    let total_plans = treatment_plans.len();
    let total_budgeted_cents: i64 = treatment_plans.iter().map(|p| p.total_price_cents).sum();
    let total_paid_cents: i64 = treatment_plans.iter().map(|p| p.paid_amount_cents).sum();
    let total_pending_cents: i64 = (total_budgeted_cents - total_paid_cents).max(0);

    let draft_count = treatment_plans
        .iter()
        .filter(|p| p.status == TreatmentPlanStatus::Draft)
        .count();

    // Filtro de pesquisa e status dos orçamentos
    let q = search_query().trim().to_lowercase();
    let filtered_plans: Vec<PatientTreatmentPlan> = treatment_plans
        .iter()
        .filter(|p| {
            let filter = status_filter();
            let matches_status = match filter.as_str() {
                "approved" => {
                    p.status == TreatmentPlanStatus::Approved
                        || p.status == TreatmentPlanStatus::InProgress
                        || p.status == TreatmentPlanStatus::Completed
                }
                "draft" => p.status == TreatmentPlanStatus::Draft,
                "paid" => p.paid_amount_cents >= p.total_price_cents && p.total_price_cents > 0,
                "unpaid" => p.paid_amount_cents < p.total_price_cents,
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

    // Handler de exclusão de orçamento
    let tok_del = token.clone();
    let pid_del = patient_id.clone();
    let cid_del = clinic_id.clone();
    let on_reload_del = reload_patient_details.clone();

    let mut handle_confirm_delete_plan = move |_| {
        let Some(target) = delete_plan_target() else {
            return;
        };
        let target_id = target.id.clone();
        let t = tok_del.clone();
        let pid = pid_del.clone();
        let cid = cid_del.clone();

        is_deleting.set(true);
        spawn(async move {
            match delete_treatment_plan(&t, &cid, &pid, &target_id).await {
                Ok(_) => {
                    toast_msg.set(Some("Orçamento excluído com sucesso.".into()));
                    is_delete_modal_open.set(false);
                    delete_plan_target.set(None);
                    on_reload_del.call(());
                }
                Err(e) => {
                    error_toast.set(Some(format!("Falha ao excluir orçamento: {}", e)));
                }
            }
            is_deleting.set(false);
        });
    };

    // Handler para registrar pagamento com normalização monetária precisa
    let tok_pay = token.clone();
    let pid_pay = patient_id.clone();
    let cid_pay = clinic_id.clone();
    let on_reload_pay = reload_patient_details.clone();

    let mut handle_confirm_payment = move |_| {
        let Some(target) = pay_target_plan() else {
            return;
        };
        let method = pay_method().trim().to_string();
        if method.is_empty() {
            error_toast.set(Some("Escolha um método de pagamento obrigatório.".into()));
            return;
        }

        let raw_input = pay_amount_input().replace("R$", "").replace(' ', "").trim().to_string();
        let normalized = if raw_input.contains(',') && raw_input.contains('.') {
            raw_input.replace('.', "").replace(',', ".")
        } else if raw_input.contains(',') {
            raw_input.replace(',', ".")
        } else {
            raw_input
        };

        let amount_float: f64 = match normalized.parse() {
            Ok(v) if v > 0.0 => v,
            _ => {
                error_toast.set(Some("Informe um valor de pagamento válido maior que zero.".into()));
                return;
            }
        };

        let amount_cents = (amount_float * 100.0).round() as i64;
        let t = tok_pay.clone();
        let pid = pid_pay.clone();
        let cid = cid_pay.clone();
        let plan_id = target.id.clone();
        let notes_val = if pay_notes().trim().is_empty() {
            None
        } else {
            Some(pay_notes().trim().to_string())
        };

        is_paying.set(true);
        spawn(async move {
            let req = RegisterPaymentRequest {
                clinic_id: cid,
                amount_cents,
                payment_method: method,
                paid_date: Some(chrono::Utc::now().to_rfc3339()),
                notes: notes_val,
            };

            match pay_treatment_plan(&t, &pid, &plan_id, req).await {
                Ok(_) => {
                    toast_msg.set(Some(format!(
                        "Pagamento de {} registrado com sucesso no orçamento!",
                        format_currency(amount_cents)
                    )));
                    is_pay_modal_open.set(false);
                    pay_target_plan.set(None);
                    on_reload_pay.call(());
                }
                Err(e) => {
                    error_toast.set(Some(format!("Falha ao registrar pagamento: {}", e)));
                }
            }
            is_paying.set(false);
        });
    };

    rsx! {
        div { class: "patient-subtab-container",
            // 1. Cabeçalho de Ações da Aba (Título à esquerda, Botão à direita)
            div { class: "tab-header-actions-row",
                div {
                    h3 { class: "tab-title-text", "Orçamentos & Planos de Tratamento" }
                    p { class: "tab-subtitle-text",
                        "Propostas comerciais com aprovação rápida, geração automática de pendência financeira e parcelamento."
                    }
                }
                if can_write {
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

            // 2. Compact Horizontal KPIs (Padrão Unificado do ToothPlus)
            div { class: "patient-subtab-kpis",
                // 1. TOTAL ORÇADO
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconTooth { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Total Orçado" }
                        span { class: "agenda-kpi-sublbl", "{total_plans} propostas ({draft_count} rascunhos)" }
                    }
                    div { class: "agenda-kpi-val", "{format_currency(total_budgeted_cents)}" }
                }

                // 2. TOTAL PAGO / RECEBIDO
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconCheck { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Total Pago / Recebido" }
                        span { class: "agenda-kpi-sublbl", "Liquidado no fluxo de caixa" }
                    }
                    div { class: "agenda-kpi-val kpi-completed", "{format_currency(total_paid_cents)}" }
                }

                // 3. SALDO PENDENTE (A RECEBER)
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconClock { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Saldo Pendente (A Receber)" }
                        span { class: "agenda-kpi-sublbl", "Aguardando liquidação" }
                    }
                    div { class: "agenda-kpi-val kpi-pending", "{format_currency(total_pending_cents)}" }
                }
            }

            // 3. Toolbar de Ações & Filtros
            div { class: "patient-subtab-toolbar",
                div { class: "search-input-wrap", style: "max-width: 340px;",
                    IconSearch { size: 16, color: "#94a3b8".to_string() }
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Buscar por título, dente ou procedimento...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                div { class: "flex items-center gap-2",
                    select {
                        class: "select-field font-xs",
                        style: "width: auto; height: 38px; padding: 0 28px 0 12px;",
                        value: "{status_filter}",
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "all", "Todos os Status" }
                        option { value: "approved", "Aprovados" }
                        option { value: "draft", "Rascunhos" }
                        option { value: "paid", "100% Pagos" }
                        option { value: "unpaid", "Com Saldo Pendente" }
                    }
                }
            }

            // 4. Lista de Orçamentos
            if filtered_plans.is_empty() {
                div { class: "empty-state-card mt-4",
                    IconTooth { size: 42, color: "#cbd5e1".to_string() }
                    h4 { class: "empty-state-title", "Nenhum orçamento encontrado" }
                    p { class: "empty-state-desc",
                        "Elabore uma nova proposta de tratamento com os procedimentos, dentes e valores planejados."
                    }
                    if can_write {
                        button {
                            class: "btn-primary mt-3",
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
                div { class: "budget-plans-list",
                    for plan in filtered_plans {
                        {
                            let is_approved = plan.status == TreatmentPlanStatus::Approved
                                || plan.status == TreatmentPlanStatus::InProgress
                                || plan.status == TreatmentPlanStatus::Completed;

                            let is_fully_paid = plan.paid_amount_cents >= plan.total_price_cents && plan.total_price_cents > 0;
                            let is_partial = plan.paid_amount_cents > 0 && !is_fully_paid;

                            let plan_clone_edit = plan.clone();
                            let plan_clone_del = plan.clone();
                            let plan_clone_pay = plan.clone();

                            rsx! {
                                div { class: "budget-plan-card", key: "{plan.id}",
                                    // Header do Card
                                    div { class: "budget-plan-header",
                                        div { class: "budget-plan-title-box",
                                            div { class: "flex items-center gap-2 flex-wrap",
                                                h3 { class: "budget-plan-title", "{plan.title}" }
                                                if is_approved {
                                                    span { class: "badge-status badge-completed", "Aprovado" }
                                                } else {
                                                    span { class: "badge-status badge-pending", "Rascunho" }
                                                }

                                                // Financial Badge
                                                if is_fully_paid {
                                                    span { class: "badge-status badge-completed", "Pago 100%" }
                                                } else if is_partial {
                                                    span { class: "badge-status badge-active",
                                                        "Parcial ({format_currency(plan.paid_amount_cents)} pagos)"
                                                    }
                                                } else {
                                                    span { class: "badge-status badge-danger", "Não Pago" }
                                                }
                                            }

                                            div { class: "budget-plan-meta flex items-center gap-2 mt-1 font-xs text-muted",
                                                if let Some(ref d_name) = plan.dentist_user_name {
                                                    span { "Dr(a). {d_name} •" }
                                                }
                                                span { "Criado em {plan.created_at.chars().take(10).collect::<String>()}" }
                                                if let Some(ref s_date) = plan.planned_start_date {
                                                    span { " • Previsão: {s_date}" }
                                                }
                                            }
                                        }

                                        // Ações e Valores no topo do card
                                        div { class: "budget-plan-actions",
                                            // Botão de Aprovação (se for rascunho)
                                            if !is_approved && can_write {
                                                {
                                                    let t_btn = token.clone();
                                                    let pid_btn = patient_id.clone();
                                                    let cid_btn = clinic_id.clone();
                                                    let p_id_btn = plan.id.clone();
                                                    let on_r_btn = reload_patient_details.clone();
                                                    rsx! {
                                                        button {
                                                            class: "btn-primary btn-sm",
                                                            disabled: is_approving(),
                                                            onclick: move |_| {
                                                                let t = t_btn.clone();
                                                                let pid = pid_btn.clone();
                                                                let cid = cid_btn.clone();
                                                                let plan_id = p_id_btn.clone();
                                                                let on_r = on_r_btn.clone();
                                                                is_approving.set(true);
                                                                spawn(async move {
                                                                    let req = UpdateTreatmentPlanStatusRequest {
                                                                        clinic_id: cid,
                                                                        status: TreatmentPlanStatus::Approved,
                                                                    };
                                                                    match update_treatment_plan_status(&t, &pid, &plan_id, req).await {
                                                                        Ok(_) => {
                                                                            toast_msg.set(Some(
                                                                                "Orçamento aprovado! Procedimentos lançados no Prontuário e pendência no Financeiro.".into()
                                                                            ));
                                                                            on_r.call(());
                                                                        }
                                                                        Err(e) => {
                                                                            error_toast.set(Some(format!("Erro ao aprovar orçamento: {}", e)));
                                                                        }
                                                                    }
                                                                    is_approving.set(false);
                                                                });
                                                            },
                                                            IconCheck { size: 14, color: "currentColor".to_string() }
                                                            span { " Aprovar Orçamento" }
                                                        }
                                                    }
                                                }
                                            }

                                            // Botão de Pagamento / Baixa
                                            if is_approved && !is_fully_paid && can_write {
                                                button {
                                                    class: "btn-secondary btn-sm",
                                                    style: "color: #059669; border-color: #a7f3d0; background: #ecfdf5; font-weight: 600;",
                                                    onclick: move |_| {
                                                        let def_reals = (plan_clone_pay.remaining_amount_cents as f64) / 100.0;
                                                        pay_amount_input.set(format!("{:.2}", def_reals));
                                                        pay_method.set("Pix".into());
                                                        pay_notes.set(String::new());
                                                        pay_target_plan.set(Some(plan_clone_pay.clone()));
                                                        is_pay_modal_open.set(true);
                                                    },
                                                    IconCheck { size: 14, color: "currentColor".to_string() }
                                                    span { " Registrar Pagamento" }
                                                }
                                            }

                                            if can_write {
                                                button {
                                                    class: "btn-action-icon",
                                                    title: "Editar Orçamento",
                                                    onclick: move |_| {
                                                        editing_plan.set(Some(plan_clone_edit.clone()));
                                                        is_plan_modal_open.set(true);
                                                    },
                                                    IconEdit { size: 16, color: "#64748b".to_string() }
                                                }
                                            }

                                            if can_delete {
                                                button {
                                                    class: "btn-action-icon text-danger",
                                                    title: "Excluir Orçamento",
                                                    onclick: move |_| {
                                                        delete_plan_target.set(Some(plan_clone_del.clone()));
                                                        is_delete_modal_open.set(true);
                                                    },
                                                    IconTrash { size: 16, color: "#ef4444".to_string() }
                                                }
                                            }
                                        }
                                    }

                                    // Corpo do Card: Mini KPIs + Subtabela
                                    div { class: "budget-plan-body",
                                        // Mini KPIs de Resumo Financeiro do Orçamento
                                        div { class: "budget-plan-kpis",
                                            div { class: "budget-mini-kpi",
                                                span { class: "budget-mini-label", "Valor Total da Proposta" }
                                                span { class: "budget-mini-val text-primary", "{format_currency(plan.total_price_cents)}" }
                                            }
                                            div { class: "budget-mini-kpi",
                                                span { class: "budget-mini-label", "Valor Pago / Amortizado" }
                                                span { class: "budget-mini-val text-success", "{format_currency(plan.paid_amount_cents)}" }
                                            }
                                            div { class: "budget-mini-kpi",
                                                span { class: "budget-mini-label", "Saldo a Liquidar" }
                                                span { class: "budget-mini-val text-warning", "{format_currency(plan.remaining_amount_cents)}" }
                                            }
                                        }

                                        // Subtabela de Procedimentos
                                        div { class: "budget-items-subtable-wrap",
                                            div { class: "budget-items-title flex items-center justify-between",
                                                span { "Procedimentos e Serviços Incluídos" }
                                                span { class: "font-xs font-normal text-muted", "{plan.items.len()} itens no orçamento" }
                                            }
                                            table { class: "budget-items-subtable",
                                                thead {
                                                    tr {
                                                        th { style: "width: 45%;", "Procedimento / Tratamento" }
                                                        th { style: "width: 15%;", "Dente / Região" }
                                                        th { style: "width: 22%;", "Faces / Detalhes" }
                                                        th { class: "text-right", style: "width: 18%;", "Valor" }
                                                    }
                                                }
                                                tbody {
                                                    for item in &plan.items {
                                                        {
                                                            let surf_text = item.surfaces.join(", ");
                                                            rsx! {
                                                                tr { key: "{item.id}",
                                                                    td {
                                                                        div { class: "font-semibold text-slate-800", "{item.procedure_name}" }
                                                                        if let Some(ref cat) = item.category {
                                                                            span { class: "font-xs text-muted", "{cat}" }
                                                                        }
                                                                    }
                                                                    td {
                                                                        if let Some(ref t) = item.tooth_number {
                                                                            span { class: "badge-status-neutral", "Dente {t}" }
                                                                        } else if let Some(ref r) = item.dental_region {
                                                                            span { class: "badge-status-neutral", "{r}" }
                                                                        } else {
                                                                            span { class: "text-muted", "Geral" }
                                                                        }
                                                                    }
                                                                    td {
                                                                        if !surf_text.is_empty() {
                                                                            span { class: "font-xs text-slate-600", "{surf_text}" }
                                                                        } else if let Some(ref n) = item.clinical_notes {
                                                                            span { class: "font-xs text-muted", "{n}" }
                                                                        } else {
                                                                            span { class: "text-muted", "-" }
                                                                        }
                                                                    }
                                                                    td { class: "text-right font-semibold text-slate-800 font-mono",
                                                                        "{format_currency(item.price_cents)}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Observações se houver
                                        if let Some(ref notes) = plan.notes {
                                            if !notes.trim().is_empty() {
                                                div { class: "plan-notes-banner mt-3",
                                                    strong { "Observações: " }
                                                    span { "{notes}" }
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

            // Modal de Pagamento de Orçamento
            if is_pay_modal_open() {
                if let Some(ref target) = *pay_target_plan.read() {
                    div { class: "modal-overlay",
                        div { class: "action-modal", style: "max-width: 480px; border-radius: 12px; background: #ffffff;",
                            div { class: "modal-header", style: "padding: 18px 24px; border-bottom: 1px solid #e2e8f0;",
                                div {
                                    h3 { class: "modal-title text-primary font-bold", style: "font-size: 1.15rem; margin: 0;", "Registrar Pagamento do Orçamento" }
                                    p { class: "modal-subtitle font-xs text-muted mt-1", "{target.title}" }
                                }
                                button { class: "modal-close", onclick: move |_| is_pay_modal_open.set(false), "×" }
                            }

                            div { class: "modal-body", style: "padding: 20px 24px; display: flex; flex-direction: column; gap: 16px;",
                                div { class: "fin-settle-info-card", style: "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px 16px;",
                                    div { class: "flex justify-between items-center",
                                        span { class: "text-muted font-xs", "Total da Proposta:" }
                                        strong { "{format_currency(target.total_price_cents)}" }
                                    }
                                    div { class: "flex justify-between items-center mt-1",
                                        span { class: "text-muted font-xs", "Já Pago / Amortizado:" }
                                        span { class: "text-success font-semibold", "{format_currency(target.paid_amount_cents)}" }
                                    }
                                    div { class: "flex justify-between items-center mt-2 pt-2 border-t",
                                        span { class: "text-slate-800 font-semibold font-sm", "Saldo a Liquidar:" }
                                        strong { class: "text-primary text-base font-mono", "{format_currency(target.remaining_amount_cents)}" }
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Valor a Pagar (R$) *" }
                                    div { class: "flex gap-2",
                                        input {
                                            r#type: "text",
                                            class: "input-field font-mono font-semibold",
                                            style: "height: 40px; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px; width: 100%;",
                                            value: "{pay_amount_input}",
                                            oninput: move |e| pay_amount_input.set(e.value()),
                                            placeholder: "0.00",
                                        }
                                        button {
                                            class: "btn-secondary btn-sm",
                                            style: "height: 40px; padding: 0 14px; font-weight: 600;",
                                            r#type: "button",
                                            onclick: {
                                                let rem = target.remaining_amount_cents;
                                                move |_| {
                                                    let val = (rem as f64) / 100.0;
                                                    pay_amount_input.set(format!("{:.2}", val));
                                                }
                                            },
                                            "Total"
                                        }
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Método de Pagamento (Obrigatório) *" }
                                    select {
                                        class: "select-field",
                                        style: "height: 40px; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px; width: 100%;",
                                        value: "{pay_method}",
                                        onchange: move |e| pay_method.set(e.value()),
                                        option { value: "Pix", "Pix" }
                                        option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                        option { value: "Cartão de Débito", "Cartão de Débito" }
                                        option { value: "Dinheiro", "Dinheiro" }
                                        option { value: "Boleto Bancário", "Boleto Bancário" }
                                        option { value: "Transferência TED/DOC", "Transferência TED/DOC" }
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label font-semibold font-xs text-slate-700 block mb-1", "Observações do Pagamento" }
                                    input {
                                        r#type: "text",
                                        class: "input-field",
                                        style: "height: 40px; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0 12px; width: 100%;",
                                        placeholder: "Ex: Pago 1ª parcela via Pix comprovante #123",
                                        value: "{pay_notes}",
                                        oninput: move |e| pay_notes.set(e.value()),
                                    }
                                }
                            }

                            div { class: "modal-footer", style: "padding: 16px 24px; background: #f8fafc; border-top: 1px solid #e2e8f0; display: flex; justify-content: flex-end; gap: 10px;",
                                button {
                                    class: "btn-secondary",
                                    style: "height: 38px; padding: 0 16px;",
                                    onclick: move |_| is_pay_modal_open.set(false),
                                    "Cancelar"
                                }
                                button {
                                    class: "btn-primary",
                                    style: "height: 38px; padding: 0 18px;",
                                    disabled: is_paying(),
                                    onclick: handle_confirm_payment,
                                    if is_paying() { "Registrando..." } else { "Confirmar Pagamento" }
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

            // Modal de Exclusão de Orçamento
            if is_delete_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal delete-modal-card",
                        div { class: "modal-header",
                            h2 { class: "modal-title text-danger font-bold", "Excluir Orçamento" }
                            button { class: "modal-close", onclick: move |_| is_delete_modal_open.set(false), "×" }
                        }
                        div { class: "modal-body",
                            if let Some(ref plan) = *delete_plan_target.read() {
                                p { "Tem certeza que deseja excluir o orçamento ", strong { "{plan.title}" }, "?" }
                                p { class: "text-muted font-xs mt-2", "Esta ação removerá o lançamento do orçamento." }
                            }
                        }
                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_delete_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-danger",
                                disabled: is_deleting(),
                                onclick: handle_confirm_delete_plan,
                                if is_deleting() { "Excluindo..." } else { "Confirmar Exclusão" }
                            }
                        }
                    }
                }
            }
        }
    }
}
