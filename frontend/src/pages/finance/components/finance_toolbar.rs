use crate::icons::{IconChevronDown, IconFilter, IconPlus, IconSearch, IconTrendingUp};
use dioxus::prelude::*;

#[component]
pub fn FinanceToolbar(
    period_filter: Signal<String>,
    start_date: Signal<String>,
    end_date: Signal<String>,
    search_query: Signal<String>,
    on_open_filter_modal: EventHandler<()>,
    on_new_transaction: EventHandler<String>,
) -> Element {
    let mut is_add_menu_open = use_signal(|| false);

    let is_custom_period = *period_filter.read() == "custom";

    rsx! {
        div { class: "finance-top-toolbar",
            div { class: "finance-toolbar-left",
                div { class: "period-select-group",
                    span { "Exibindo financeiro" }
                    select {
                        class: "form-select",
                        style: "height: 38px; min-width: 150px; font-weight: 600;",
                        value: "{period_filter}",
                        onchange: move |e| period_filter.set(e.value()),
                        option { value: "today", "de hoje" }
                        option { value: "week", "desta semana" }
                        option { value: "month", "desse mês" }
                        option { value: "year", "desse ano" }
                        option { value: "custom", "escolher período" }
                        option { value: "all", "de todos os períodos" }
                    }
                }

                if is_custom_period {
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        div { class: "period-date-input-group",
                            label { "Data inicial*" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                style: "height: 36px; padding: 2px 8px; font-size: 12.5px;",
                                value: "{start_date}",
                                oninput: move |e| start_date.set(e.value()),
                            }
                        }
                        div { class: "period-date-input-group",
                            label { "Data final*" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                style: "height: 36px; padding: 2px 8px; font-size: 12.5px;",
                                value: "{end_date}",
                                oninput: move |e| end_date.set(e.value()),
                            }
                        }
                    }
                }

                div { class: "finance-search-box",
                    IconSearch { size: 16, color: "#64748b".to_string() }
                    input {
                        r#type: "text",
                        placeholder: "Buscar lançamentos...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                button {
                    r#type: "button",
                    class: "btn-filter-toggle",
                    onclick: move |_| on_open_filter_modal.call(()),
                    IconFilter { size: 15, color: "#94a3b8".to_string() }
                    span { "FILTRAR" }
                }
            }

            div { class: "finance-toolbar-right",
                button {
                    r#type: "button",
                    class: "btn-add-dropdown-green",
                    onclick: move |_| is_add_menu_open.set(!is_add_menu_open()),
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "ADICIONAR" }
                    IconChevronDown { size: 13, color: "#ffffff".to_string() }
                }

                if is_add_menu_open() {
                    div { class: "finance-add-menu",
                        button {
                            r#type: "button",
                            class: "finance-menu-item menu-item-expense",
                            onclick: move |_| {
                                is_add_menu_open.set(false);
                                on_new_transaction.call("expense".to_string());
                            },
                            span { "↗" }
                            span { "Despesa" }
                        }
                        button {
                            r#type: "button",
                            class: "finance-menu-item menu-item-income",
                            onclick: move |_| {
                                is_add_menu_open.set(false);
                                on_new_transaction.call("income".to_string());
                            },
                            span { "↙" }
                            span { "Receita" }
                        }
                        button {
                            r#type: "button",
                            class: "finance-menu-item menu-item-income",
                            onclick: move |_| {
                                is_add_menu_open.set(false);
                                on_new_transaction.call("income_unlinked".to_string());
                            },
                            span { "↙" }
                            span { "Receita sem vínculo" }
                        }
                    }
                }

                button {
                    r#type: "button",
                    class: "btn-filter-toggle",
                    IconTrendingUp { size: 15, color: "#94a3b8".to_string() }
                    span { "Relatórios ▾" }
                }
            }
        }
    }
}
