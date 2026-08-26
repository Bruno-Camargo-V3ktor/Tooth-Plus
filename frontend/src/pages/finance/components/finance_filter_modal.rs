use crate::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn FinanceFilterModal(
    is_open: bool,
    filter_income: Signal<bool>,
    filter_unlinked: Signal<bool>,
    filter_expense: Signal<bool>,
    filter_paid: Signal<bool>,
    filter_unpaid: Signal<bool>,
    filter_scheduled: Signal<bool>,
    account_filter: Signal<String>,
    payment_method_filter: Signal<String>,
    on_close: EventHandler<()>,
    on_apply: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    rsx! {
        Modal {
            title: "Filtrar por".to_string(),
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                button {
                    r#type: "button",
                    class: "btn-modal-ghost",
                    onclick: move |_| on_close.call(()),
                    "FECHAR"
                }
                button {
                    r#type: "button",
                    class: "btn-filter-apply",
                    onclick: move |_| on_apply.call(()),
                    "FILTRAR"
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 16px;",
                // Linha 1: Tipos de Transação
                div { class: "filter-checkbox-grid-3",
                    label { class: "filter-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{filter_income}",
                            onchange: move |e| filter_income.set(e.checked()),
                        }
                        span { "Receitas" }
                    }
                    label { class: "filter-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{filter_unlinked}",
                            onchange: move |e| filter_unlinked.set(e.checked()),
                        }
                        span { "Receitas sem vínculo" }
                    }
                    label { class: "filter-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{filter_expense}",
                            onchange: move |e| filter_expense.set(e.checked()),
                        }
                        span { "Despesas" }
                    }
                }

                // Linha 2: Status
                div { class: "filter-checkbox-grid-3",
                    label { class: "filter-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{filter_paid}",
                            onchange: move |e| filter_paid.set(e.checked()),
                        }
                        span { "Pagas" }
                    }
                    label { class: "filter-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{filter_unpaid}",
                            onchange: move |e| filter_unpaid.set(e.checked()),
                        }
                        span { "Não pagas" }
                    }
                    label { class: "filter-checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{filter_scheduled}",
                            onchange: move |e| filter_scheduled.set(e.checked()),
                        }
                        span { "Agendadas" }
                    }
                }

                // Linha 3: Selects
                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Conta financeira" }
                        select {
                            class: "form-select",
                            value: "{account_filter}",
                            onchange: move |e| account_filter.set(e.value()),
                            option { value: "all", "Todas as contas" }
                            option { value: "caixa_principal", "Caixa Principal (Balcão)" }
                            option { value: "banco_itau", "Conta Corrente Itaú" }
                            option { value: "banco_inter", "Banco Inter PJ" }
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Meio de pagamento" }
                        select {
                            class: "form-select",
                            value: "{payment_method_filter}",
                            onchange: move |e| payment_method_filter.set(e.value()),
                            option { value: "all", "Todos os meios" }
                            option { value: "pix", "PIX" }
                            option { value: "cartao_credito", "Cartão de Crédito" }
                            option { value: "cartao_debito", "Cartão de Débito" }
                            option { value: "dinheiro", "Dinheiro" }
                            option { value: "boleto", "Boleto Bancário" }
                        }
                    }
                }
            }
        }
    }
}
