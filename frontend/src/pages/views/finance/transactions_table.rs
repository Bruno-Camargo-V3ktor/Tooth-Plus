//! # Tabela de Transações e Resumo Financeiro (Frontend)
//!
//! Exibe a listagem de lançamentos de receitas e despesas, indicadores de saldo líquido,
//! liquidação de contas a pagar/receber e exclusão de lançamentos.

use crate::components::icons::{IconFinance, IconTrash};
use dioxus::prelude::*;
use shared::finance::{Transaction, TransactionDirection, TransactionStatus};

/// Formata valor em centavos para moeda BRL.
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

/// Seção da tabela de transações com ações de liquidação e remoção.
#[component]
pub fn TransactionsTableSection(
    transactions: Vec<Transaction>,
    can_update_status: bool,
    can_delete: bool,
    on_settle: EventHandler<Transaction>,
    on_delete: EventHandler<Transaction>,
) -> Element {
    if transactions.is_empty() {
        return rsx! {
            div { class: "empty-state-card",
                IconFinance { size: 48, color: "var(--text-muted, #8c8c8c)".to_string() }
                h3 { "Nenhum lançamento financeiro localizado" }
                p { "Utilize os filtros acima ou crie uma nova receita/despesa." }
            }
        };
    }

    rsx! {
        div { class: "table-responsive",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Data" }
                        th { "Descrição / Paciente" }
                        th { "Categoria" }
                        th { "Forma de Pagto" }
                        th { "Status" }
                        th { "Valor" }
                        th { "Ações" }
                    }
                }
                tbody {
                    for tx in &transactions {
                        {
                            let tx_clone = tx.clone();
                            let tx_clone_del = tx.clone();
                            let is_income = tx.direction == TransactionDirection::Income;
                            let val_badge = if is_income { "text-success font-mono font-weight-bold" } else { "text-danger font-mono font-weight-bold" };
                            let val_prefix = if is_income { "+" } else { "-" };

                            let status_badge = match tx.status {
                                TransactionStatus::Paid => "badge-success",
                                TransactionStatus::Pending => "badge-warning",
                                TransactionStatus::Canceled => "badge-danger",
                                TransactionStatus::Refunded => "badge-outline",
                            };
                            let status_label = match tx.status {
                                TransactionStatus::Paid => "Liquidado",
                                TransactionStatus::Pending => "Pendente",
                                TransactionStatus::Canceled => "Cancelado",
                                TransactionStatus::Refunded => "Estornado",
                            };

                            let date_display = if !tx.due_date.is_empty() {
                                tx.due_date.chars().take(10).collect::<String>()
                            } else {
                                "-".to_string()
                            };

                            rsx! {
                                tr { key: "{tx.id}",
                                    td { class: "font-mono font-xs",
                                        "{date_display}"
                                    }
                                    td {
                                        strong { "{tx.description}" }
                                        if let Some(ref p_name) = tx.patient_name {
                                            div { class: "text-muted font-xs", "Paciente: {p_name}" }
                                        }
                                    }
                                    td {
                                        span { class: "badge-outline", "{tx.category}" }
                                    }
                                    td {
                                        span { class: "font-xs", "{tx.payment_method.as_deref().unwrap_or(\"-\")}" }
                                    }
                                    td {
                                        span { class: "{status_badge}", "{status_label}" }
                                    }
                                    td {
                                        span { class: "{val_badge}",
                                            "{val_prefix} {format_currency(tx.amount_cents)}"
                                        }
                                    }
                                    td { class: "actions-cell",
                                        if tx.status == TransactionStatus::Pending && can_update_status {
                                            button {
                                                class: "btn-secondary btn-sm",
                                                title: "Liquidar / Confirmar Pagamento",
                                                onclick: move |_| on_settle.call(tx_clone.clone()),
                                                "Liquidar"
                                            }
                                        }
                                        if can_delete {
                                            button {
                                                class: "btn-icon text-danger",
                                                title: "Excluir Lançamento",
                                                onclick: move |_| on_delete.call(tx_clone_del.clone()),
                                                IconTrash { size: 14, color: "currentColor".to_string() }
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
