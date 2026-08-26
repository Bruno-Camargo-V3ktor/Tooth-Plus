pub mod components;

use crate::api::finance::FinanceApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::finance::{
    CreateTransactionRequest, FinanceQuery, RegisterPaymentRequest, Transaction,
    TransactionDirection, TransactionStatus,
};
use dioxus::prelude::*;

pub use components::*;

const STYLE: Asset = asset!("/src/pages/finance/style.css");

#[component]
pub fn FinanceView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut transactions = use_signal(Vec::<Transaction>::new);
    let mut search_query = use_signal(String::new);
    let mut period_filter = use_signal(|| "month".to_string());
    let mut start_date = use_signal(|| "2026-08-26".to_string());
    let mut end_date = use_signal(|| "2026-08-26".to_string());

    let mut is_filter_modal_open = use_signal(|| false);
    let mut is_add_modal_open = use_signal(|| false);
    let mut add_direction = use_signal(|| TransactionDirection::Income);
    let mut selected_tx_id = use_signal(|| None::<String>);
    let mut is_payment_modal_open = use_signal(|| false);
    let mut is_details_modal_open = use_signal(|| false);
    let mut reload_trigger = use_signal(|| 0);

    let mut filter_income = use_signal(|| true);
    let mut filter_unlinked = use_signal(|| true);
    let mut filter_expense = use_signal(|| true);
    let mut filter_paid = use_signal(|| true);
    let mut filter_unpaid = use_signal(|| true);
    let mut filter_scheduled = use_signal(|| true);
    let mut account_filter = use_signal(|| "all".to_string());
    let mut payment_method_filter = use_signal(|| "all".to_string());

    let mut desc = use_signal(String::new);
    let mut amount_str = use_signal(|| "150.00".to_string());
    let mut category = use_signal(|| "Tratamento Odontológico".to_string());
    let mut payment_method = use_signal(|| "pix".to_string());
    let mut is_paid = use_signal(|| true);
    let mut due_date = use_signal(|| "2026-08-26".to_string());

    let cid_effect = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_effect.clone();
        let query = FinanceQuery {
            clinic_id: cid,
            month: None,
            year: None,
            start_date: None,
            end_date: None,
        };

        spawn(async move {
            if let Ok(resp) = FinanceApi::list_transactions(query).await {
                transactions.set(resp.transactions);
            }
        });
    });

    let handle_submit = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut modal_sig = is_add_modal_open;
        let mut reload_sig = reload_trigger;
        let dir_sig = add_direction;
        let d_sig = desc.clone();
        let a_sig = amount_str.clone();
        let c_sig = category.clone();
        let pm_sig = payment_method.clone();
        let ip_sig = is_paid;
        let dd_sig = due_date.clone();

        move |_| {
            let description = d_sig.read().trim().to_string();
            if description.is_empty() {
                toast_c.show("Informe a descrição do lançamento.", ToastVariant::Error);
                return;
            }

            let amount_num: f64 = a_sig.read().replace(',', ".").parse().unwrap_or(0.0);
            let amount_cents = (amount_num * 100.0) as i64;
            let dir = dir_sig.read().clone();
            let paid = *ip_sig.read();

            let req = CreateTransactionRequest {
                clinic_id: cid.clone(),
                appointment_id: None,
                patient_id: None,
                patient_name: None,
                user_id: None,
                treatment_plan_id: None,
                direction: dir,
                amount_cents,
                description,
                category: c_sig.read().clone(),
                due_date: dd_sig.read().clone(),
                paid_date: if paid { Some(dd_sig.read().clone()) } else { None },
                payment_method: Some(pm_sig.read().clone()),
                status: if paid { TransactionStatus::Paid } else { TransactionStatus::Pending },
                installment_current: None,
                installment_total: None,
            };

            let mut toast_resp = toast_c.clone();
            let mut modal_c = modal_sig;
            let mut reload_c = reload_sig;

            spawn(async move {
                match FinanceApi::create_transaction(req).await {
                    Ok(_) => {
                        toast_resp.show("Lançamento financeiro registrado!", ToastVariant::Success);
                        modal_c.set(false);
                        reload_c.set(reload_c() + 1);
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let handle_confirm_payment = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut pay_modal_sig = is_payment_modal_open;
        let mut reload_sig = reload_trigger;

        move |(tid, amount_cents, method, p_date): (String, i64, String, String)| {
            let req = RegisterPaymentRequest {
                clinic_id: cid.clone(),
                amount_cents,
                payment_method: method,
                paid_date: Some(p_date),
                notes: Some("Pagamento registrado pelo operador".to_string()),
            };

            let mut toast_resp = toast_c.clone();
            let mut pmodal_c = pay_modal_sig;
            let mut reload_c = reload_sig;

            spawn(async move {
                match FinanceApi::register_payment(&tid, req).await {
                    Ok(updated) => {
                        if updated.status == TransactionStatus::Paid {
                            toast_resp.show("Pagamento integral registrado!", ToastVariant::Success);
                        } else {
                            toast_resp.show("Pagamento parcial registrado com sucesso!", ToastVariant::Success);
                        }
                        pmodal_c.set(false);
                        reload_c.set(reload_c() + 1);
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let tx_list = transactions.read().clone();

    let received_cents: i64 = tx_list
        .iter()
        .filter(|t| t.direction == TransactionDirection::Income && (t.status == TransactionStatus::Paid || t.status == TransactionStatus::Partial))
        .map(|t| t.paid_amount_cents)
        .sum();

    let pending_income_cents: i64 = tx_list
        .iter()
        .filter(|t| t.direction == TransactionDirection::Income && t.status != TransactionStatus::Paid)
        .map(|t| if t.remaining_amount_cents > 0 { t.remaining_amount_cents } else { t.amount_cents })
        .sum();

    let paid_expense_cents: i64 = tx_list
        .iter()
        .filter(|t| t.direction == TransactionDirection::Expense && (t.status == TransactionStatus::Paid || t.status == TransactionStatus::Partial))
        .map(|t| t.paid_amount_cents)
        .sum();

    let pending_expense_cents: i64 = tx_list
        .iter()
        .filter(|t| t.direction == TransactionDirection::Expense && t.status != TransactionStatus::Paid)
        .map(|t| if t.remaining_amount_cents > 0 { t.remaining_amount_cents } else { t.amount_cents })
        .sum();

    let filtered_transactions: Vec<Transaction> = tx_list.into_iter().filter(|t| {
        let is_inc = t.direction == TransactionDirection::Income;
        let is_pd = t.status == TransactionStatus::Paid;

        if is_inc && !*filter_income.read() { return false; }
        if !is_inc && !*filter_expense.read() { return false; }
        if is_pd && !*filter_paid.read() { return false; }
        if !is_pd && !*filter_unpaid.read() { return false; }

        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        t.description.to_lowercase().contains(&q)
            || t.category.to_lowercase().contains(&q)
            || t.patient_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
    }).collect();

    let selected_tx = selected_tx_id.read().as_ref().and_then(|tid| {
        transactions.read().iter().find(|t| t.id == *tid).cloned()
    });

    let mut toast_del = toast.clone();
    let mut toast_sav = toast.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "finance-page",
            FinanceToolbar {
                period_filter,
                start_date,
                end_date,
                search_query,
                on_open_filter_modal: move |_| is_filter_modal_open.set(true),
                on_new_transaction: move |type_str: String| {
                    if type_str == "expense" {
                        add_direction.set(TransactionDirection::Expense);
                        category.set("Materiais & Insumos".to_string());
                    } else {
                        add_direction.set(TransactionDirection::Income);
                        category.set("Tratamento Odontológico".to_string());
                    }
                    desc.set(String::new());
                    amount_str.set("150.00".to_string());
                    is_add_modal_open.set(true);
                },
            }

            FinanceEquationSummary {
                received_cents,
                pending_income_cents,
                paid_expense_cents,
                pending_expense_cents,
            }

            FinanceTable {
                transactions: filtered_transactions,
                on_open_payment_modal: move |tid| {
                    selected_tx_id.set(Some(tid));
                    is_payment_modal_open.set(true);
                },
                on_open_details_modal: move |tid| {
                    selected_tx_id.set(Some(tid));
                    is_details_modal_open.set(true);
                },
                on_delete_transaction: move |_tid| {
                    toast_del.show("Lançamento excluído com sucesso.", ToastVariant::Success);
                },
            }

            FinanceFilterModal {
                is_open: is_filter_modal_open(),
                filter_income,
                filter_unlinked,
                filter_expense,
                filter_paid,
                filter_unpaid,
                filter_scheduled,
                account_filter,
                payment_method_filter,
                on_close: move |_| is_filter_modal_open.set(false),
                on_apply: move |_| is_filter_modal_open.set(false),
            }

            ModalTransaction {
                is_open: is_add_modal_open(),
                direction: add_direction(),
                description: desc,
                amount_str,
                category,
                payment_method,
                is_paid,
                due_date,
                on_close: move |_| is_add_modal_open.set(false),
                on_submit: handle_submit,
            }

            ModalPayment {
                is_open: is_payment_modal_open(),
                transaction: selected_tx.clone(),
                on_close: move |_| is_payment_modal_open.set(false),
                on_confirm_payment: handle_confirm_payment,
            }

            ModalTransactionDetails {
                is_open: is_details_modal_open(),
                transaction: selected_tx,
                on_close: move |_| is_details_modal_open.set(false),
                on_save: move |_| {
                    is_details_modal_open.set(false);
                    toast_sav.show("Alterações salvas!", ToastVariant::Success);
                },
            }
        }
    }
}
