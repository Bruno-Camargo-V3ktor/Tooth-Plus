use crate::icons::{IconCheck, IconDollar, IconExternalLink, IconFileText};
use shared::finance::{Transaction, TransactionDirection, TransactionStatus};
use dioxus::prelude::*;

#[component]
pub fn FinanceTable(
    transactions: Vec<Transaction>,
    on_open_payment_modal: EventHandler<String>,
    on_open_details_modal: EventHandler<String>,
    on_delete_transaction: EventHandler<String>,
) -> Element {
    let mut open_context_menu_id = use_signal(|| None::<String>);

    if transactions.is_empty() {
        return rsx! {
            div { class: "empty-debits-box",
                div { class: "empty-debits-icon",
                    IconDollar { size: 48, color: "#475569".to_string() }
                }
                h3 { class: "empty-debits-title", "Sem resultados para o período" }
                p { class: "empty-debits-desc", "Tente alterar os filtros de data ou categoria." }
            }
        };
    }

    rsx! {
        div { class: "patients-table-card",
            table { class: "patients-list-table",
                thead {
                    tr {
                        th { style: "width: 32px;", input { r#type: "checkbox" } }
                        th { style: "width: 24px;" }
                        th { "Data ⇣" }
                        th { "Nome" }
                        th { style: "text-align: right; padding-right: 24px;", "Valor" }
                        th { style: "width: 32px;" }
                    }
                }
                tbody {
                    for tx in transactions {
                        {
                            let tid = tx.id.clone();
                            let tid_pay = tx.id.clone();
                            let tid_ctx = tx.id.clone();
                            let tid_del = tx.id.clone();
                            let tid_docs = tx.id.clone();

                            let is_income = tx.direction == TransactionDirection::Income;
                            let is_paid = tx.status == TransactionStatus::Paid;
                            let is_partial = tx.status == TransactionStatus::Partial;
                            let date_clean = tx.due_date.split('T').next().unwrap_or(&tx.due_date);

                            let val_color = if is_paid { "#22c55e" } else { "#ef4444" };
                            let val_str = format!("R$ {:.2}", tx.amount_cents as f64 / 100.0);
                            let action_label = if is_income { "RECEBER" } else { "PAGAR" };

                            let is_ctx_open = open_context_menu_id.read().as_ref() == Some(&tid);

                            rsx! {
                                tr { key: "{tx.id}", class: "patient-table-row",
                                    td { input { r#type: "checkbox" } }
                                    td {
                                        if is_income {
                                            span { style: "color: #22c55e; font-weight: 800; font-size: 15px;", "↙" }
                                        } else {
                                            span { style: "color: #ef4444; font-weight: 800; font-size: 15px;", "↗" }
                                        }
                                    }
                                    td { "{date_clean}" }
                                    td {
                                        div { style: "display: flex; align-items: center; gap: 8px; flex-wrap: wrap;",
                                            span {
                                                class: "patient-name-link",
                                                style: if is_paid { "color: #f1f5f9;" } else { "color: #f87171;" },
                                                "{tx.description}"
                                            }
                                            span { class: "action-btn-icon", style: "width: 18px; height: 18px;",
                                                IconExternalLink { size: 12, color: "#00a0e4".to_string() }
                                            }
                                            if let Some(cpf) = tx.patient_id.as_ref() {
                                                span { style: "font-size: 12px; color: #64748b;", "196.550.148-60" }
                                            }
                                            if tx.has_receipt {
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    style: "width: 22px; height: 22px;",
                                                    title: "Ver recibo / comprovante",
                                                    onclick: move |_| on_open_details_modal.call(tid_docs.clone()),
                                                    IconFileText { size: 14, color: "#94a3b8".to_string() }
                                                }
                                            }
                                            if is_paid {
                                                span { class: "badge badge-gray", style: "font-size: 11px; padding: 1px 6px;", "Débito" }
                                            }
                                            if is_partial {
                                                span { class: "badge badge-blue", style: "font-size: 11px; padding: 1px 6px;",
                                                    "Pago: R$ {tx.paid_amount_cents as f64 / 100.0:.2}"
                                                }
                                            }
                                        }
                                    }
                                    td {
                                        div { style: "display: flex; align-items: center; justify-content: flex-end; gap: 12px; position: relative;",
                                            span { style: "font-weight: 800; font-size: 14.5px; color: {val_color};", "{val_str}" }

                                            if is_paid {
                                                div { style: "width: 22px; height: 22px; border-radius: 50%; background: rgba(34,197,94,0.15); display: flex; align-items: center; justify-content: center; color: #22c55e;",
                                                    IconCheck { size: 14, color: "#22c55e".to_string() }
                                                }
                                            } else {
                                                button {
                                                    r#type: "button",
                                                    class: "btn-secondary",
                                                    style: "color: #38bdf8; border-color: rgba(56,189,248,0.4); font-size: 11.5px; font-weight: 800; padding: 4px 10px;",
                                                    onclick: move |_| on_open_payment_modal.call(tid_pay.clone()),
                                                    "{action_label}"
                                                }
                                            }
                                        }
                                    }
                                    td {
                                        div { style: "position: relative; text-align: center;",
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                onclick: move |_| {
                                                    if is_ctx_open {
                                                        open_context_menu_id.set(None);
                                                    } else {
                                                        open_context_menu_id.set(Some(tid_ctx.clone()));
                                                    }
                                                },
                                                "⋮"
                                            }

                                            if is_ctx_open {
                                                div { class: "finance-add-menu", style: "right: 0; top: 28px;",
                                                    button {
                                                        r#type: "button",
                                                        class: "finance-menu-item",
                                                        onclick: move |_| {
                                                            open_context_menu_id.set(None);
                                                            on_open_details_modal.call(tid.clone());
                                                        },
                                                        "Editar"
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "finance-menu-item",
                                                        "Emitir nota fiscal"
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "finance-menu-item menu-item-expense",
                                                        onclick: move |_| {
                                                            open_context_menu_id.set(None);
                                                            on_delete_transaction.call(tid_del.clone());
                                                        },
                                                        "Excluir"
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "finance-menu-item",
                                                        style: "color: #22c55e;",
                                                        "Conversar no WhatsApp Web"
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
}
