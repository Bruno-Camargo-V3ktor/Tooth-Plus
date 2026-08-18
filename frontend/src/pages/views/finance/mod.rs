//! # Módulo de Gestão Financeira da Clínica Odontológica (Frontend)
//!
//! Controla o fluxo de caixa, receitas de procedimentos, comissões de dentistas,
//! despesas operacionais e liquidação de pagamentos.

pub mod transaction_modal;
pub mod transactions_table;

pub use transaction_modal::*;
pub use transactions_table::*;

use crate::api::{delete_transaction, fetch_finance_data, update_transaction_status};
use crate::components::icons::{
    IconArrowDown, IconArrowUp, IconCheck, IconFinance, IconPlus, IconSearch,
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use chrono::{Datelike, Utc};
use dioxus::prelude::*;
use shared::finance::{
    FinanceSummary, Transaction, TransactionDirection, TransactionStatus,
    UpdateTransactionStatusRequest,
};

/// Formata moeda BRL para exibição em KPIs.
fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;
    if is_negative {
        format!("-R$ {}.{:02}", reals, centavos)
    } else {
        format!("R$ {}.{:02}", reals, centavos)
    }
}

/// Abas de filtro do módulo financeiro.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FinanceTab {
    All,
    Income,
    Expense,
    Pending,
}

/// Presets de período
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DateFilterPreset {
    Today,
    Week,
    Month,
    Year,
    Custom,
}

/// Componente principal da tela de Gestão Financeira.
#[component]
pub fn FinanceView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read_all = permissions::has_permission(&sess, &clinic, "finance:read_all");
    let can_read_income =
        can_read_all || permissions::has_permission(&sess, &clinic, "finance:read_income");
    let can_read_expense =
        can_read_all || permissions::has_permission(&sess, &clinic, "finance:read_expense");
    let can_read_pending =
        can_read_all || permissions::has_permission(&sess, &clinic, "finance:read_pending");
    let can_write_income = permissions::has_permission(&sess, &clinic, "finance:write_income");
    let can_write_expense = permissions::has_permission(&sess, &clinic, "finance:write_expense");
    let can_update_status = permissions::has_permission(&sess, &clinic, "finance:update_status");
    let can_delete = permissions::has_permission(&sess, &clinic, "finance:delete");

    if !can_read_all && !can_read_income && !can_read_expense && !can_read_pending {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar o módulo financeiro desta clínica." }
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

    let mut active_tab = use_signal(|| initial_tab);
    let mut search_query = use_signal(String::new);
    let mut reload_counter = use_signal(|| 0usize);
    let mut toast_msg = use_signal(|| None::<String>);

    let mut is_create_modal_open = use_signal(|| false);
    let mut create_initial_dir = use_signal(|| TransactionDirection::Income);

    let mut is_settle_modal_open = use_signal(|| false);
    let mut settle_target_tx = use_signal(|| None::<Transaction>);
    let mut settle_payment_method = use_signal(|| "Pix".to_string());
    let mut is_settling = use_signal(|| false);

    let mut is_delete_modal_open = use_signal(|| false);
    let mut delete_target_tx = use_signal(|| None::<Transaction>);
    let mut is_deleting = use_signal(|| false);

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();
    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let cid_res = clinic_id.clone();
    let tok_res = token.clone();

    let finance_resource = use_resource(move || {
        let cid = cid_res.clone();
        let tok = tok_res.clone();
        let m = selected_month();
        let y = selected_year();
        let _ = reload_counter();

        async move {
            if cid.is_empty() || tok.is_empty() {
                return Err("Sessão inválida.".into());
            }
            fetch_finance_data(&tok, &cid, Some(m), Some(y), None, None).await
        }
    });

    let open_create_income = move |_| {
        create_initial_dir.set(TransactionDirection::Income);
        is_create_modal_open.set(true);
    };

    let open_create_expense = move |_| {
        create_initial_dir.set(TransactionDirection::Expense);
        is_create_modal_open.set(true);
    };

    let tok_set = token.clone();
    let cid_set = clinic_id.clone();
    let mut handle_settle = move |_| {
        let Some(target) = settle_target_tx() else {
            return;
        };
        let cid = cid_set.clone();
        let tok = tok_set.clone();
        let method = settle_payment_method();
        let mut set_open = is_settle_modal_open;
        let mut target_sig = settle_target_tx;
        let mut rel_sig = reload_counter;
        let mut toast = toast_msg;
        let mut set_sig = is_settling;

        set_sig.set(true);
        spawn(async move {
            let req = UpdateTransactionStatusRequest {
                status: TransactionStatus::Paid,
                paid_date: Some(Utc::now().to_rfc3339()),
                payment_method: Some(method),
            };
            match update_transaction_status(&tok, &cid, &target.id, req).await {
                Ok(_) => {
                    set_open.set(false);
                    target_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Lançamento liquidado com sucesso!".into()));
                }
                Err(err) => {
                    toast.set(Some(err));
                }
            }
            set_sig.set(false);
        });
    };

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();
    let mut handle_delete = move |_| {
        let Some(target) = delete_target_tx() else {
            return;
        };
        let cid = cid_del.clone();
        let tok = tok_del.clone();
        let mut del_open = is_delete_modal_open;
        let mut target_sig = delete_target_tx;
        let mut rel_sig = reload_counter;
        let mut toast = toast_msg;
        let mut del_sig = is_deleting;

        del_sig.set(true);
        spawn(async move {
            match delete_transaction(&tok, &cid, &target.id).await {
                Ok(_) => {
                    del_open.set(false);
                    target_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Lançamento excluído com sucesso!".into()));
                }
                Err(err) => {
                    toast.set(Some(err));
                }
            }
            del_sig.set(false);
        });
    };

    rsx! {
        div { class: "finance-page-container",
            if let Some(ref err) = *toast_msg.read() {
                div { class: "toast-error",
                    span { "{err}" }
                    button { class: "toast-close-btn", onclick: move |_| toast_msg.set(None), "×" }
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
                        "Mês Atual"
                    }
                    button {
                        class: if date_preset() == DateFilterPreset::Year { "fin-preset-chip active" } else { "fin-preset-chip" },
                        onclick: move |_| date_preset.set(DateFilterPreset::Year),
                        "Ano"
                    }
                }

                div { class: "header-actions-group",
                    if can_write_income {
                        button { class: "btn-primary", onclick: open_create_income,
                            IconPlus { size: 16, color: "white".to_string() }
                            span { " Nova Entrada" }
                        }
                    }
                    if can_write_expense {
                        button { class: "btn-secondary", onclick: open_create_expense,
                            IconPlus { size: 16, color: "#1e293b".to_string() }
                            span { " Nova Saída" }
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

                    let filtered_txs: Vec<Transaction> = all_txs.iter().filter(|t| {
                        match active_tab() {
                            FinanceTab::All => true,
                            FinanceTab::Income => t.direction == TransactionDirection::Income,
                            FinanceTab::Expense => t.direction == TransactionDirection::Expense,
                            FinanceTab::Pending => t.status == TransactionStatus::Pending,
                        }
                    }).filter(|t| {
                        let q = search_query().to_lowercase();
                        if q.is_empty() { return true; }
                        t.description.to_lowercase().contains(&q)
                            || t.category.to_lowercase().contains(&q)
                            || t.patient_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    }).cloned().collect();

                    rsx! {
                        div { class: "finance-kpi-row",
                            div { class: "fin-kpi-card",
                                div { class: "fin-kpi-header",
                                    span { class: "fin-kpi-title", "Total de Receitas" }
                                    span { class: "fin-kpi-badge badge-green", "Entradas" }
                                }
                                div { class: "fin-kpi-body",
                                    div { class: "fin-kpi-value text-green", "{format_currency(summary.total_income_cents)}" }
                                    div { class: "fin-kpi-icon-box bg-green",
                                        IconArrowUp { size: 20, color: "#10b981".to_string() }
                                    }
                                }
                                div { class: "fin-kpi-footer",
                                    span { "A receber pendente: {format_currency(summary.pending_income_cents)}" }
                                }
                            }

                            div { class: "fin-kpi-card",
                                div { class: "fin-kpi-header",
                                    span { class: "fin-kpi-title", "Total de Despesas" }
                                    span { class: "fin-kpi-badge badge-red", "Saídas" }
                                }
                                div { class: "fin-kpi-body",
                                    div { class: "fin-kpi-value text-red", "{format_currency(summary.total_expense_cents)}" }
                                    div { class: "fin-kpi-icon-box bg-red",
                                        IconArrowDown { size: 20, color: "#ef4444".to_string() }
                                    }
                                }
                                div { class: "fin-kpi-footer",
                                    span { "A pagar pendente: {format_currency(summary.pending_expense_cents)}" }
                                }
                            }

                            div { class: "fin-kpi-card",
                                div { class: "fin-kpi-header",
                                    span { class: "fin-kpi-title", "Resultado Líquido" }
                                    span { class: if summary.net_balance_cents >= 0 { "fin-kpi-badge badge-blue" } else { "fin-kpi-badge badge-red" }, "Balanço" }
                                }
                                div { class: "fin-kpi-body",
                                    div { class: if summary.net_balance_cents >= 0 { "fin-kpi-value text-blue" } else { "fin-kpi-value text-red" },
                                        "{format_currency(summary.net_balance_cents)}"
                                    }
                                    div { class: "fin-kpi-icon-box bg-blue",
                                        IconFinance { size: 20, color: "#0052cc".to_string() }
                                    }
                                }
                                div { class: "fin-kpi-footer",
                                    span { "{summary.total_transactions_count} lançamentos totais" }
                                }
                            }
                        }

                        div { class: "fin-filter-row",
                            div { class: "fin-search-bar",
                                div { class: "search-icon", IconSearch { size: 16, color: "currentColor".to_string() } }
                                input {
                                    class: "search-input",
                                    placeholder: "Buscar por descrição, paciente ou categoria...",
                                    value: "{search_query}",
                                    oninput: move |e| search_query.set(e.value())
                                }
                            }

                            div { class: "fin-tabs-bar",
                                button {
                                    class: if active_tab() == FinanceTab::All { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::All),
                                    span { "Todos" }
                                }
                                button {
                                    class: if active_tab() == FinanceTab::Income { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::Income),
                                    span { "Receitas" }
                                }
                                button {
                                    class: if active_tab() == FinanceTab::Expense { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::Expense),
                                    span { "Despesas" }
                                }
                                button {
                                    class: if active_tab() == FinanceTab::Pending { "fin-tab-btn active" } else { "fin-tab-btn" },
                                    onclick: move |_| active_tab.set(FinanceTab::Pending),
                                    span { "Pendentes" }
                                }
                            }
                        }

                        TransactionsTableSection {
                            transactions: filtered_txs,
                            can_update_status,
                            can_delete,
                            on_settle: move |tx: Transaction| {
                                settle_target_tx.set(Some(tx));
                                is_settle_modal_open.set(true);
                            },
                            on_delete: move |tx: Transaction| {
                                delete_target_tx.set(Some(tx));
                                is_delete_modal_open.set(true);
                            },
                        }
                    }
                }
            }

            if is_create_modal_open() {
                TransactionModal {
                    is_open: is_create_modal_open,
                    initial_direction: create_initial_dir(),
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    reload_counter,
                    toast_msg,
                }
            }

            if is_settle_modal_open() {
                if let Some(ref tx) = *settle_target_tx.read() {
                    div { class: "modal-overlay",
                        div { class: "action-modal",
                            div { class: "modal-header",
                                h2 { class: "modal-title", "Liquidar Pagamento" }
                                button { class: "modal-close", onclick: move |_| is_settle_modal_open.set(false), "×" }
                            }
                            div { class: "modal-body",
                                p { "Confirmar recebimento / pagamento de ", strong { "{tx.description}" }, " no valor de ", strong { "{format_currency(tx.amount_cents)}" }, "?" }
                                div { class: "form-group mt-3",
                                    label { "Forma de Pagamento" }
                                    select {
                                        class: "form-input",
                                        value: "{settle_payment_method}",
                                        onchange: move |e| settle_payment_method.set(e.value()),
                                        option { value: "Pix", "Pix" }
                                        option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                        option { value: "Cartão de Débito", "Cartão de Débito" }
                                        option { value: "Dinheiro", "Dinheiro" }
                                        option { value: "Boleto", "Boleto Bancário" }
                                        option { value: "Transferência", "Transferência TED/DOC" }
                                    }
                                }
                            }
                            div { class: "modal-footer-actions",
                                button { class: "btn-secondary", onclick: move |_| is_settle_modal_open.set(false), "Cancelar" }
                                button {
                                    class: "btn-primary",
                                    disabled: is_settling(),
                                    onclick: move |e| handle_settle(e),
                                    if is_settling() { "Liquidando..." } else { "Confirmar Liquidação" }
                                }
                            }
                        }
                    }
                }
            }

            if is_delete_modal_open() {
                if let Some(ref tx) = *delete_target_tx.read() {
                    div { class: "modal-overlay",
                        div { class: "action-modal delete-modal-card",
                            div { class: "modal-header",
                                h2 { class: "modal-title text-danger", "Excluir Lançamento Financeiro" }
                                button { class: "modal-close", onclick: move |_| is_delete_modal_open.set(false), "×" }
                            }
                            div { class: "modal-body",
                                p { "Tem certeza que deseja excluir o lançamento ", strong { "{tx.description}" }, "?" }
                                p { class: "text-muted font-xs mt-2", "Esta ação impactará os relatórios e o saldo consolidado." }
                            }
                            div { class: "modal-footer-actions",
                                button { class: "btn-secondary", onclick: move |_| is_delete_modal_open.set(false), "Cancelar" }
                                button {
                                    class: "btn-danger",
                                    disabled: is_deleting(),
                                    onclick: move |e| handle_delete(e),
                                    if is_deleting() { "Excluindo..." } else { "Confirmar Exclusão" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
