use crate::icons::{IconDollar, IconFileText, IconTrash};
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
                        th { "Forma Pagto" }
                        th { "Valor" }
                        th { "Status" }
                        th { style: "text-align: right; padding-right: 20px;", "Ações" }
                    }
                }
                tbody {
                    for tx in transactions {
                        {
                            let tx_id = tx.id.clone();
                            let tx_id_det = tx.id.clone();
                            let tx_id_del = tx.id.clone();
                            let is_income = tx.direction == TransactionDirection::Income;
                            let is_pending = tx.status == TransactionStatus::Pending || tx.status == TransactionStatus::Partial;
                            let p_name = tx.patient_name.clone().unwrap_or_else(|| "Clínica / Geral".to_string());
                            let p_id_opt = tx.patient_id.clone();

                            let val_formatted = format!("R$ {:.2}", (tx.amount_cents as f64) / 100.0);
                            let cat_name = tx.category.clone();
                            let method_name = tx.payment_method.clone().unwrap_or_else(|| "PIX".to_string()).to_uppercase();

                            let badge_cls = match tx.status {
                                TransactionStatus::Paid => "badge badge-green",
                                TransactionStatus::Partial => "badge badge-yellow",
                                TransactionStatus::Pending => "badge badge-yellow",
                                TransactionStatus::Canceled | TransactionStatus::Refunded => "badge badge-gray",
                            };

                            let badge_label = match tx.status {
                                TransactionStatus::Paid => "Pago",
                                TransactionStatus::Partial => "Parcial",
                                TransactionStatus::Pending => "Aberto",
                                TransactionStatus::Canceled => "Cancelado",
                                TransactionStatus::Refunded => "Estornado",
                            };

                            rsx! {
                                tr { key: "{tx.id}", class: "patient-table-row",
                                    td { "{tx.due_date}" }
                                    td {
                                        strong { style: "color: #f1f5f9;", "{tx.description}" }
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
                                    td { "{cat_name}" }
                                    td { "{method_name}" }
                                    td {
                                        span {
                                            style: if is_income { "font-weight: 700; color: #22c55e;" } else { "font-weight: 700; color: #ef4444;" },
                                            if is_income { "+ " } else { "- " }
                                            "{val_formatted}"
                                        }
                                    }
                                    td {
                                        span { class: "{badge_cls}", "{badge_label}" }
                                    }
                                    td {
                                        div { style: "display: flex; align-items: center; justify-content: flex-end; gap: 6px;",
                                            if is_pending {
                                                button {
                                                    r#type: "button",
                                                    class: "btn-pay-action-blue",
                                                    onclick: move |_| on_open_payment_modal.call(tx_id.clone()),
                                                    if is_income { "RECEBER" } else { "PAGAR" }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Ver Detalhes / Comprovantes",
                                                onclick: move |_| on_open_details_modal.call(tx_id_det.clone()),
                                                IconFileText { size: 15, color: "#94a3b8".to_string() }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Excluir Lançamento",
                                                onclick: move |_| on_delete_transaction.call(tx_id_del.clone()),
                                                IconTrash { size: 15, color: "#ef4444".to_string() }
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
