use crate::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn ModalTransaction(
    is_open: bool,
    is_income: bool,
    description: Signal<String>,
    amount_str: Signal<String>,
    category: Signal<String>,
    payment_method: Signal<String>,
    due_date: Signal<String>,
    is_paid: Signal<bool>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let title = if is_income { "Nova Receita".to_string() } else { "Nova Despesa".to_string() };

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
                    "Confirmar Lançamento"
                }
            },

            div { class: "form-field",
                label { class: "form-label", "Descrição / Identificação *" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: if is_income { "Ex: Consulta Dr. Roberto, Implante..." } else { "Ex: Compra de Luvas, Aluguel..." },
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
                    label { class: "form-label", "Data de Vencimento *" }
                    input {
                        class: "form-input",
                        r#type: "date",
                        value: "{due_date}",
                        oninput: move |e| due_date.set(e.value()),
                    }
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Categoria" }
                    select {
                        class: "form-select",
                        value: "{category}",
                        onchange: move |e| category.set(e.value()),
                        option { value: "Tratamentos", "Tratamentos & Procedimentos" }
                        option { value: "Materiais", "Materiais & Insumos" }
                        option { value: "Operacional", "Despesas Operacionais" }
                        option { value: "Outros", "Outros" }
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Forma de Pagamento" }
                    select {
                        class: "form-select",
                        value: "{payment_method}",
                        onchange: move |e| payment_method.set(e.value()),
                        option { value: "PIX", "PIX" }
                        option { value: "Cartão de Crédito", "Cartão de Crédito" }
                        option { value: "Cartão de Débito", "Cartão de Débito" }
                        option { value: "Boleto", "Boleto" }
                        option { value: "Dinheiro", "Dinheiro" }
                    }
                }
            }

            div { class: "form-checkbox-wrap", style: "margin-top: 4px;",
                input {
                    r#type: "checkbox",
                    id: "chk-is-paid",
                    checked: "{is_paid}",
                    onchange: move |e| is_paid.set(e.checked()),
                }
                label { r#for: "chk-is-paid", "Lançamento já quitado / pago agora" }
            }
        }
    }
}
