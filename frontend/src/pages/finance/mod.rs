//! # Módulo de Gestão Financeira (Tooth Plus V2)
//!
//! Exibe o fluxo de caixa da clínica, relatórios consolidados de receitas e despesas,
//! controle de liquidação e registro de lançamentos rápidos.

use crate::api::finance::FinanceApi;
use crate::api::{ActiveClinicState, SessionState};
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconDollar, IconPlus, IconSearch};
use dioxus::prelude::*;
use shared::finance::{
    CreateTransactionRequest, FinanceQuery, FinanceResponse, FinanceSummary,
    Transaction, TransactionDirection, TransactionStatus,
};

const STYLE: Asset = asset!("/src/pages/finance/style.css");

fn format_currency_br(cents: i64) -> String {
    let reais = cents as f64 / 100.0;
    format!("R$ {:.2}", reais).replace('.', ",")
}

#[component]
pub fn FinanceView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut transactions = use_signal(Vec::<Transaction>::new);
    let mut summary = use_signal(FinanceSummary::default);
    let mut is_loading = use_signal(|| true);

    let mut filter_direction = use_signal(|| "all".to_string());
    let mut filter_status = use_signal(|| "all".to_string());
    let mut search_text = use_signal(String::new);

    // Estado do modal de novo lançamento
    let mut show_new_modal = use_signal(|| false);
    let mut modal_direction = use_signal(|| TransactionDirection::Income);
    let mut modal_desc = use_signal(String::new);
    let mut modal_amount_str = use_signal(String::new);
    let mut modal_category = use_signal(|| "Procedimentos Clínicos".to_string());
    let mut modal_due_date = use_signal(|| {
        js_sys::eval("new Date().toISOString().split('T')[0]")
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "2026-08-25".to_string())
    });
    let mut modal_is_paid = use_signal(|| true);

    // Carrega dados financeiros
    let load_finance = {
        let cid = clinic_id.clone();
        let mut trans_sig = transactions.clone();
        let mut sum_sig = summary.clone();
        let mut load_sig = is_loading.clone();
        let mut toast_sig = toast.clone();

        move || {
            let clinic_key = cid.clone();
            let mut t_sig = trans_sig.clone();
            let mut s_sig = sum_sig.clone();
            let mut l_sig = load_sig.clone();
            let mut toast_c = toast_sig.clone();

            spawn(async move {
                l_sig.set(true);
                let q = FinanceQuery {
                    clinic_id: clinic_key,
                    start_date: None,
                    end_date: None,
                    month: None,
                    year: None,
                };
                match FinanceApi::list_transactions(q).await {
                    Ok(resp) => {
                        t_sig.set(resp.transactions);
                        s_sig.set(resp.summary);
                    }
                    Err(e) => {
                        toast_c.show(format!("Erro ao carregar financeiro: {}", e), ToastVariant::Error);
                    }
                }
                l_sig.set(false);
            });
        }
    };

    use_effect({
        let mut lf = load_finance.clone();
        move || {
            lf();
        }
    });

    // Filtra transações na interface
    let dir_filter = filter_direction.read().clone();
    let st_filter = filter_status.read().clone();
    let q_filter = search_text.read().to_lowercase();

    let filtered_transactions: Vec<Transaction> = transactions
        .read()
        .iter()
        .filter(|t| {
            if dir_filter == "income" && t.direction != TransactionDirection::Income { return false; }
            if dir_filter == "expense" && t.direction != TransactionDirection::Expense { return false; }
            if st_filter == "paid" && t.status != TransactionStatus::Paid { return false; }
            if st_filter == "pending" && t.status != TransactionStatus::Pending { return false; }
            if !q_filter.is_empty() && !t.description.to_lowercase().contains(&q_filter) && !t.patient_name.as_deref().unwrap_or("").to_lowercase().contains(&q_filter) {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    // Handler para salvar novo lançamento
    let handle_save_transaction = {
        let cid = clinic_id.clone();
        let mut show_m = show_new_modal.clone();
        let mut toast_s = toast.clone();
        let mut lf = load_finance.clone();

        move |_| {
            let desc = modal_desc.read().trim().to_string();
            let amt_val: f64 = modal_amount_str.read().replace(',', ".").parse().unwrap_or(0.0);
            let cents = (amt_val * 100.0) as i64;

            if desc.is_empty() || cents <= 0 {
                toast_s.show("Informe uma descrição e um valor válido.", ToastVariant::Error);
                return;
            }

            let dir = *modal_direction.read();
            let is_p = *modal_is_paid.read();
            let status = if is_p { TransactionStatus::Paid } else { TransactionStatus::Pending };
            let paid_d = if is_p { Some(modal_due_date.read().clone()) } else { None };

            let req = CreateTransactionRequest {
                clinic_id: cid.clone(),
                appointment_id: None,
                patient_id: None,
                patient_name: None,
                user_id: None,
                treatment_plan_id: None,
                direction: dir,
                amount_cents: cents,
                description: desc,
                category: modal_category.read().clone(),
                due_date: modal_due_date.read().clone(),
                paid_date: paid_d,
                payment_method: Some("PIX".to_string()),
                status,
                installment_current: Some(1),
                installment_total: Some(1),
            };

            let mut toast_c = toast_s.clone();
            let mut show_c = show_m.clone();
            let mut lf_c = lf.clone();

            spawn(async move {
                match FinanceApi::create_transaction(req).await {
                    Ok(_) => {
                        toast_c.show("Lançamento cadastrado com sucesso!", ToastVariant::Success);
                        show_c.set(false);
                        lf_c();
                    }
                    Err(e) => {
                        toast_c.show(format!("Erro ao criar lançamento: {}", e), ToastVariant::Error);
                    }
                }
            });
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "finance-page",

            // 1. KPI Cards
            div { class: "finance-kpi-grid",
                div { class: "finance-kpi-card kpi-income",
                    span { class: "finance-kpi-label", "Total Recebido" }
                    span { class: "finance-kpi-value val-income", "{format_currency_br(summary.read().total_income_cents)}" }
                }
                div { class: "finance-kpi-card kpi-expense",
                    span { class: "finance-kpi-label", "Total Despesas" }
                    span { class: "finance-kpi-value val-expense", "{format_currency_br(summary.read().total_expense_cents)}" }
                }
                div { class: "finance-kpi-card kpi-pending",
                    span { class: "finance-kpi-label", "A Receber (Previsto)" }
                    span { class: "finance-kpi-value val-pending", "{format_currency_br(summary.read().pending_income_cents)}" }
                }
                div { class: "finance-kpi-card kpi-balance",
                    span { class: "finance-kpi-label", "Saldo em Caixa" }
                    span { class: "finance-kpi-value val-balance", "{format_currency_br(summary.read().net_balance_cents)}" }
                }
            }

            // 2. Toolbar & Filtros
            div { class: "finance-toolbar",
                div { class: "finance-filters",
                    select {
                        class: "finance-select",
                        value: "{filter_direction}",
                        onchange: move |e| filter_direction.set(e.value()),
                        option { value: "all", "Todas as movimentações" }
                        option { value: "income", "Apenas Receitas (Entradas)" }
                        option { value: "expense", "Apenas Despesas (Saídas)" }
                    }

                    select {
                        class: "finance-select",
                        value: "{filter_status}",
                        onchange: move |e| filter_status.set(e.value()),
                        option { value: "all", "Todos os status" }
                        option { value: "paid", "Liquidados / Pagos" }
                        option { value: "pending", "Pendentes / A Vencer" }
                    }

                    div { class: "finance-search-box",
                        IconSearch { size: 16, color: "#94a3b8".to_string() }
                        input {
                            class: "finance-search-input",
                            r#type: "text",
                            placeholder: "Buscar por descrição ou paciente...",
                            value: "{search_text}",
                            oninput: move |e| search_text.set(e.value()),
                        }
                    }
                }

                div { class: "finance-actions",
                    button {
                        class: "btn-income",
                        onclick: move |_| {
                            modal_direction.set(TransactionDirection::Income);
                            modal_category.set("Tratamentos & Procedimentos".to_string());
                            modal_desc.set(String::new());
                            modal_amount_str.set(String::new());
                            show_new_modal.set(true);
                        },
                        IconPlus { size: 16, color: "#ffffff".to_string() }
                        span { "Nova Receita" }
                    }
                    button {
                        class: "btn-expense",
                        onclick: move |_| {
                            modal_direction.set(TransactionDirection::Expense);
                            modal_category.set("Despesas Operacionais".to_string());
                            modal_desc.set(String::new());
                            modal_amount_str.set(String::new());
                            show_new_modal.set(true);
                        },
                        IconPlus { size: 16, color: "#ffffff".to_string() }
                        span { "Nova Despesa" }
                    }
                }
            }

            // 3. Tabela de Lançamentos
            div { class: "finance-table-container",
                if is_loading() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "💰" }
                        p { class: "empty-state-title", "Carregando transações..." }
                    }
                } else if filtered_transactions.is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "🔍" }
                        p { class: "empty-state-title", "Nenhuma movimentação encontrada" }
                        p { class: "empty-state-desc", "Utilize os botões acima para registrar receitas ou despesas da clínica." }
                    }
                } else {
                    table { class: "finance-table",
                        thead {
                            tr {
                                th { "Data Venc." }
                                th { "Descrição / Paciente" }
                                th { "Categoria" }
                                th { "Status" }
                                th { "Forma Pgto" }
                                th { style: "text-align: right;", "Valor" }
                            }
                        }
                        tbody {
                            for t in filtered_transactions.iter() {
                                {
                                    let is_inc = t.direction == TransactionDirection::Income;
                                    let is_p = t.status == TransactionStatus::Paid;
                                    let amt_fmt = format_currency_br(t.amount_cents);
                                    let date_fmt = t.due_date.split('T').next().unwrap_or(&t.due_date).to_string();
                                    let status_lbl = t.status.label();
                                    let sign_prefix = if is_inc { "+" } else { "-" };
                                    let method_str = t.payment_method.clone().unwrap_or_else(|| "PIX".to_string());

                                    rsx! {
                                        tr {
                                            key: "{t.id}",
                                            td { "{date_fmt}" }
                                            td {
                                                div { style: "font-weight: 700;", "{t.description}" }
                                                if let Some(ref pn) = t.patient_name {
                                                    div { style: "font-size: 12px; color: #64748b;", "Paciente: {pn}" }
                                                }
                                            }
                                            td { "{t.category}" }
                                            td {
                                                span {
                                                    class: if is_p { "badge-paid" } else { "badge-pending" },
                                                    "{status_lbl}"
                                                }
                                            }
                                            td { "{method_str}" }
                                            td {
                                                style: "text-align: right;",
                                                span {
                                                    class: if is_inc { "amount-income" } else { "amount-expense" },
                                                    "{sign_prefix} {amt_fmt}"
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

            // 4. Modal de Novo Lançamento
            if *show_new_modal.read() {
                div { class: "modal-overlay",
                    onclick: move |_| show_new_modal.set(false),

                    div { class: "modal-box modal-sm", onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            span { class: "modal-title",
                                {
                                    if *modal_direction.read() == TransactionDirection::Income {
                                        "Registrar Receita"
                                    } else {
                                        "Registrar Despesa"
                                    }
                                }
                            }
                            button { class: "modal-close-btn", onclick: move |_| show_new_modal.set(false), "✕" }
                        }

                        div { class: "modal-body",
                            div { class: "form-field",
                                label { class: "form-label", "Descrição *" }
                                input {
                                    class: "form-input",
                                    r#type: "text",
                                    placeholder: "Ex: Restauração Resina - João",
                                    value: "{modal_desc}",
                                    oninput: move |e| modal_desc.set(e.value()),
                                }
                            }

                            div { class: "form-row-2 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Valor (R$) *" }
                                    input {
                                        class: "form-input",
                                        r#type: "number",
                                        step: "0.01",
                                        placeholder: "0,00",
                                        value: "{modal_amount_str}",
                                        oninput: move |e| modal_amount_str.set(e.value()),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Data de Vencimento *" }
                                    input {
                                        class: "form-input",
                                        r#type: "date",
                                        value: "{modal_due_date}",
                                        oninput: move |e| modal_due_date.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-field",
                                label { class: "form-label", "Categoria" }
                                select {
                                    class: "form-select",
                                    value: "{modal_category}",
                                    onchange: move |e| modal_category.set(e.value()),
                                    option { "Tratamentos & Procedimentos" }
                                    option { "Consultas & Avaliações" }
                                    option { "Despesas Operacionais" }
                                    option { "Materiais & Insumos" }
                                    option { "Laboratório de Prótese" }
                                    option { "Aluguel & Contas" }
                                }
                            }

                            label { class: "form-checkbox-wrap",
                                input {
                                    r#type: "checkbox",
                                    checked: "{modal_is_paid}",
                                    onchange: move |e| modal_is_paid.set(e.checked()),
                                }
                                "Lançamento já liquidado (Pago / Recebido na hora)"
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-modal-ghost", onclick: move |_| show_new_modal.set(false), "Cancelar" }
                            button {
                                class: "btn-modal-primary",
                                onclick: handle_save_transaction,
                                "Salvar Lançamento"
                            }
                        }
                    }
                }
            }
        }
    }
}
