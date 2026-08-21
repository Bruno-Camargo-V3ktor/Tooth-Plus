//! # Listagem de Transações e Lançamentos Financeiros (Frontend)
//!
//! Exibe os lançamentos de receitas e despesas em cards modernos com badges
//! de categoria em português, status de liquidação e ações contextuais.

use crate::components::icons::{IconCheck, IconClock, IconFinance, IconTrash};
use dioxus::prelude::*;
use shared::finance::{Transaction, TransactionDirection, TransactionStatus};

/// Formata valor em centavos para moeda BRL.
fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;
    if is_negative {
        format!("- R$ {}.{:02}", reals, centavos)
    } else {
        format!("R$ {}.{:02}", reals, centavos)
    }
}

/// Normaliza e traduz nomes de categorias para exibição amigável em português.
pub fn format_category_display(cat: &str) -> String {
    match cat.trim().to_lowercase().as_str() {
        "consultation" => "Procedimento Clínico".to_string(),
        "treatment" => "Tratamento Odontológico".to_string(),
        "surgery" => "Cirurgia".to_string(),
        "return" => "Retorno".to_string(),
        "supplies" => "Insumos & Estoque".to_string(),
        "rent" => "Custos Fixos / Aluguel".to_string(),
        "utilities" => "Água / Luz / Internet".to_string(),
        "commission" => "Salários & Repasses".to_string(),
        "maintenance" => "Manutenção & Equipamentos".to_string(),
        "other_income" => "Outra Receita".to_string(),
        "other_expense" => "Outra Despesa".to_string(),
        _ => {
            if cat.is_empty() {
                "Geral".to_string()
            } else {
                cat.to_string()
            }
        }
    }
}

/// Seção da listagem de transações com cards modernos e ações de liquidação e remoção.
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
                div { class: "empty-state-icon-box",
                    IconFinance { size: 32, color: "#64748b".to_string() }
                }
                h3 { "Nenhuma movimentação encontrada" }
                p { "Não há lançamentos financeiros registrados para este período. Altere os filtros de data ou registre uma nova entrada/saída." }
            }
        };
    }

    rsx! {
        div { class: "finance-list-container",
            for tx in &transactions {
                {
                    let tx_clone = tx.clone();
                    let tx_clone_del = tx.clone();
                    let is_income = tx.direction == TransactionDirection::Income;
                    let is_pending = tx.status == TransactionStatus::Pending;
                    let is_partial = tx.status == TransactionStatus::Partial;
                    let can_settle_now = is_pending || is_partial;
                    let val_prefix = if is_income { "+ " } else { "- " };
                    let amount_color_cls = if is_income { "text-success" } else { "text-danger" };

                    let status_cls = match tx.status {
                        TransactionStatus::Paid => "status-paid",
                        TransactionStatus::Partial => "status-pending",
                        TransactionStatus::Pending => "status-pending",
                        TransactionStatus::Canceled => "status-canceled",
                        TransactionStatus::Refunded => "status-refunded",
                    };
                    let status_label = match tx.status {
                        TransactionStatus::Paid => "Liquidado",
                        TransactionStatus::Partial => "Parcialmente Pago",
                        TransactionStatus::Pending => "Não Pago / Pendente",
                        TransactionStatus::Canceled => "Cancelado",
                        TransactionStatus::Refunded => "Estornado",
                    };

                    let date_display = if !tx.due_date.is_empty() {
                        if let Ok(ndt) = chrono::NaiveDate::parse_from_str(
                            tx.due_date.chars().take(10).collect::<String>().as_str(),
                            "%Y-%m-%d",
                        ) {
                            ndt.format("%d/%m/%Y").to_string()
                        } else {
                            tx.due_date.chars().take(10).collect::<String>()
                        }
                    } else {
                        "-".to_string()
                    };

                    let category_label = format_category_display(&tx.category);

                    rsx! {
                        div { key: "{tx.id}", class: "finance-item-card",
                            div { class: "finance-card-left",
                                div {
                                    class: if is_pending {
                                        "fin-dir-indicator dir-pending"
                                    } else if is_income {
                                        "fin-dir-indicator dir-income"
                                    } else {
                                        "fin-dir-indicator dir-expense"
                                    },
                                    if is_pending {
                                        IconClock { size: 18, color: "currentColor".to_string() }
                                    } else if is_income {
                                        span { "↓" }
                                    } else {
                                        span { "↑" }
                                    }
                                }
                                div { class: "fin-card-info",
                                    div { class: "fin-card-title-row",
                                        span { class: "fin-tx-description", "{tx.description}" }
                                        span { class: "fin-category-badge", "{category_label}" }
                                        if tx.appointment_id.is_some() || tx.is_calculated_pending {
                                            span { class: "fin-simulated-badge", "Agenda Automática" }
                                        }
                                        if tx.treatment_plan_id.is_some() {
                                            span { class: "fin-treatment-plan-badge", "🦷 Orçamento Clínico" }
                                        }
                                    }
                                    div { class: "fin-card-meta-row",
                                        span { class: "fin-meta-item", "Vencimento: {date_display}" }
                                        if let Some(ref p_name) = tx.patient_name {
                                            span { class: "fin-meta-item", "Paciente: {p_name}" }
                                        }
                                        if let Some(ref method) = tx.payment_method {
                                            span { class: "fin-meta-item fin-meta-method", "{method}" }
                                        }
                                    }
                                }
                            }

                            div { class: "finance-card-right",
                                div { class: "fin-amount-col",
                                    span { class: "fin-amount-text {amount_color_cls}",
                                        "{val_prefix}{format_currency(tx.amount_cents)}"
                                    }
                                    span { class: "fin-status-pill {status_cls}", "{status_label}" }
                                }
                                div { class: "fin-card-actions",
                                    if can_settle_now && can_update_status {
                                        button {
                                            class: "btn-liquidar-action",
                                            title: "Registrar pagamento / liquidação",
                                            onclick: move |_| on_settle.call(tx_clone.clone()),
                                            IconCheck { size: 14, color: "currentColor".to_string() }
                                            span { if is_partial { " Pagar Saldo" } else { " Liquidar" } }
                                        }
                                    }
                                    if can_delete {
                                        button {
                                            class: "item-action-icon-btn btn-danger-icon",
                                            title: "Excluir Lançamento Financeiro",
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
