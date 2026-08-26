use crate::icons::{IconMinus, IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn FinanceToolbar(
    type_filter: Signal<String>,
    search_query: Signal<String>,
    on_search: EventHandler<()>,
    on_new_income: EventHandler<()>,
    on_new_expense: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "finance-toolbar",
            div { class: "finance-filters",
                select {
                    class: "finance-select",
                    value: "{type_filter}",
                    onchange: move |e| {
                        type_filter.set(e.value());
                        on_search.call(());
                    },
                    option { value: "ALL", "Todas as Movimentações" }
                    option { value: "INCOME", "Apenas Receitas" }
                    option { value: "EXPENSE", "Apenas Despesas" }
                }

                div { class: "finance-search-box",
                    IconSearch { size: 15, color: "#64748b".to_string() }
                    input {
                        r#type: "text",
                        class: "finance-search-input",
                        placeholder: "Buscar lançamento por descrição...",
                        value: "{search_query}",
                        oninput: move |e| {
                            search_query.set(e.value());
                            on_search.call(());
                        },
                    }
                }
            }

            div { class: "finance-actions",
                button {
                    r#type: "button",
                    class: "btn-income",
                    onclick: move |_| on_new_income.call(()),
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "+ Nova Receita" }
                }
                button {
                    r#type: "button",
                    class: "btn-expense",
                    onclick: move |_| on_new_expense.call(()),
                    IconMinus { size: 15, color: "#ffffff".to_string() }
                    span { "- Nova Despesa" }
                }
            }
        }
    }
}
