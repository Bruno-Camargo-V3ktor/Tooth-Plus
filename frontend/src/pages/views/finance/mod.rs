//! # Módulo de Gestão Financeira e Fluxo de Caixa (Frontend)
//!
//! Controla receitas, despesas, conciliação e demonstrativo de resultados.

pub mod transaction_modal;
pub mod transactions_table;

pub use transaction_modal::*;
pub use transactions_table::*;

use crate::api::{delete_transaction, fetch_finance_data, update_transaction_status};
use crate::components::icons::{
    IconCalendar, IconChevronLeft, IconChevronRight, IconClock, IconFinance, IconPlus, IconRefresh,
    IconSearch,
};
use crate::permissions::has_permission;
use crate::{ActiveClinicState, SessionState};
use chrono::Datelike;
use dioxus::prelude::*;
use shared::finance::{
    FinanceResponse, FinanceSummary, Transaction, TransactionDirection, TransactionStatus,
    UpdateTransactionStatusRequest,
};

/// Predefinições de filtro por período temporal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFilterPreset {
    Today,
    Week,
    Month,
    Year,
    Custom,
}

/// Abas de navegação do módulo financeiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinanceTab {
    All,
    Income,
    Expense,
    Pending,
}

/// Formata valor em centavos para moeda BRL com separador de milhar.
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

/// Retorna os limites da semana (Segunda-feira a Domingo) com base em uma data de referência.
fn get_week_bounds(ref_date: chrono::NaiveDate) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let weekday_num = ref_date.weekday().num_days_from_monday();
    let monday = ref_date - chrono::Duration::days(weekday_num as i64);
    let sunday = monday + chrono::Duration::days(6);
    (monday, sunday)
}

/// Formata a data com o dia da semana em português (ex: "Hoje • Quarta-feira (18/08/2026)").
fn format_date_with_weekday(date_str: &str) -> String {
    let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return date_str.to_string();
    };

    let today = chrono::Local::now().date_naive();
    let weekday_pt = match date.weekday() {
        chrono::Weekday::Mon => "Segunda-feira",
        chrono::Weekday::Tue => "Terça-feira",
        chrono::Weekday::Wed => "Quarta-feira",
        chrono::Weekday::Thu => "Quinta-feira",
        chrono::Weekday::Fri => "Sexta-feira",
        chrono::Weekday::Sat => "Sábado",
        chrono::Weekday::Sun => "Domingo",
    };

    if date == today {
        format!("Hoje • {} ({})", weekday_pt, date.format("%d/%m/%Y"))
    } else {
        format!("{} • {}", date.format("%d/%m/%Y"), weekday_pt)
    }
}

#[component]
pub fn FinanceView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read_all = has_permission(&sess, &clinic, "finance:read_all")
        || has_permission(&sess, &clinic, "finance:read");
    let can_read_income = can_read_all || has_permission(&sess, &clinic, "finance:read_income");
    let can_read_expense = can_read_all || has_permission(&sess, &clinic, "finance:read_expense");
    let can_read_pending = can_read_all || has_permission(&sess, &clinic, "finance:read_pending");
    let can_read = can_read_all || can_read_income || can_read_expense || can_read_pending;

    let can_write_income = has_permission(&sess, &clinic, "finance:write_income")
        || has_permission(&sess, &clinic, "finance:write");
    let can_write_expense = has_permission(&sess, &clinic, "finance:write_expense")
        || has_permission(&sess, &clinic, "finance:write");
    let can_write = can_write_income || can_write_expense;

    let can_update_status = has_permission(&sess, &clinic, "finance:update_status");
    let can_delete = has_permission(&sess, &clinic, "finance:delete");

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para visualizar o fluxo financeiro desta clínica." }
            }
        };
    }

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();
    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let mut active_tab = use_signal(move || {
        if can_read_all {
            FinanceTab::All
        } else if can_read_income {
            FinanceTab::Income
        } else if can_read_expense {
            FinanceTab::Expense
        } else {
            FinanceTab::Pending
        }
    });

    let mut active_preset = use_signal(|| DateFilterPreset::Month);

    let current_dt = chrono::Local::now();
    let mut selected_month = use_signal(|| current_dt.month() as i32);
    let mut selected_year = use_signal(|| current_dt.year());

    let mut selected_date_str = use_signal(|| current_dt.format("%Y-%m-%d").to_string());
    let mut selected_week_str = use_signal(|| current_dt.format("%Y-%m-%d").to_string());

    let mut custom_start_date = use_signal(String::new);
    let mut custom_end_date = use_signal(String::new);

    let mut search_query = use_signal(String::new);
    let mut filter_category = use_signal(|| "all".to_string());

    let mut is_create_modal_open = use_signal(|| false);
    let mut create_initial_dir = use_signal(|| TransactionDirection::Income);

    let mut is_settle_modal_open = use_signal(|| false);
    let mut settle_target_tx = use_signal(|| None::<Transaction>);
    let mut settle_amount_input = use_signal(String::new);
    let mut settle_payment_method = use_signal(|| "Pix".to_string());
    let mut settle_notes = use_signal(String::new);
    let mut is_settling = use_signal(|| false);

    let mut is_delete_modal_open = use_signal(|| false);
    let mut delete_target_tx = use_signal(|| None::<Transaction>);
    let mut is_deleting = use_signal(|| false);

    let mut reload_counter = use_signal(|| 0);
    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let finance_resource = use_resource(move || {
        let t = tok.clone();
        let c = cid.clone();
        let preset = active_preset();
        let s_date = selected_date_str();
        let s_week = selected_week_str();
        let s_month = selected_month();
        let s_year = selected_year();
        let c_start = custom_start_date();
        let c_end = custom_end_date();
        let _rel = reload_counter();

        async move {
            if t.is_empty() || c.is_empty() {
                return None;
            }

            let (start_opt, end_opt) = match preset {
                DateFilterPreset::Today => {
                    let start = format!("{}T00:00:00Z", s_date);
                    let end = format!("{}T23:59:59Z", s_date);
                    (Some(start), Some(end))
                }
                DateFilterPreset::Week => {
                    let ref_d = chrono::NaiveDate::parse_from_str(&s_week, "%Y-%m-%d")
                        .unwrap_or_else(|_| chrono::Local::now().date_naive());
                    let (mon, sun) = get_week_bounds(ref_d);
                    let start = format!("{}T00:00:00Z", mon.format("%Y-%m-%d"));
                    let end = format!("{}T23:59:59Z", sun.format("%Y-%m-%d"));
                    (Some(start), Some(end))
                }
                DateFilterPreset::Month => {
                    let start = format!("{:04}-{:02}-01T00:00:00Z", s_year, s_month);
                    let days_in_month = match s_month {
                        4 | 6 | 9 | 11 => 30,
                        2 => {
                            if (s_year % 4 == 0 && s_year % 100 != 0) || (s_year % 400 == 0) {
                                29
                            } else {
                                28
                            }
                        }
                        _ => 31,
                    };
                    let end = format!("{:04}-{:02}-{:02}T23:59:59Z", s_year, s_month, days_in_month);
                    (Some(start), Some(end))
                }
                DateFilterPreset::Year => {
                    let start = format!("{:04}-01-01T00:00:00Z", s_year);
                    let end = format!("{:04}-12-31T23:59:59Z", s_year);
                    (Some(start), Some(end))
                }
                DateFilterPreset::Custom => {
                    let start = if !c_start.is_empty() {
                        Some(format!("{}T00:00:00Z", c_start))
                    } else {
                        None
                    };
                    let end = if !c_end.is_empty() {
                        Some(format!("{}T23:59:59Z", c_end))
                    } else {
                        None
                    };
                    (start, end)
                }
            };

            fetch_finance_data(&t, &c, None, None, start_opt, end_opt).await.ok()
        }
    });

    let tok_set = token.clone();
    let cid_set = clinic_id.clone();

    let mut handle_settle = move |_| {
        let Some(ref tx) = *settle_target_tx.read() else {
            return;
        };
        let method = settle_payment_method().trim().to_string();
        if method.is_empty() {
            let mut err = error_toast;
            err.set(Some("Escolha um método de pagamento obrigatório.".into()));
            return;
        }

        let clean_val = settle_amount_input()
            .replace("R$", "")
            .replace(".", "")
            .replace(",", ".")
            .trim()
            .to_string();

        let amount_float: f64 = match clean_val.parse() {
            Ok(v) if v > 0.0 => v,
            _ => {
                let mut err = error_toast;
                err.set(Some("Informe um valor de liquidação válido maior que zero.".into()));
                return;
            }
        };

        let amount_cents = (amount_float * 100.0).round() as i64;
        let tx_id = tx.id.clone();
        let t = tok_set.clone();
        let c = cid_set.clone();
        let notes_val = if settle_notes().trim().is_empty() { None } else { Some(settle_notes().trim().to_string()) };
        let mut is_set = is_settling;
        let mut open_sig = is_settle_modal_open;
        let mut rel_sig = reload_counter;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_set.set(true);
        spawn(async move {
            let req = shared::finance::RegisterPaymentRequest {
                clinic_id: c,
                amount_cents,
                payment_method: method,
                paid_date: Some(chrono::Utc::now().to_rfc3339()),
                notes: notes_val,
            };
            match crate::api::finance::register_transaction_payment(&t, &tx_id, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Pagamento / liquidação registrada com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao liquidar lançamento: {}", e)));
                }
            }
            is_set.set(false);
        });
    };

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();

    let mut handle_delete = move |_| {
        let Some(ref tx) = *delete_target_tx.read() else {
            return;
        };
        let tx_id = tx.id.clone();
        let t = tok_del.clone();
        let c = cid_del.clone();
        let mut open_sig = is_delete_modal_open;
        let mut rel_sig = reload_counter;
        let mut is_del = is_deleting;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_del.set(true);
        spawn(async move {
            match delete_transaction(&t, &c, &tx_id).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Lançamento excluído!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir lançamento: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    let handle_prev_month = move |_| {
        if selected_month() == 1 {
            selected_month.set(12);
            selected_year.set(selected_year() - 1);
        } else {
            selected_month.set(selected_month() - 1);
        }
    };

    let handle_next_month = move |_| {
        if selected_month() == 12 {
            selected_month.set(1);
            selected_year.set(selected_year() + 1);
        } else {
            selected_month.set(selected_month() + 1);
        }
    };

    let handle_current_month = move |_| {
        let now = chrono::Local::now();
        selected_month.set(now.month() as i32);
        selected_year.set(now.year());
    };

    let handle_prev_week = move |_| {
        let ref_d = chrono::NaiveDate::parse_from_str(&selected_week_str(), "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let new_d = ref_d - chrono::Duration::days(7);
        selected_week_str.set(new_d.format("%Y-%m-%d").to_string());
    };

    let handle_next_week = move |_| {
        let ref_d = chrono::NaiveDate::parse_from_str(&selected_week_str(), "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let new_d = ref_d + chrono::Duration::days(7);
        selected_week_str.set(new_d.format("%Y-%m-%d").to_string());
    };

    let handle_current_week = move |_| {
        let now = chrono::Local::now().date_naive();
        selected_week_str.set(now.format("%Y-%m-%d").to_string());
    };

    let handle_prev_day = move |_| {
        let ref_d = chrono::NaiveDate::parse_from_str(&selected_date_str(), "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let new_d = ref_d - chrono::Duration::days(1);
        selected_date_str.set(new_d.format("%Y-%m-%d").to_string());
    };

    let handle_next_day = move |_| {
        let ref_d = chrono::NaiveDate::parse_from_str(&selected_date_str(), "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let new_d = ref_d + chrono::Duration::days(1);
        selected_date_str.set(new_d.format("%Y-%m-%d").to_string());
    };

    let handle_today_day = move |_| {
        let now = chrono::Local::now().date_naive();
        selected_date_str.set(now.format("%Y-%m-%d").to_string());
    };

    let handle_prev_year = move |_| {
        selected_year.set(selected_year() - 1);
    };

    let handle_next_year = move |_| {
        selected_year.set(selected_year() + 1);
    };

    let handle_current_year = move |_| {
        selected_year.set(chrono::Local::now().year());
    };

    let month_name = match selected_month() {
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
        _ => "Dezembro",
    };

    let period_display_label = format!("{} de {}", month_name, selected_year());
    let formatted_day_display = format_date_with_weekday(&selected_date_str());

    let formatted_week_display = {
        let ref_d = chrono::NaiveDate::parse_from_str(&selected_week_str(), "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let (mon, sun) = get_week_bounds(ref_d);
        format!("{} a {}", mon.format("%d/%m/%Y"), sun.format("%d/%m/%Y"))
    };

    rsx! {
        div { class: "documents-view-container",
            if let Some(ref msg) = *toast_msg.read() {
                div { class: "toast toast-success",
                    span { "{msg}" }
                    button { class: "toast-close", onclick: move |_| toast_msg.set(None), "✕" }
                }
            }
            if let Some(ref err) = *error_toast.read() {
                div { class: "toast toast-error",
                    span { "{err}" }
                    button { class: "toast-close", onclick: move |_| error_toast.set(None), "✕" }
                }
            }

            match &*finance_resource.read() {
                None => rsx! {
                    div { class: "loading-card",
                        div { class: "loading-spinner" }
                        p { "Carregando fluxo financeiro..." }
                    }
                },
                Some(None) => rsx! {
                    div { class: "empty-state-card",
                        IconFinance { size: 48, color: "#94a3b8".to_string() }
                        h3 { "Falha ao carregar dados financeiros" }
                        p { "Verifique a conexão ou tente novamente." }
                    }
                },
                Some(Some(res)) => {
                    let summary = &res.summary;
                    let all_txs = &res.transactions;

                    let filtered_txs: Vec<Transaction> = all_txs
                        .iter()
                        .filter(|tx| {
                            let match_tab = match active_tab() {
                                FinanceTab::All => true,
                                FinanceTab::Income => tx.direction == TransactionDirection::Income,
                                FinanceTab::Expense => tx.direction == TransactionDirection::Expense,
                                FinanceTab::Pending => tx.status == TransactionStatus::Pending,
                            };

                            let match_search = if search_query().trim().is_empty() {
                                true
                            } else {
                                let q = search_query().to_lowercase();
                                let formatted_cat = format_category_display(&tx.category).to_lowercase();
                                tx.description.to_lowercase().contains(&q)
                                    || tx.patient_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                    || tx.category.to_lowercase().contains(&q)
                                    || formatted_cat.contains(&q)
                            };

                            let match_cat = if filter_category() == "all" {
                                true
                            } else {
                                format_category_display(&tx.category) == filter_category() || tx.category == filter_category()
                            };

                            match_tab && match_search && match_cat
                        })
                        .cloned()
                        .collect();

                    let all_count = all_txs.len();
                    let income_count = all_txs.iter().filter(|t| t.direction == TransactionDirection::Income).count();
                    let expense_count = all_txs.iter().filter(|t| t.direction == TransactionDirection::Expense).count();
                    let pending_count = all_txs.iter().filter(|t| t.status == TransactionStatus::Pending).count();

                    rsx! {
                        // 1. TOP: Main Tabs Switcher (Padrão Underline de Estoque e Documentos)
                        div { class: "documents-tab-bar",
                            if can_read_all {
                                button {
                                    class: if active_tab() == FinanceTab::All { "doc-main-tab active" } else { "doc-main-tab" },
                                    onclick: move |_| active_tab.set(FinanceTab::All),
                                    IconFinance { size: 16, color: "currentColor".to_string() }
                                    span { " Todas as Movimentações ({all_count})" }
                                }
                            }
                            if can_read_income {
                                button {
                                    class: if active_tab() == FinanceTab::Income { "doc-main-tab active" } else { "doc-main-tab" },
                                    onclick: move |_| active_tab.set(FinanceTab::Income),
                                    span { class: "type-card-arrow-icon text-income", "↓" }
                                    span { " Receitas ({income_count})" }
                                }
                            }
                            if can_read_expense {
                                button {
                                    class: if active_tab() == FinanceTab::Expense { "doc-main-tab active" } else { "doc-main-tab" },
                                    onclick: move |_| active_tab.set(FinanceTab::Expense),
                                    span { class: "type-card-arrow-icon text-expense", "↑" }
                                    span { " Despesas ({expense_count})" }
                                }
                            }
                            if can_read_pending {
                                button {
                                    class: if active_tab() == FinanceTab::Pending { "doc-main-tab active" } else { "doc-main-tab" },
                                    onclick: move |_| active_tab.set(FinanceTab::Pending),
                                    IconClock { size: 16, color: "currentColor".to_string() }
                                    span { " Pendentes ({pending_count})" }
                                }
                            }
                        }


                        // 2. 4 Compact Horizontal KPIs
                        div { class: "agenda-kpi-row",
                            // 1. ENTRADAS REALIZADAS
                            if can_read_income {
                                div { class: "agenda-kpi-card",
                                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                                        span { class: "type-card-arrow-icon text-income", "↓" }
                                    }
                                    div { class: "agenda-kpi-text-col",
                                        span { class: "agenda-kpi-lbl", "Entradas Realizadas" }
                                        span { class: "agenda-kpi-sublbl", "Receitas liquidadas" }
                                    }
                                    div { class: "agenda-kpi-val kpi-completed", "{format_currency(summary.total_income_cents)}" }
                                }
                            }

                            // 2. SAÍDAS REALIZADAS
                            if can_read_expense {
                                div { class: "agenda-kpi-card",
                                    div { class: "agenda-kpi-icon-wrapper kpi-icon-expense",
                                        span { class: "type-card-arrow-icon text-expense", "↑" }
                                    }
                                    div { class: "agenda-kpi-text-col",
                                        span { class: "agenda-kpi-lbl", "Saídas Realizadas" }
                                        span { class: "agenda-kpi-sublbl", "Despesas pagas" }
                                    }
                                    div { class: "agenda-kpi-val kpi-expense", "{format_currency(summary.total_expense_cents)}" }
                                }
                            }

                            // 3. SALDO EM CAIXA
                            if can_read_all || (can_read_income && can_read_expense) {
                                div { class: "agenda-kpi-card",
                                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                                        span { class: "font-bold", "$" }
                                    }
                                    div { class: "agenda-kpi-text-col",
                                        span { class: "agenda-kpi-lbl", "Saldo em Caixa" }
                                        span { class: "agenda-kpi-sublbl", "Diferença líquida" }
                                    }
                                    div { class: "agenda-kpi-val", "{format_currency(summary.net_balance_cents)}" }
                                }
                            }

                            // 4. PREVISÃO PENDENTE
                            if can_read_pending {
                                div { class: "agenda-kpi-card",
                                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                                        IconClock { size: 16, color: "currentColor".to_string() }
                                    }
                                    div { class: "agenda-kpi-text-col",
                                        span { class: "agenda-kpi-lbl", "Previsão Pendente" }
                                        span { class: "agenda-kpi-sublbl", "A receber da agenda" }
                                    }
                                    div { class: "agenda-kpi-val kpi-pending", "{format_currency(summary.pending_income_cents)}" }
                                }
                            }
                        }


                        // 3. MIDDLE: Preset Filters on Left, Action Buttons on Right
                        div { class: "fin-top-header-row",
                            div { class: "fin-presets-group",
                                button {
                                    class: if active_preset() == DateFilterPreset::Today { "fin-preset-btn active" } else { "fin-preset-btn" },
                                    onclick: move |_| active_preset.set(DateFilterPreset::Today),
                                    "Hoje"
                                }
                                button {
                                    class: if active_preset() == DateFilterPreset::Week { "fin-preset-btn active" } else { "fin-preset-btn" },
                                    onclick: move |_| active_preset.set(DateFilterPreset::Week),
                                    "Semana"
                                }
                                button {
                                    class: if active_preset() == DateFilterPreset::Month { "fin-preset-btn active" } else { "fin-preset-btn" },
                                    onclick: move |_| active_preset.set(DateFilterPreset::Month),
                                    "Mês"
                                }
                                button {
                                    class: if active_preset() == DateFilterPreset::Year { "fin-preset-btn active" } else { "fin-preset-btn" },
                                    onclick: move |_| active_preset.set(DateFilterPreset::Year),
                                    "Ano"
                                }
                                button {
                                    class: if active_preset() == DateFilterPreset::Custom { "fin-preset-btn active" } else { "fin-preset-btn" },
                                    onclick: move |_| active_preset.set(DateFilterPreset::Custom),
                                    "Personalizado"
                                }
                            }

                            if can_write_income || can_write_expense {
                                div { class: "fin-top-actions-group",
                                    if can_write_income {
                                        button {
                                            class: "btn-primary",
                                            onclick: move |_| {
                                                create_initial_dir.set(TransactionDirection::Income);
                                                is_create_modal_open.set(true);
                                            },
                                            IconPlus { size: 16, color: "currentColor".to_string() }
                                            span { " Nova Entrada" }
                                        }
                                    }
                                    if can_write_expense {
                                        button {
                                            class: "btn-secondary btn-nova-saida",
                                            onclick: move |_| {
                                                create_initial_dir.set(TransactionDirection::Expense);
                                                is_create_modal_open.set(true);
                                            },
                                            IconPlus { size: 16, color: "currentColor".to_string() }
                                            span { " Nova Saída" }
                                        }
                                    }
                                }
                            }
                        }


                        // 4. Controls Toolbar (Date Navigator + Search + Category Select + Refresh)
                        div { class: "view-toolbar",
                            if active_preset() == DateFilterPreset::Month {
                                div { class: "fin-date-navigator",
                                    button { class: "fin-date-arrow", onclick: handle_prev_month, title: "Mês Anterior",
                                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                                    }
                                    div { class: "fin-date-label-wrapper",
                                        IconCalendar { size: 15, color: "currentColor".to_string() }
                                        span { class: "fin-date-label", "{period_display_label}" }
                                    }
                                    button { class: "fin-date-arrow", onclick: handle_next_month, title: "Próximo Mês",
                                        IconChevronRight { size: 16, color: "currentColor".to_string() }
                                    }
                                    button { class: "fin-date-today", onclick: handle_current_month, "Mês Atual" }
                                }
                            } else if active_preset() == DateFilterPreset::Week {
                                div { class: "fin-date-navigator",
                                    button { class: "fin-date-arrow", onclick: handle_prev_week, title: "Semana Anterior",
                                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                                    }
                                    div { class: "fin-date-label-wrapper",
                                        IconCalendar { size: 15, color: "currentColor".to_string() }
                                        span { class: "fin-date-label", "{formatted_week_display}" }
                                    }
                                    button { class: "fin-date-arrow", onclick: handle_next_week, title: "Próxima Semana",
                                        IconChevronRight { size: 16, color: "currentColor".to_string() }
                                    }
                                    button { class: "fin-date-today", onclick: handle_current_week, "Semana Atual" }
                                }
                            } else if active_preset() == DateFilterPreset::Today {
                                div { class: "fin-date-navigator",
                                    button { class: "fin-date-arrow", onclick: handle_prev_day, title: "Dia Anterior",
                                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                                    }
                                    div { class: "fin-date-label-wrapper",
                                        IconCalendar { size: 15, color: "currentColor".to_string() }
                                        span { class: "fin-date-label", "{formatted_day_display}" }
                                    }
                                    button { class: "fin-date-arrow", onclick: handle_next_day, title: "Próximo Dia",
                                        IconChevronRight { size: 16, color: "currentColor".to_string() }
                                    }
                                    button { class: "fin-date-today", onclick: handle_today_day, "Hoje" }
                                }
                            } else if active_preset() == DateFilterPreset::Year {
                                div { class: "fin-date-navigator",
                                    button { class: "fin-date-arrow", onclick: handle_prev_year, title: "Ano Anterior",
                                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                                    }
                                    div { class: "fin-date-label-wrapper",
                                        IconCalendar { size: 15, color: "currentColor".to_string() }
                                        span { class: "fin-date-label", "Ano de {selected_year}" }
                                    }
                                    button { class: "fin-date-arrow", onclick: handle_next_year, title: "Próximo Ano",
                                        IconChevronRight { size: 16, color: "currentColor".to_string() }
                                    }
                                    button { class: "fin-date-today", onclick: handle_current_year, "Ano Atual" }
                                }
                            } else if active_preset() == DateFilterPreset::Custom {
                                div { class: "fin-custom-date-row",
                                    div { class: "fin-custom-date-field",
                                        span { class: "fin-custom-lbl", "De:" }
                                        input {
                                            class: "fin-custom-input",
                                            r#type: "date",
                                            value: "{custom_start_date}",
                                            oninput: move |e| custom_start_date.set(e.value()),
                                        }
                                    }
                                    div { class: "fin-custom-date-field",
                                        span { class: "fin-custom-lbl", "Até:" }
                                        input {
                                            class: "fin-custom-input",
                                            r#type: "date",
                                            value: "{custom_end_date}",
                                            oninput: move |e| custom_end_date.set(e.value()),
                                        }
                                    }
                                }
                            }

                            div { class: "search-input-wrap",
                                IconSearch { size: 18, color: "#94a3b8".to_string() }
                                input {
                                    r#type: "text",
                                    class: "search-input",
                                    placeholder: "Buscar por descrição, paciente ou categoria...",
                                    value: "{search_query}",
                                    oninput: move |e| search_query.set(e.value()),
                                }
                            }

                            div { class: "toolbar-actions",
                                select {
                                    class: "modern-select fin-cat-select",
                                    value: "{filter_category}",
                                    onchange: move |e: FormEvent| filter_category.set(e.value()),
                                    option { value: "all", "Todas as Categorias" }
                                    option { value: "Procedimento Clínico", "Procedimentos Clínicos" }
                                    option { value: "Tratamento Odontológico", "Tratamento Odontológico" }
                                    option { value: "Cirurgia", "Cirurgia" }
                                    option { value: "Retorno", "Retorno" }
                                    option { value: "Insumos & Estoque", "Insumos & Estoque" }
                                    option { value: "Custos Fixos / Aluguel", "Custos Fixos / Aluguel" }
                                    option { value: "Água / Luz / Internet", "Água / Luz / Internet" }
                                    option { value: "Salários & Repasses", "Salários & Repasses" }
                                    option { value: "Manutenção & Equipamentos", "Manutenção & Equipamentos" }
                                    option { value: "Outra Receita", "Outra Receita" }
                                    option { value: "Outra Despesa", "Outra Despesa" }
                                }

                                button {
                                    class: "btn-refresh",
                                    onclick: move |_| reload_counter.set(reload_counter() + 1),
                                    title: "Recarregar dados",
                                    IconRefresh { size: 16, color: "#475569".to_string() }
                                }
                            }
                        }

                        // 5. Lista de Lançamentos
                        TransactionsTableSection {
                            transactions: filtered_txs,
                            can_update_status,
                            can_delete,
                            on_settle: move |tx: Transaction| {
                                let rem_cents = if tx.remaining_amount_cents > 0 {
                                    tx.remaining_amount_cents
                                } else if tx.paid_amount_cents == 0 {
                                    tx.amount_cents
                                } else {
                                    (tx.amount_cents - tx.paid_amount_cents).max(0)
                                };
                                let val_reals = (rem_cents as f64) / 100.0;
                                settle_amount_input.set(format!("{:.2}", val_reals));
                                settle_payment_method.set(tx.payment_method.clone().unwrap_or_else(|| "Pix".to_string()));
                                settle_notes.set(String::new());
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
                    can_write_income,
                    can_write_expense,
                    reload_counter,
                    toast_msg,
                    error_toast,
                }
            }


            if is_settle_modal_open() {
                if let Some(ref tx) = *settle_target_tx.read() {
                    {
                        let total_cents = tx.amount_cents;
                        let paid_cents = tx.paid_amount_cents;
                        let rem_cents = if tx.remaining_amount_cents > 0 { tx.remaining_amount_cents } else { (total_cents - paid_cents).max(0) };

                        rsx! {
                            div { class: "modal-overlay",
                                div { class: "action-modal modal-small settle-modal-card", style: "max-width: 480px;",
                                    div { class: "settings-header",
                                        h2 { class: "settings-title text-primary", "Liquidar / Receber Movimentação" }
                                        button { class: "close-btn", onclick: move |_| is_settle_modal_open.set(false), "×" }
                                    }
                                    div { class: "settings-content",
                                        div { class: "fin-settle-info-card mb-3",
                                            div { class: "fin-settle-desc font-semibold text-slate-800", "{tx.description}" }
                                            div { class: "flex justify-between items-center mt-2 text-xs text-muted",
                                                span { "Categoria:" }
                                                span { "{format_category_display(&tx.category)}" }
                                            }
                                            div { class: "flex justify-between items-center mt-1 text-xs text-muted",
                                                span { "Valor Total:" }
                                                strong { "{format_currency(total_cents)}" }
                                            }
                                            if paid_cents > 0 {
                                                div { class: "flex justify-between items-center mt-1 text-xs text-success",
                                                    span { "Já Pago:" }
                                                    strong { "{format_currency(paid_cents)}" }
                                                }
                                            }
                                            div { class: "flex justify-between items-center mt-2 pt-2 border-t font-semibold text-sm",
                                                span { "Saldo a Liquidar:" }
                                                span { class: "text-primary text-base", "{format_currency(rem_cents)}" }
                                            }
                                        }

                                        div { class: "form-group mb-3",
                                            label { class: "form-label font-xs font-semibold", "Valor do Pagamento (R$) *" }
                                            div { class: "flex gap-2",
                                                input {
                                                    r#type: "text",
                                                    class: "input-field font-mono",
                                                    value: "{settle_amount_input}",
                                                    oninput: move |e| settle_amount_input.set(e.value()),
                                                    placeholder: "0.00",
                                                }
                                                button {
                                                    class: "btn-secondary btn-sm",
                                                    r#type: "button",
                                                    onclick: move |_| {
                                                        let val = (rem_cents as f64) / 100.0;
                                                        settle_amount_input.set(format!("{:.2}", val));
                                                    },
                                                    "Totalidade"
                                                }
                                            }
                                        }

                                        div { class: "form-group mb-3",
                                            label { class: "form-label font-xs font-semibold", "Método de Pagamento (Obrigatório) *" }
                                            select {
                                                class: "select-field",
                                                value: "{settle_payment_method}",
                                                onchange: move |e| settle_payment_method.set(e.value()),
                                                option { value: "Pix", "Pix" }
                                                option { value: "Cartão de Crédito", "Cartão de Crédito" }
                                                option { value: "Cartão de Débito", "Cartão de Débito" }
                                                option { value: "Dinheiro", "Dinheiro" }
                                                option { value: "Boleto Bancário", "Boleto Bancário" }
                                                option { value: "Transferência TED/DOC", "Transferência TED/DOC" }
                                            }
                                        }

                                        div { class: "form-group mb-2",
                                            label { class: "form-label font-xs", "Observações / Comprovante" }
                                            input {
                                                r#type: "text",
                                                class: "input-field",
                                                placeholder: "Ex: Comprovante Pix #1234, terminal 2...",
                                                value: "{settle_notes}",
                                                oninput: move |e| settle_notes.set(e.value()),
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
                }
            }

            if is_delete_modal_open() {
                if let Some(ref tx) = *delete_target_tx.read() {
                    div { class: "modal-overlay",
                        div { class: "action-modal modal-small delete-modal-card",
                            div { class: "settings-header",
                                h2 { class: "settings-title text-danger", "Excluir Movimentação" }
                                button { class: "close-btn", onclick: move |_| is_delete_modal_open.set(false), "×" }
                            }
                            div { class: "settings-content",
                                div { class: "fin-delete-info-card",
                                    div { class: "fin-delete-desc", "{tx.description}" }
                                    div { class: "fin-delete-val", "{format_currency(tx.amount_cents)}" }
                                }
                                div { class: "alert-banner alert-warning mt-3",
                                    span { "Atenção: Esta ação não pode ser desfeita e removerá o lançamento do fluxo de caixa." }
                                }
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
