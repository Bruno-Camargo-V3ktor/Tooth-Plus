use crate::icons::{IconCheck, IconDollar, IconTrash};
use crate::router::Route;
use shared::finance::{Transaction, TransactionDirection, TransactionStatus};
use dioxus::prelude::*;

#[component]
pub fn FinanceTable(
    transactions: Vec<Transaction>,
    on_open_payment_modal: EventHandler<String>,
    on_open_details_modal: EventHandler<String>,
    on_delete_transaction: EventHandler<String>,
) -> Element {
    let mut selected_patient_id = consume_context::<Signal<Option<String>>>();
    let mut open_context_menu_id = use_signal(|| None::<String>);
    let nav = navigator();

    if transactions.is_empty() {
        return rsx! {
            div { class: "empty-debits-box",
                div { class: "empty-debits-icon",
                    IconDollar { size: 48, color: "#475569".to_string() }
                }
                h3 { class: "empty-debits-title", "Nenhuma movimentação no período" }
                p { class: "empty-debits-desc", "Utilize a barra de ferramentas para registrar recebimentos ou despesas da clínica." }
            }
        };
    }

    rsx! {
        div { class: "patients-table-card",
            table { class: "patients-list-table",
                thead {
                    tr {
                        th { "Vencimento ⇣" }
                        th { "Descrição" }
                        th { "Paciente / Fornecedor" }
                        th { "Categoria" }
                        th { style: "text-align: right; padding-right: 24px;", "Valor" }
                        th { style: "text-align: center; width: 48px;", "" }
                    }
                }
                tbody {
                    for tx in transactions {
                        {
                            let tx_id = tx.id.clone();
                            let tx_id_ctx = tx.id.clone();
                            let tx_id_det = tx.id.clone();
                            let tx_id_del = tx.id.clone();
                            let is_income = tx.direction == TransactionDirection::Income;
                            let is_paid = tx.status == TransactionStatus::Paid;
                            let p_name = tx.patient_name.clone().unwrap_or_else(|| "Clínica / Geral".to_string());
                            let p_id_opt = tx.patient_id.clone();

                            let val_formatted = format!("R$ {:.2}", (tx.amount_cents as f64) / 100.0);
                            let cat_name = tx.category.clone();
                            let is_ctx_open = open_context_menu_id.read().as_ref() == Some(&tx.id);

                            let action_label = if is_income { "RECEBER" } else { "PAGAR" };
                            let val_color = if is_income { "#22c55e" } else { "#ef4444" };
                            let val_prefix = if is_income { "+ " } else { "- " };

                            rsx! {
                                tr { key: "{tx.id}", class: "patient-table-row",
                                    td { style: "color: #94a3b8; font-size: 13px;", "{tx.due_date}" }
                                    td {
                                        div {
                                            strong { style: "color: #f1f5f9; font-size: 13.5px; display: block;", "{tx.description}" }
                                            if let Some(ref pm) = tx.payment_method {
                                                span { style: "font-size: 11.5px; color: #64748b;", "{pm.to_uppercase()}" }
                                            }
                                        }
                                    }
                                    td {
                                        span {
                                            class: "patient-name-link",
                                            onclick: move |_| {
                                                if let Some(ref pid) = p_id_opt {
                                                    selected_patient_id.set(Some(pid.clone()));
                                                    nav.push(Route::PatientsView {});
                                                }
                                            },
                                            "{p_name}"
                                        }
                                    }
                                    td {
                                        span { style: "font-size: 12.5px; color: #94a3b8;", "{cat_name}" }
                                    }
                                    td { style: "text-align: right;",
                                        div { style: "display: inline-flex; align-items: center; justify-content: flex-end; gap: 12px;",
                                            span { style: "font-weight: 800; font-size: 14.5px; color: {val_color};",
                                                "{val_prefix}{val_formatted}"
                                            }

                                            if is_paid {
                                                div { style: "width: 22px; height: 22px; border-radius: 50%; background: rgba(34,197,94,0.15); display: flex; align-items: center; justify-content: center; color: #22c55e;",
                                                    IconCheck { size: 14, color: "#22c55e".to_string() }
                                                }
                                            } else {
                                                button {
                                                    r#type: "button",
                                                    class: "btn-secondary",
                                                    style: "color: #38bdf8; border-color: rgba(56,189,248,0.4); font-size: 11.5px; font-weight: 800; padding: 4px 10px;",
                                                    onclick: move |_| on_open_payment_modal.call(tx_id.clone()),
                                                    "{action_label}"
                                                }
                                            }
                                        }
                                    }
                                    td { style: "text-align: center; position: relative;",
                                        button {
                                            r#type: "button",
                                            class: "action-btn-icon",
                                            onclick: move |_| {
                                                if is_ctx_open {
                                                    open_context_menu_id.set(None);
                                                } else {
                                                    open_context_menu_id.set(Some(tx_id_ctx.clone()));
                                                }
                                            },
                                            "⋮"
                                        }

                                        if is_ctx_open {
                                            div { class: "finance-add-menu", style: "right: 0; top: 28px; z-index: 50;",
                                                button {
                                                    r#type: "button",
                                                    class: "finance-menu-item",
                                                    onclick: move |_| {
                                                        open_context_menu_id.set(None);
                                                        on_open_details_modal.call(tx_id_det.clone());
                                                    },
                                                    "Editar lançamento"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "finance-menu-item menu-item-expense",
                                                    onclick: move |_| {
                                                        open_context_menu_id.set(None);
                                                        on_delete_transaction.call(tx_id_del.clone());
                                                    },
                                                    "Excluir"
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
