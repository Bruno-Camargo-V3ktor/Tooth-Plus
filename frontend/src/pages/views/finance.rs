use crate::api;
use crate::components::icons::*;
use crate::components::ui_blocks::ActionModal;
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use chrono::{Datelike, Local, Utc};
use dioxus::prelude::*;
use shared::finance::{
    CreateTransactionRequest, Transaction, TransactionDirection,
    TransactionStatus, UpdateTransactionStatusRequest,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DateFilterPreset {
    Today,
    Week,
    Month,
    Year,
    Custom,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FinanceTab {
    All,
    Income,
    Expense,
    Pending,
}

fn format_currency(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    let reais = abs / 100;
    let centavos = abs % 100;
    format!("{}R$ {},{:02}", sign, format_thousands(reais), centavos)
}

fn format_thousands(num: i64) -> String {
    let s = num.to_string();
    let mut result = String::new();
    let mut count = 0;
    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push('.');
        }
        result.push(c);
        count += 1;
    }
    result.chars().rev().collect()
}

fn format_date_br(iso_str: &str) -> String {
    if iso_str.len() >= 10 {
        let parts: Vec<&str> = iso_str[0..10].split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
        }
    }
    iso_str.to_string()
}

fn get_today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "Janeiro",
        2 => "Fevereiro",
        3 => "Março",
        4 => "Abril",
        5 => "Maio",
        6 => "Junho",
        7 => "Julho",
        8 => "Agosto",
        9 => "Setembro",
        10 => "Outubro",
        11 => "Novembro",
        12 => "Dezembro",
        _ => "",
    }
}

#[component]
pub fn FinanceView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read_all = permissions::has_permission(&sess, &clinic, "finance:read_all");
    let can_read_income = can_read_all || permissions::has_permission(&sess, &clinic, "finance:read_income");
    let can_read_expense = can_read_all || permissions::has_permission(&sess, &clinic, "finance:read_expense");
    let can_read_pending = can_read_all || permissions::has_permission(&sess, &clinic, "finance:read_pending");
    let can_write_income = permissions::has_permission(&sess, &clinic, "finance:write_income");
    let can_write_expense = permissions::has_permission(&sess, &clinic, "finance:write_expense");
    let can_update_status = permissions::has_permission(&sess, &clinic, "finance:update_status");
    let can_delete = permissions::has_permission(&sess, &clinic, "finance:delete");

    if !can_read_all && !can_read_income && !can_read_expense && !can_read_pending {
        return rsx! {
            div { class: "access-denied-container",
                div { class: "access-denied-card",
                    h2 { "Acesso Restrito" }
                    p { "Você não possui privilégios de acesso para visualizar o módulo financeiro desta clínica." }
                }
            }
        };
    }

    let initial_tab = if can_read_all {
        FinanceTab::All
    } else if can_read_income {
        FinanceTab::Income
    } else if can_read_expense {
        FinanceTab::Expense
    } else {
        FinanceTab::Pending
    };

    let now = Utc::now();
    let mut selected_month = use_signal(|| now.month());
    let mut selected_year = use_signal(|| now.year());
    let mut date_preset = use_signal(|| DateFilterPreset::Month);
    let mut custom_start_date = use_signal(|| Local::now().format("%Y-%m-%d").to_string());
    let mut custom_end_date = use_signal(|| Local::now().format("%Y-%m-%d").to_string());

    let mut active_tab = use_signal(|| initial_tab);
    let mut search_query = use_signal(|| String::new());
    let mut category_filter = use_signal(|| "all".to_string());

    let mut is_create_modal_open = use_signal(|| false);
    let mut create_initial_dir = use_signal(|| TransactionDirection::Income);

    let mut is_settle_modal_open = use_signal(|| false);
    let mut settle_target_tx = use_signal(|| None::<Transaction>);
    let mut settle_payment_method = use_signal(|| "Pix".to_string());

    let mut is_delete_modal_open = use_signal(|| false);
    let mut delete_target_tx = use_signal(|| None::<Transaction>);

    let mut reload_counter = use_signal(|| 0);
    let mut action_error = use_signal(|| None::<String>);

    let clinic_id = clinic.as_ref().map(|c| c.clinic_id.clone()).unwrap_or_default();
    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let cid_res = clinic_id.clone();
    let tok_res = token.clone();

    let finance_resource = use_resource(move || {
        let cid = cid_res.clone();
        let tok = tok_res.clone();
        let preset = date_preset();
        let m = selected_month();
        let y = selected_year();
        let c_start = custom_start_date();
        let c_end = custom_end_date();
        let _ = reload_counter();

        async move {
            if cid.is_empty() || tok.is_empty() {
                return Err("Sessão inválida.".into());
            }

            let (req_m, req_y, req_start, req_end) = match preset {
                DateFilterPreset::Month => (Some(m), Some(y), None, None),
                DateFilterPreset::Today => {
                    let today = Local::now().format("%Y-%m-%d").to_string();
                    (None, None, Some(today.clone()), Some(today))
                }
                DateFilterPreset::Week => {
                    let end = Local::now().format("%Y-%m-%d").to_string();
                    let start = (Local::now() - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
                    (None, None, Some(start), Some(end))
                }
                DateFilterPreset::Year => {
                    let start = format!("{}-01-01", y);
                    let end = format!("{}-12-31", y);
                    (None, None, Some(start), Some(end))
                }
                DateFilterPreset::Custom => {
                    (None, None, Some(c_start), Some(c_end))
                }
            };

            api::fetch_finance_data(&tok, &cid, req_m, req_y, req_start, req_end).await
        }
    });

    let prev_month = move |_| {
        if selected_month() == 1 {
            selected_month.set(12);
            selected_year.set(selected_year() - 1);
        } else {
            selected_month.set(selected_month() - 1);
        }
    };

    let next_month = move |_| {
        if selected_month() == 12 {
            selected_month.set(1);
            selected_year.set(selected_year() + 1);
        } else {
            selected_month.set(selected_month() + 1);
        }
    };

    let current_period_now = move |_| {
        let cur = Utc::now();
        selected_month.set(cur.month());
        selected_year.set(cur.year());
    };

    let open_create_income = move |_| {
        create_initial_dir.set(TransactionDirection::Income);
        is_create_modal_open.set(true);
    };

    let open_create_expense = move |_| {
        create_initial_dir.set(TransactionDirection::Expense);
        is_create_modal_open.set(true);
    };

    let cid_settle = clinic_id.clone();
    let tok_settle = token.clone();
    let handle_settle = move |_| {
        let Some(target) = settle_target_tx() else { return };
        let cid = cid_settle.clone();
        let tok = tok_settle.clone();
        let method = settle_payment_method();
        spawn(async move {
            let req = UpdateTransactionStatusRequest {
                status: TransactionStatus::Paid,
                paid_date: Some(Utc::now().to_rfc3339()),
                payment_method: Some(method),
            };
            match api::update_transaction_status(&tok, &cid, &target.id, req).await {
                Ok(_) => {
                    is_settle_modal_open.set(false);
                    settle_target_tx.set(None);
                    reload_counter.set(reload_counter() + 1);
                }
                Err(err) => {
                    action_error.set(Some(err));
                }
            }
        });
    };

    let cid_del = clinic_id.clone();
    let tok_del = token.clone();
    let handle_delete = move |_| {
        let Some(target) = delete_target_tx() else { return };
        let cid = cid_del.clone();
        let tok = tok_del.clone();
        spawn(async move {
            match api::delete_transaction(&tok, &cid, &target.id).await {
                Ok(_) => {
                    is_delete_modal_open.set(false);
                    delete_target_tx.set(None);
                    reload_counter.set(reload_counter() + 1);
                }
                Err(err) => {
                    action_error.set(Some(err));
                }
            }
        });
    };

    rsx! {
        div { class: "finance-page-container",
            if let Some(err) = action_error() {
                div { class: "toast-error",
                    span { "{err}" }
                    button { class: "toast-close-btn", onclick: move |_| action_error.set(None), "×" }
                }
            }

            div { class: "fin-top-bar",
                div { class: "fin-presets-group",
                    button {
                        class: if date_preset() == DateFilterPreset::Today { "fin-preset-chip active" } else { "fin-preset-chip" },
                        onclick: move |_| date_preset.set(DateFilterPreset::Today),
                        "Hoje"
                    }
                    button {
                        class: if date_preset() == DateFilterPreset::Week { "fin-preset-chip active" } else { "fin-preset-chip" },
                        onclick: move |_| date_preset.set(DateFilterPreset::Week),
                        "Últimos 7 dias"
                    }
                    button {
                        class: if date_preset() == DateFilterPreset::Month { "fin-preset-chip active" } else { "fin-preset-chip" },
                        onclick: move |_| date_preset.set(DateFilterPreset::Month),
                        "Mês"
                    }
                    button {
                        class: if date_preset() == DateFilterPreset::Year { "fin-preset-chip active" } else { "fin-preset-chip" },
                        onclick: move |_| date_preset.set(DateFilterPreset::Year),
                        "Ano"
                    }
                    button {
                        class: if date_preset() == DateFilterPreset::Custom { "fin-preset-chip active" } else { "fin-preset-chip" },
                        onclick: move |_| date_preset.set(DateFilterPreset::Custom),
                        "Personalizado"
                    }
                }

                div { class: "header-actions-group",
                    if can_write_income {
                        button { class: "btn-primary", onclick: open_create_income,
                            IconPlus { size: 16, color: "white".to_string() }
                            span { "Nova Entrada" }
                        }
                    }
                    if can_write_expense {
                        button { class: "btn-secondary", onclick: open_create_expense,
                            IconPlus { size: 16, color: "#1e293b".to_string() }
                            span { "Nova Saída" }
                        }
                    }
                }
            }

            match finance_resource.read().as_ref() {
                None => rsx! {
                    div { class: "agenda-loading-box",
                        p { "Carregando demonstrativo financeiro..." }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "agenda-error-box",
                        p { "{e}" }
                        button { class: "btn-secondary", onclick: move |_| reload_counter.set(reload_counter() + 1), "Tentar Novamente" }
                    }
                },
                Some(Ok(data)) => {
                    let summary = &data.summary;
                    let all_txs = &data.transactions;

                    let filtered_txs: Vec<&Transaction> = all_txs.iter().filter(|t| {
                        match active_tab() {
                            FinanceTab::All => true,
                            FinanceTab::Income => t.direction == TransactionDirection::Income,
                            FinanceTab::Expense => t.direction == TransactionDirection::Expense,
                            FinanceTab::Pending => t.status == TransactionStatus::Pending,
                        }
                    }).filter(|t| {
                        match t.direction {
                            TransactionDirection::Income => can_read_income,
                            TransactionDirection::Expense => can_read_expense,
                        }
                    }).filter(|t| {
                        let query = search_query().to_lowercase();
                        if query.is_empty() { return true; }
                        t.description.to_lowercase().contains(&query)
                            || t.category.to_lowercase().contains(&query)
                            || t.patient_name.as_deref().unwrap_or("").to_lowercase().contains(&query)
                    }).filter(|t| {
                        let cat = category_filter();
                        if cat == "all" { return true; }
                        t.category == cat
                    }).collect();

                    let mut categories_set: Vec<String> = all_txs.iter().map(|t| t.category.clone()).collect();
                    categories_set.sort();
                    categories_set.dedup();

                    let inc_count = all_txs.iter().filter(|t| t.direction == TransactionDirection::Income).count();
                    let exp_count = all_txs.iter().filter(|t| t.direction == TransactionDirection::Expense).count();
                    let pend_count = all_txs.iter().filter(|t| t.status == TransactionStatus::Pending).count();
                    let total_count = all_txs.len();

                    rsx! {
                        div { class: "finance-kpi-row",
                            if can_read_income {
                                div { class: "fin-kpi-card fin-card-income",
                                    div { class: "fin-kpi-header",
                                        span { class: "fin-kpi-title", "Entradas Realizadas" }
                                        span { class: "fin-kpi-badge fin-badge-income", "Receitas" }
                                    }
                                    div { class: "fin-kpi-body",
                                        div { class: "fin-kpi-value text-income", "{format_currency(summary.total_income_cents)}" }
                                        div { class: "fin-kpi-icon-box icon-income", "↓" }
                                    }
                                    div { class: "fin-kpi-footer",
                                        span { "Valores liquidados no período" }
                                    }
                                }
                            }

                            if can_read_expense {
                                div { class: "fin-kpi-card fin-card-expense",
                                    div { class: "fin-kpi-header",
                                        span { class: "fin-kpi-title", "Saídas Realizadas" }
                                        span { class: "fin-kpi-badge fin-badge-expense", "Despesas" }
                                    }
                                    div { class: "fin-kpi-body",
                                        div { class: "fin-kpi-value text-expense", "{format_currency(summary.total_expense_cents)}" }
                                        div { class: "fin-kpi-icon-box icon-expense", "↑" }
                                    }
                                    div { class: "fin-kpi-footer",
                                        span { "Pagamentos e custos efetuados" }
                                    }
                                }
                            }

                            if can_read_income && can_read_expense {
                                div { class: "fin-kpi-card fin-card-balance",
                                    div { class: "fin-kpi-header",
                                        span { class: "fin-kpi-title", "Saldo em Caixa" }
                                        span { class: "fin-kpi-badge fin-badge-balance", "Resultado" }
                                    }
                                    div { class: "fin-kpi-body",
                                        div {
                                            class: if summary.net_balance_cents >= 0 { "fin-kpi-value text-income" } else { "fin-kpi-value text-expense" },
                                            "{format_currency(summary.net_balance_cents)}"
                                        }
                                        div { class: "fin-kpi-icon-box icon-balance", "$" }
                                    }
                                    div { class: "fin-kpi-footer",
                                        span { "Diferença líquida (Entradas - Saídas)" }
                                    }
                                }
                            }

                            if can_read_pending {
                                div { class: "fin-kpi-card fin-card-pending",
                                    div { class: "fin-kpi-header",
                                        span { class: "fin-kpi-title", "Previsão Pendente" }
                                        span { class: "fin-kpi-badge fin-badge-pending", "Aberto" }
                                    }
                                    div { class: "fin-kpi-body",
                                        div { class: "fin-kpi-value text-pending", "{format_currency(summary.pending_income_cents)}" }
                                        div { class: "fin-kpi-icon-box icon-pending", "⏱" }
                                    }
                                    div { class: "fin-kpi-footer",
                                        span { "A receber da agenda e lançamentos" }
                                    }
                                }
                            }
                        }

                        div { class: "fin-controls-toolbar",
                            if date_preset() == DateFilterPreset::Month {
                                div { class: "fin-date-navigator",
                                    button { class: "fin-date-arrow", onclick: prev_month, "‹" }
                                    div { class: "fin-date-label",
                                        "{get_month_name(selected_month())} de {selected_year()}"
                                    }
                                    button { class: "fin-date-arrow", onclick: next_month, "›" }
                                    button { class: "fin-date-today", onclick: current_period_now, "Mês Atual" }
                                }
                            } else if date_preset() == DateFilterPreset::Custom {
                                div { class: "fin-custom-range-wrapper",
                                    label { class: "fin-range-label", "De:" }
                                    input {
                                        class: "modern-input-field fin-range-date-input",
                                        r#type: "date",
                                        value: "{custom_start_date()}",
                                        oninput: move |e| custom_start_date.set(e.value())
                                    }
                                    label { class: "fin-range-label", "Até:" }
                                    input {
                                        class: "modern-input-field fin-range-date-input",
                                        r#type: "date",
                                        value: "{custom_end_date()}",
                                        oninput: move |e| custom_end_date.set(e.value())
                                    }
                                }
                            } else if date_preset() == DateFilterPreset::Today {
                                div { class: "fin-date-info-badge",
                                    "📅 Hoje ({Local::now().format(\"%d/%m/%Y\")})"
                                }
                            } else if date_preset() == DateFilterPreset::Week {
                                div { class: "fin-date-info-badge",
                                    "📅 Últimos 7 dias"
                                }
                            } else {
                                div { class: "fin-date-info-badge",
                                    "📅 Ano de {selected_year()}"
                                }
                            }

                            div { class: "fin-search-filter-group",
                                div { class: "fin-search-input-wrapper",
                                    span { class: "fin-search-icon", "🔍" }
                                    input {
                                        class: "fin-search-input",
                                        placeholder: "Buscar por descrição, paciente ou categoria...",
                                        value: "{search_query()}",
                                        oninput: move |e| search_query.set(e.value())
                                    }
                                }

                                select {
                                    class: "fin-category-select",
                                    value: "{category_filter()}",
                                    onchange: move |e| category_filter.set(e.value()),
                                    option { value: "all", "Todas Categorias" }
                                    for cat in categories_set {
                                        option { value: "{cat}", "{cat}" }
                                    }
                                }
                            }
                        }

                        div { class: "finance-tabs-nav",
                            if can_read_all {
                                button {
                                    class: if active_tab() == FinanceTab::All { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::All),
                                    "Todos ({total_count})"
                                }
                            }
                            if can_read_income {
                                button {
                                    class: if active_tab() == FinanceTab::Income { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::Income),
                                    "Entradas ({inc_count})"
                                }
                            }
                            if can_read_expense {
                                button {
                                    class: if active_tab() == FinanceTab::Expense { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::Expense),
                                    "Saídas ({exp_count})"
                                }
                            }
                            if can_read_pending {
                                button {
                                    class: if active_tab() == FinanceTab::Pending { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::Pending),
                                    "Pendentes ({pend_count})"
                                }
                            }
                        }

                        if filtered_txs.is_empty() {
                            div { class: "agenda-empty-state",
                                p { class: "empty-title", "Nenhuma movimentação encontrada para o período." }
                                p { class: "empty-subtitle", "Altere o filtro de mês ou adicione um novo lançamento." }
                            }
                        } else {
                            div { class: "finance-list-container",
                                for tx in filtered_txs {
                                    div { class: "finance-item-card", key: "{tx.id}",
                                        div { class: "finance-card-left",
                                            div {
                                                class: if tx.direction == TransactionDirection::Income { "fin-dir-indicator dir-income" } else { "fin-dir-indicator dir-expense" },
                                                if tx.direction == TransactionDirection::Income { "↓" } else { "↑" }
                                            }
                                            div { class: "fin-card-info",
                                                div { class: "fin-card-title-row",
                                                    span { class: "fin-tx-description", "{tx.description}" }
                                                    span { class: "fin-category-badge", "{tx.category}" }
                                                    if tx.is_calculated_pending {
                                                        span { class: "fin-simulated-badge", "Agenda Automática" }
                                                    }
                                                }
                                                div { class: "fin-card-meta-row",
                                                    span { class: "fin-meta-item", "Vencimento: {format_date_br(&tx.due_date)}" }
                                                    if let Some(ref p_date) = tx.paid_date {
                                                        span { class: "fin-meta-item", "Pago em: {format_date_br(p_date)}" }
                                                    }
                                                    if let Some(ref method) = tx.payment_method {
                                                        span { class: "fin-meta-item fin-meta-method", "{method}" }
                                                    }
                                                    if tx.installment_total > 1 {
                                                        span { class: "fin-meta-item", "Parcela {tx.installment_current}/{tx.installment_total}" }
                                                    }
                                                    if let Some(ref p_name) = tx.patient_name {
                                                        span { class: "fin-meta-item", "Paciente: {p_name}" }
                                                    }
                                                }
                                            }
                                        }

                                        div { class: "finance-card-right",
                                            div { class: "fin-amount-col",
                                                div {
                                                    class: if tx.direction == TransactionDirection::Income { "fin-amount-text income" } else { "fin-amount-text expense" },
                                                    if tx.direction == TransactionDirection::Income { "+ " } else { "- " }
                                                    "{format_currency(tx.amount_cents)}"
                                                }
                                                span { class: "app-status-badge {tx.status.color_class()}", "{tx.status.label()}" }
                                            }

                                            div { class: "fin-actions-col",
                                                if tx.status == TransactionStatus::Pending && can_update_status && !tx.is_calculated_pending {
                                                    button {
                                                        class: "btn-small btn-primary",
                                                        title: "Confirmar Pagamento / Baixa",
                                                        onclick: {
                                                            let t = (*tx).clone();
                                                            move |_| {
                                                                settle_target_tx.set(Some(t.clone()));
                                                                settle_payment_method.set("Pix".to_string());
                                                                is_settle_modal_open.set(true);
                                                            }
                                                        },
                                                        "Dar Baixa"
                                                    }
                                                }
                                                if can_delete && !tx.is_calculated_pending {
                                                    button {
                                                        class: "icon-action-btn btn-danger-action",
                                                        title: "Excluir Registro",
                                                        onclick: {
                                                            let t = (*tx).clone();
                                                            move |_| {
                                                                delete_target_tx.set(Some(t.clone()));
                                                                is_delete_modal_open.set(true);
                                                            }
                                                        },
                                                        IconTrash { size: 16, color: "#ef4444".to_string() }
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

            TransactionFormModal {
                is_open: is_create_modal_open(),
                clinic_id: clinic_id.clone(),
                token: token.clone(),
                initial_direction: create_initial_dir(),
                on_close: move |_| is_create_modal_open.set(false),
                on_success: move |_| {
                    is_create_modal_open.set(false);
                    reload_counter.set(reload_counter() + 1);
                }
            }

            ActionModal {
                is_open: is_settle_modal_open(),
                title: "Confirmar Recebimento / Pagamento".to_string(),
                on_close: move |_| is_settle_modal_open.set(false),
                div { class: "modal-form-vertical",
                    p { class: "app-modal-hint",
                        if let Some(ref target) = settle_target_tx() {
                            "Você está confirmando o pagamento de: "
                            b { "{target.description} ({format_currency(target.amount_cents)})" }
                        } else {
                            ""
                        }
                    }

                    div { class: "input-group-wrapper full-width",
                        label { "Forma de Pagamento" }
                        select {
                            class: "modern-input-field",
                            value: "{settle_payment_method()}",
                            onchange: move |e| settle_payment_method.set(e.value()),
                            option { value: "Pix", "Pix" }
                            option { value: "Cartão de Crédito", "Cartão de Crédito" }
                            option { value: "Cartão de Débito", "Cartão de Débito" }
                            option { value: "Boleto Bancário", "Boleto Bancário" }
                            option { value: "Dinheiro", "Dinheiro (Espécie)" }
                            option { value: "Transferência Bancária", "Transferência / TED" }
                        }
                    }

                    div { class: "modal-footer-actions",
                        button { class: "btn-secondary", onclick: move |_| is_settle_modal_open.set(false), "Cancelar" }
                        button { class: "btn-primary", onclick: handle_settle, "Confirmar Baixa" }
                    }
                }
            }

            ActionModal {
                is_open: is_delete_modal_open(),
                title: "Excluir Lançamento Financeiro".to_string(),
                on_close: move |_| is_delete_modal_open.set(false),
                div { class: "modal-form-vertical",
                    p { class: "app-modal-hint text-error-small",
                        "Tem certeza que deseja remover esta transação financeira? Esta ação não pode ser desfeita."
                    }
                    div { class: "modal-footer-actions",
                        button { class: "btn-secondary", onclick: move |_| is_delete_modal_open.set(false), "Cancelar" }
                        button { class: "btn-danger", onclick: handle_delete, "Confirmar Exclusão" }
                    }
                }
            }
        }
    }
}

#[component]
fn TransactionFormModal(
    is_open: bool,
    clinic_id: String,
    token: String,
    initial_direction: TransactionDirection,
    on_close: EventHandler<()>,
    on_success: EventHandler<()>,
) -> Element {
    let mut direction = use_signal(|| initial_direction);
    let mut description = use_signal(|| String::new());
    let mut category = use_signal(|| "Procedimento Clínico".to_string());
    let mut amount_input = use_signal(|| String::new());
    let mut due_date = use_signal(|| get_today_iso());
    let mut is_paid_now = use_signal(|| true);
    let mut payment_method = use_signal(|| "Pix".to_string());
    let mut is_submitting = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    use_effect(use_reactive(&initial_direction, move |d| {
        direction.set(d);
        if d == TransactionDirection::Expense {
            category.set("Insumos & Estoque".to_string());
        } else {
            category.set("Procedimento Clínico".to_string());
        }
    }));

    let cid = clinic_id.clone();
    let tok = token.clone();

    let handle_submit = move |_| {
        if description().trim().is_empty() {
            form_error.set(Some("Informe a descrição do lançamento.".into()));
            return;
        }

        let clean_amount = amount_input().replace(',', ".").replace("R$", "").trim().to_string();
        let Ok(amt_float) = clean_amount.parse::<f64>() else {
            form_error.set(Some("Informe um valor monetário válido.".into()));
            return;
        };

        if amt_float <= 0.0 {
            form_error.set(Some("O valor deve ser maior que zero.".into()));
            return;
        }

        let amount_cents = (amt_float * 100.0).round() as i64;
        let due_iso = format!("{}T12:00:00Z", due_date());
        let (st, paid_iso, method) = if is_paid_now() {
            (TransactionStatus::Paid, Some(Utc::now().to_rfc3339()), Some(payment_method()))
        } else {
            (TransactionStatus::Pending, None, None)
        };

        let req = CreateTransactionRequest {
            clinic_id: cid.clone(),
            appointment_id: None,
            patient_id: None,
            patient_name: None,
            user_id: None,
            direction: direction(),
            amount_cents,
            description: description().trim().to_string(),
            category: category(),
            due_date: due_iso,
            paid_date: paid_iso,
            payment_method: method,
            status: st,
            installment_current: Some(1),
            installment_total: Some(1),
        };

        is_submitting.set(true);
        form_error.set(None);
        let t = tok.clone();

        spawn(async move {
            match api::create_transaction(&t, req).await {
                Ok(_) => {
                    is_submitting.set(false);
                    on_success.call(());
                }
                Err(err) => {
                    is_submitting.set(false);
                    form_error.set(Some(err));
                }
            }
        });
    };

    rsx! {
        ActionModal {
            is_open,
            title: if direction() == TransactionDirection::Income { "Nova Entrada Financeira".to_string() } else { "Nova Saída / Despesa".to_string() },
            on_close: move |_| on_close.call(()),
            div { class: "modal-form-vertical",
                if let Some(err) = form_error() {
                    div { class: "toast-error", "{err}" }
                }

                div { class: "form-grid",
                    div { class: "input-group-wrapper",
                        label { "Tipo de Movimentação" }
                        select {
                            class: "modern-input-field",
                            value: if direction() == TransactionDirection::Income { "income" } else { "expense" },
                            onchange: move |e| {
                                if e.value() == "income" {
                                    direction.set(TransactionDirection::Income);
                                    category.set("Procedimento Clínico".to_string());
                                } else {
                                    direction.set(TransactionDirection::Expense);
                                    category.set("Insumos & Estoque".to_string());
                                }
                            },
                            option { value: "income", "Entrada (Receita)" }
                            option { value: "expense", "Saída (Despesa)" }
                        }
                    }

                    div { class: "input-group-wrapper",
                        label { "Categoria" }
                        select {
                            class: "modern-input-field",
                            value: "{category()}",
                            onchange: move |e| category.set(e.value()),
                            if direction() == TransactionDirection::Income {
                                option { value: "Procedimento Clínico", "Procedimento Clínico" }
                                option { value: "Consulta", "Consulta" }
                                option { value: "Ortodontia", "Ortodontia" }
                                option { value: "Implante", "Implante" }
                                option { value: "Estética", "Estética" }
                                option { value: "Outra Receita", "Outra Receita" }
                            } else {
                                option { value: "Insumos & Estoque", "Insumos & Estoque" }
                                option { value: "Manutenção", "Manutenção e Equipamentos" }
                                option { value: "Aluguel & Contas", "Aluguel & Contas Básicas" }
                                option { value: "Laboratório de Prótese", "Laboratório de Prótese" }
                                option { value: "Comissão & Honorários", "Comissão & Honorários" }
                                option { value: "Marketing & Software", "Marketing & Software" }
                                option { value: "Outra Despesa", "Outra Despesa" }
                            }
                        }
                    }

                    div { class: "input-group-wrapper full-width",
                        label { "Descrição do Lançamento" }
                        input {
                            class: "modern-input-field",
                            placeholder: "Ex: Clareamento a Laser, Dental Cremer, etc.",
                            value: "{description()}",
                            oninput: move |e| description.set(e.value())
                        }
                    }

                    div { class: "input-group-wrapper",
                        label { "Valor (R$)" }
                        input {
                            class: "modern-input-field",
                            placeholder: "0,00",
                            value: "{amount_input()}",
                            oninput: move |e| amount_input.set(e.value())
                        }
                    }

                    div { class: "input-group-wrapper",
                        label { "Data de Vencimento" }
                        input {
                            class: "modern-input-field",
                            r#type: "date",
                            value: "{due_date()}",
                            oninput: move |e| due_date.set(e.value())
                        }
                        div { class: "agenda-quick-dates-row",
                            button {
                                class: "agenda-quick-date-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let target = Local::now();
                                    due_date.set(target.format("%Y-%m-%d").to_string());
                                },
                                "Hoje"
                            }
                            button {
                                class: "agenda-quick-date-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let target = Local::now() + chrono::Duration::days(1);
                                    due_date.set(target.format("%Y-%m-%d").to_string());
                                },
                                "Amanhã"
                            }
                            button {
                                class: "agenda-quick-date-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let target = Local::now() + chrono::Duration::days(7);
                                    due_date.set(target.format("%Y-%m-%d").to_string());
                                },
                                "+7 dias"
                            }
                            button {
                                class: "agenda-quick-date-btn",
                                r#type: "button",
                                onclick: move |_| {
                                    let target = Local::now() + chrono::Duration::days(30);
                                    due_date.set(target.format("%Y-%m-%d").to_string());
                                },
                                "+30 dias"
                            }
                        }
                    }

                    div { class: "input-group-wrapper full-width",
                        label { "Status do Pagamento" }
                        div { class: "checkbox-group-wrapper",
                            label { class: "checkbox-label",
                                input {
                                    r#type: "checkbox",
                                    checked: is_paid_now(),
                                    onchange: move |e| is_paid_now.set(e.checked())
                                }
                                span { "Já foi pago / liquidado agora" }
                            }
                        }
                    }

                    if is_paid_now() {
                        div { class: "input-group-wrapper full-width",
                            label { "Forma de Pagamento Utilizada" }
                            select {
                                class: "modern-input-field",
                                value: "{payment_method()}",
                                onchange: move |e| payment_method.set(e.value()),
                                option { value: "Pix", "Pix" }
                                option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                option { value: "Cartão de Débito", "Cartão de Débito" }
                                option { value: "Boleto Bancário", "Boleto Bancário" }
                                option { value: "Dinheiro", "Dinheiro (Espécie)" }
                                option { value: "Transferência Bancária", "Transferência / TED" }
                            }
                        }
                    }
                }

                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", r#type: "button", onclick: move |_| on_close.call(()), "Cancelar" }
                    button { class: "btn-primary", r#type: "button", onclick: handle_submit, disabled: is_submitting(),
                        if is_submitting() { "Registrando..." } else { "Salvar Lançamento" }
                    }
                }
            }
        }
    }
}
