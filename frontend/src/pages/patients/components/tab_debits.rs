use crate::icons::{IconDollar, IconPlus};
use shared::finance::Transaction;
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn TabDebits(
    patient: Patient,
    transactions: Vec<Transaction>,
    on_new_debit: EventHandler<()>,
) -> Element {
    let mut show_received = use_signal(|| false);
    let mut period_filter = use_signal(|| "all".to_string());

    let patient_tx: Vec<Transaction> = transactions
        .into_iter()
        .filter(|t| t.patient_id.as_deref() == Some(&patient.id) || t.patient_name.as_deref() == Some(&patient.full_name))
        .collect();

    let total_received_cents: i64 = patient_tx
        .iter()
        .filter(|t| t.status == shared::finance::TransactionStatus::Paid)
        .map(|t| t.paid_amount_cents.max(t.amount_cents))
        .sum();

    let total_pending_cents: i64 = patient_tx
        .iter()
        .filter(|t| t.status != shared::finance::TransactionStatus::Paid)
        .map(|t| t.remaining_amount_cents.max(t.amount_cents))
        .sum();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            // Top Toolbar da Aba Débitos
            div { class: "debits-top-toolbar",
                div { style: "display: flex; align-items: center; gap: 24px;",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        label { class: "switch-input-custom",
                            input {
                                r#type: "checkbox",
                                checked: "{show_received}",
                                onchange: move |e| show_received.set(e.checked()),
                            }
                            span { class: "switch-slider" }
                        }
                        span { style: "font-size: 13.5px; color: #cbd5e1; font-weight: 500;", "Mostrar recebidos" }
                    }

                    div { style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 13px; color: #94a3b8;", "Exibindo débitos" }
                        select {
                            class: "form-select",
                            style: "height: 36px; padding: 0 10px; font-size: 13px;",
                            value: "{period_filter}",
                            onchange: move |e| period_filter.set(e.value()),
                            option { value: "all", "de todos os períodos" }
                            option { value: "month", "deste mês" }
                            option { value: "year", "deste ano" }
                        }
                    }
                }

                div { style: "display: flex; align-items: center; gap: 10px;",
                    button {
                        r#type: "button",
                        class: "btn-export",
                        style: "height: 38px;",
                        "📊 Relatórios ▾"
                    }
                    button {
                        r#type: "button",
                        class: "btn-new-patient-green",
                        style: "height: 38px;",
                        onclick: move |_| on_new_debit.call(()),
                        IconPlus { size: 15, color: "#ffffff".to_string() }
                        span { "NOVO DÉBITO" }
                    }
                }
            }

            // Barra de Totais (Recebido vs A Receber)
            div { class: "debits-summary-bars",
                div { class: "debit-total-box",
                    span { class: "debit-total-label", "TOTAL RECEBIDO" }
                    span { class: "debit-total-val-received", "R$ {total_received_cents as f64 / 100.0:.2}" }
                }
                div { class: "debit-total-box",
                    span { class: "debit-total-label", "TOTAL A RECEBER" }
                    span { class: "debit-total-val-pending", "R$ {total_pending_cents as f64 / 100.0:.2}" }
                }
            }

            // Tabela / Empty State
            if patient_tx.is_empty() {
                div { class: "empty-debits-box",
                    div { class: "empty-debits-icon",
                        IconDollar { size: 48, color: "#475569".to_string() }
                    }
                    h3 { class: "empty-debits-title", "Sem débitos" }
                    p { class: "empty-debits-desc", "Esse paciente não possui débitos cadastrados." }
                }
            } else {
                div { class: "patients-table-card",
                    table { class: "patients-list-table",
                        thead {
                            tr {
                                th { "Data ⇡" }
                                th { "Nome / Procedimento" }
                                th { "Status" }
                                th { style: "text-align: right;", "Valor" }
                            }
                        }
                        tbody {
                            for tx in patient_tx {
                                tr {
                                    key: "{tx.id}",
                                    td { "{tx.due_date.split('T').next().unwrap_or(&tx.due_date)}" }
                                    td {
                                        strong { style: "color: #f1f5f9;", "{tx.description}" }
                                    }
                                    td {
                                        span {
                                            class: if tx.status == shared::finance::TransactionStatus::Paid { "badge-stock-ok" } else { "badge-stock-low" },
                                            "{tx.status.label()}"
                                        }
                                    }
                                    td { style: "text-align: right; font-weight: 700; color: #f8fafc;", "R$ {tx.amount_cents as f64 / 100.0:.2}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
