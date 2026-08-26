use shared::finance::{Transaction, TransactionDirection, TransactionStatus};
use dioxus::prelude::*;

#[component]
pub fn FinanceTable(transactions: Vec<Transaction>) -> Element {
    rsx! {
        div { class: "finance-table-container",
            table { class: "finance-table",
                thead {
                    tr {
                        th { "Data / Vencimento" }
                        th { "Descrição / Paciente" }
                        th { "Categoria" }
                        th { "Forma de Pagamento" }
                        th { "Valor" }
                        th { "Status" }
                    }
                }
                tbody {
                    for tx in transactions {
                        {
                            let is_income = tx.direction == TransactionDirection::Income;
                            let amt_str = format!("R$ {:.2}", tx.amount_cents as f64 / 100.0);
                            let date_fmt = tx.due_date.split('T').next().unwrap_or(&tx.due_date).to_string();
                            let is_paid = tx.status == TransactionStatus::Paid;
                            let status_label = if is_paid { "Pago / Recebido" } else { "Pendente" };
                            let pm_display = tx.payment_method.as_deref().unwrap_or("-").to_string();
                            let desc_display = tx.description.clone();
                            let cat_display = tx.category.clone();

                            rsx! {
                                tr {
                                    key: "{tx.id}",
                                    td { "{date_fmt}" }
                                    td {
                                        strong { style: "color: #f1f5f9;", "{desc_display}" }
                                    }
                                    td { "{cat_display}" }
                                    td { "{pm_display}" }
                                    td {
                                        span {
                                            class: if is_income { "amount-income" } else { "amount-expense" },
                                            if is_income { "+ " } else { "- " }
                                            "{amt_str}"
                                        }
                                    }
                                    td {
                                        span {
                                            class: if is_paid { "badge-paid" } else { "badge-pending" },
                                            "{status_label}"
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
