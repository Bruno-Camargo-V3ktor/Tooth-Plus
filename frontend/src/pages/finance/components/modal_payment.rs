use crate::components::modal::Modal;
use shared::finance::Transaction;
use dioxus::prelude::*;

#[component]
pub fn ModalPayment(
    is_open: bool,
    transaction: Option<Transaction>,
    on_close: EventHandler<()>,
    on_confirm_payment: EventHandler<(String, i64, String, String)>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let tx = match transaction {
        Some(t) => t,
        None => return rsx! {},
    };

    let remaining_cents = if tx.remaining_amount_cents > 0 {
        tx.remaining_amount_cents
    } else {
        tx.amount_cents - tx.paid_amount_cents
    };

    let default_val_str = format!("{:.2}", remaining_cents as f64 / 100.0);
    let mut payment_amount_str = use_signal(|| default_val_str);
    let mut payment_method = use_signal(|| "pix".to_string());
    let mut payment_date = use_signal(|| "2026-08-26".to_string());
    let mut notes = use_signal(String::new);

    let tid = tx.id.clone();
    let is_income = tx.direction == shared::finance::TransactionDirection::Income;
    let action_title = if is_income { "Registrar Recebimento" } else { "Registrar Pagamento de Despesa" };

    rsx! {
        Modal {
            title: action_title.to_string(),
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                button {
                    r#type: "button",
                    class: "btn-modal-ghost",
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                button {
                    r#type: "button",
                    class: "btn-modal-primary",
                    onclick: move |_| {
                        let amount_num: f64 = payment_amount_str.read().replace(',', ".").parse().unwrap_or(0.0);
                        let amount_cents = (amount_num * 100.0) as i64;
                        on_confirm_payment.call((
                            tid.clone(),
                            amount_cents,
                            payment_method.read().clone(),
                            payment_date.read().clone(),
                        ));
                    },
                    "Confirmar Pagamento"
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 14px;",
                // Resumo do valor e saldo
                div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.08); padding: 12px 14px; border-radius: 8px; display: grid; grid-template-columns: repeat(3, 1fr); text-align: center; gap: 8px;",
                    div {
                        span { style: "display: block; font-size: 11px; color: #94a3b8; text-transform: uppercase;", "Valor Total" }
                        strong { style: "font-size: 14px; color: #f1f5f9;", "R$ {tx.amount_cents as f64 / 100.0:.2}" }
                    }
                    div {
                        span { style: "display: block; font-size: 11px; color: #94a3b8; text-transform: uppercase;", "Já Pago" }
                        strong { style: "font-size: 14px; color: #22c55e;", "R$ {tx.paid_amount_cents as f64 / 100.0:.2}" }
                    }
                    div {
                        span { style: "display: block; font-size: 11px; color: #94a3b8; text-transform: uppercase;", "Saldo Restante" }
                        strong { style: "font-size: 14px; color: #ef4444;", "R$ {remaining_cents as f64 / 100.0:.2}" }
                    }
                }

                div { class: "form-field",
                    label { class: "form-label", "Valor a Pagar / Receber Agora (R$) *" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        step: "0.01",
                        value: "{payment_amount_str}",
                        oninput: move |e| payment_amount_str.set(e.value()),
                    }
                    span { style: "font-size: 11.5px; color: #94a3b8; margin-top: 2px;",
                        "💡 Para pagamento parcial, digite um valor menor que o saldo restante."
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Meio de Pagamento *" }
                        select {
                            class: "form-select",
                            value: "{payment_method}",
                            onchange: move |e| payment_method.set(e.value()),
                            option { value: "pix", "PIX" }
                            option { value: "cartao_debito", "Cartão de Débito" }
                            option { value: "cartao_credito", "Cartão de Crédito" }
                            option { value: "dinheiro", "Dinheiro (Caixa Balcão)" }
                            option { value: "boleto", "Boleto Bancário" }
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Data do Pagamento *" }
                        input {
                            class: "form-input",
                            r#type: "date",
                            value: "{payment_date}",
                            oninput: move |e| payment_date.set(e.value()),
                        }
                    }
                }

                div { class: "form-field",
                    label { class: "form-label", "Observação / Anotação do Pagamento" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Pago 1ª parcela em dinheiro, restante no PIX...",
                        value: "{notes}",
                        oninput: move |e| notes.set(e.value()),
                    }
                }
            }
        }
    }
}
