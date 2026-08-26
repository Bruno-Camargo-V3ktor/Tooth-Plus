pub mod components;

use crate::api::finance::FinanceApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::finance::{
    CreateTransactionRequest, FinanceQuery, FinanceSummary, Transaction,
    TransactionDirection, TransactionStatus,
};
use dioxus::prelude::*;

pub use components::{FinanceKpis, FinanceTable, FinanceToolbar, ModalTransaction};

const STYLE: Asset = asset!("/src/pages/finance/style.css");

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
    let mut summary = use_signal(|| FinanceSummary {
        total_income_cents: 0,
        total_expense_cents: 0,
        net_balance_cents: 0,
        pending_income_cents: 0,
        pending_expense_cents: 0,
        total_transactions_count: 0,
    });
    let mut type_filter = use_signal(|| "ALL".to_string());
    let mut search_query = use_signal(String::new);
    let mut show_modal = use_signal(|| false);
    let mut is_income = use_signal(|| true);

    // Form fields
    let mut description = use_signal(String::new);
    let mut amount_str = use_signal(String::new);
    let mut category = use_signal(|| "Tratamentos".to_string());
    let mut payment_method = use_signal(|| "PIX".to_string());
    let mut due_date = use_signal(|| "2026-08-26".to_string());
    let mut is_paid = use_signal(|| true);

    let load_finance = {
        let cid = clinic_id.clone();
        let mut tx_sig = transactions;
        let mut sum_sig = summary;

        move || {
            let cid = cid.clone();
            let query = FinanceQuery {
                clinic_id: cid,
                month: None,
                year: None,
                start_date: None,
                end_date: None,
            };

            spawn(async move {
                if let Ok(resp) = FinanceApi::list_transactions(query).await {
                    tx_sig.set(resp.transactions);
                    sum_sig.set(resp.summary);
                }
            });
        }
    };

    use_effect({
        let mut loader = load_finance.clone();
        move || loader()
    });

    let handle_submit = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut loader = load_finance.clone();
        let mut modal_sig = show_modal;
        let is_inc = is_income.clone();
        let desc = description.clone();
        let amt = amount_str.clone();
        let cat = category.clone();
        let pm = payment_method.clone();
        let dd = due_date.clone();
        let paid = is_paid.clone();

        move |_| {
            let desc_val = desc.read().trim().to_string();
            let amt_val = amt.read().trim().replace(',', ".");
            let parsed_amt: f64 = amt_val.parse().unwrap_or(0.0);

            if desc_val.is_empty() || parsed_amt <= 0.0 {
                toast_c.show("Preencha a descrição e um valor válido.", ToastVariant::Error);
                return;
            }

            let amount_cents = (parsed_amt * 100.0) as i64;
            let direction = if *is_inc.read() { TransactionDirection::Income } else { TransactionDirection::Expense };
            let status = if *paid.read() { TransactionStatus::Paid } else { TransactionStatus::Pending };

            let req = CreateTransactionRequest {
                clinic_id: cid.clone(),
                appointment_id: None,
                patient_id: None,
                patient_name: None,
                user_id: None,
                treatment_plan_id: None,
                direction,
                amount_cents,
                description: desc_val,
                category: cat.read().clone(),
                due_date: dd.read().clone(),
                paid_date: if *paid.read() { Some(dd.read().clone()) } else { None },
                payment_method: Some(pm.read().clone()),
                status,
                installment_current: Some(1),
                installment_total: Some(1),
            };

            let mut toast_resp = toast_c.clone();
            let mut loader_c = loader.clone();
            let mut modal_c = modal_sig;

            spawn(async move {
                match FinanceApi::create_transaction(req).await {
                    Ok(_) => {
                        toast_resp.show("Lançamento registrado com sucesso!", ToastVariant::Success);
                        modal_c.set(false);
                        loader_c();
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let filtered_tx: Vec<Transaction> = transactions.read().iter().filter(|tx| {
        let tf = type_filter.read().clone();
        if tf == "INCOME" && tx.direction != TransactionDirection::Income { return false; }
        if tf == "EXPENSE" && tx.direction != TransactionDirection::Expense { return false; }

        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        tx.description.to_lowercase().contains(&q) || tx.category.to_lowercase().contains(&q)
    }).cloned().collect();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "finance-page",
            FinanceKpis { summary: summary() }

            FinanceToolbar {
                type_filter,
                search_query,
                on_search: move |_| {
                    let mut loader = load_finance.clone();
                    loader();
                },
                on_new_income: move |_| {
                    is_income.set(true);
                    show_modal.set(true);
                },
                on_new_expense: move |_| {
                    is_income.set(false);
                    show_modal.set(true);
                },
            }

            FinanceTable { transactions: filtered_tx }

            ModalTransaction {
                is_open: show_modal(),
                is_income: is_income(),
                description,
                amount_str,
                category,
                payment_method,
                due_date,
                is_paid,
                on_close: move |_| show_modal.set(false),
                on_submit: handle_submit,
            }
        }
    }
}
