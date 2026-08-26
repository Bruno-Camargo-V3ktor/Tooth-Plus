use crate::icons::IconDollar;
use shared::finance::{Transaction, TransactionDirection, TransactionStatus};
use dioxus::prelude::*;

#[component]
pub fn FinanceTable(transactions: Vec<Transaction>) -> Element {
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
                        th { "Data ⇣" }
                        th { "Nome / Descrição" }
                        th { "Categoria" }
                        th { "Status" }
                        th { style: "text-align: right;", "Valor" }
                    }
                }
                tbody {
                    for tx in transactions {
                        {
                            let is_income = tx.direction == TransactionDirection::Income;
                            let is_paid = tx.status == TransactionStatus::Paid;
                            let amount_fmt = format!("R$ {:.2}", tx.amount_cents as f64 / 100.0);
                            let val_color = if is_income { "#22c55e" } else { "#ef4444" };
                            let val_prefix = if is_income { "+ " } else { "- " };
                            let date_clean = tx.due_date.split('T').next().unwrap_or(&tx.due_date);

                            let badge_text = match (is_income, is_paid) {
                                (true, true) => "Recebida",
                                (true, false) => "A Receber",
                                (false, true) => "Paga",
                                (false, false) => "A Pagar",
                            };

                            let badge_cls = match (is_income, is_paid) {
                                (_, true) => "badge badge-green",
                                (true, false) => "badge badge-blue",
                                (false, false) => "badge badge-red",
                            };

                            rsx! {
                                tr { key: "{tx.id}", class: "patient-table-row",
                                    td { "{date_clean}" }
                                    td {
                                        strong { style: "color: #f1f5f9; font-size: 13.5px;", "{tx.description}" }
                                        if let Some(ref p_name) = tx.patient_name {
                                            span { style: "display: block; font-size: 11.5px; color: #94a3b8;", "Paciente: {p_name}" }
                                        }
                                    }
                                    td { "{tx.category}" }
                                    td {
                                        span { class: "{badge_cls}", "{badge_text}" }
                                    }
                                    td { style: "text-align: right; font-weight: 800; font-size: 14px; color: {val_color};",
                                        "{val_prefix}{amount_fmt}"
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
