use crate::components::modal::Modal;
use shared::finance::TransactionDirection;
use dioxus::prelude::*;

#[component]
pub fn ModalTransaction(
    is_open: bool,
    direction: TransactionDirection,
    description: Signal<String>,
    amount_str: Signal<String>,
    category: Signal<String>,
    payment_method: Signal<String>,
    is_paid: Signal<bool>,
    due_date: Signal<String>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let title = if direction == TransactionDirection::Income {
        "Nova Receita / Entrada".to_string()
    } else {
        "Nova Despesa / Saída".to_string()
    };

    rsx! {
        Modal {
            title,
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
                    onclick: move |_| on_submit.call(()),
                    "Salvar Lançamento"
                }
            },

            div { class: "form-field",
                label { class: "form-label", "Descrição / Identificação *" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: "Ex: Pagamento Restauração, Compra de Resina, Aluguel...",
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Valor (R$) *" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        step: "0.01",
                        placeholder: "0.00",
                        value: "{amount_str}",
                        oninput: move |e| amount_str.set(e.value()),
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Categoria" }
                    select {
                        class: "form-select",
                        value: "{category}",
                        onchange: move |e| category.set(e.value()),
                        if direction == TransactionDirection::Income {
                            option { value: "Tratamento Odontológico", "Tratamento Odontológico" }
                            option { value: "Consulta & Avaliação", "Consulta & Avaliação" }
                            option { value: "Manutenção Ortodôntica", "Manutenção Ortodôntica" }
                            option { value: "Outras Entradas", "Outras Entradas" }
                        } else {
                            option { value: "Materiais & Insumos", "Materiais & Insumos" }
                            option { value: "Aluguel & Condomínio", "Aluguel & Condomínio" }
                            option { value: "Laboratório de Prótese", "Laboratório de Prótese" }
                            option { value: "Energia, Água & Internet", "Energia, Água & Internet" }
                            option { value: "Equipe & Honorários", "Equipe & Honorários" }
                            option { value: "Outras Despesas", "Outras Despesas" }
                        }
                    }
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Forma de Pagamento" }
                    select {
                        class: "form-select",
                        value: "{payment_method}",
                        onchange: move |e| payment_method.set(e.value()),
                        option { value: "pix", "PIX" }
                        option { value: "cartao_credito", "Cartão de Crédito" }
                        option { value: "cartao_debito", "Cartão de Débito" }
                        option { value: "dinheiro", "Dinheiro" }
                        option { value: "boleto", "Boleto" }
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Data de Vencimento" }
                    input {
                        class: "form-input",
                        r#type: "date",
                        value: "{due_date}",
                        oninput: move |e| due_date.set(e.value()),
                    }
                }
            }

            div { style: "display: flex; align-items: center; gap: 8px; margin-top: 6px;",
                input {
                    r#type: "checkbox",
                    id: "chk-is-paid",
                    checked: "{is_paid}",
                    onchange: move |e| is_paid.set(e.checked()),
                }
                label { r#for: "chk-is-paid", style: "font-size: 13.5px; color: #cbd5e1; cursor: pointer;",
                    if direction == TransactionDirection::Income {
                        "Recebido no momento do lançamento"
                    } else {
                        "Despesa já paga (débito imediato)"
                    }
                }
            }
        }
    }
}
