use crate::icons::{IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn StockToolbar(
    search_query: Signal<String>,
    on_new_movement: EventHandler<()>,
    on_new_item: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "stock-toolbar",
            div { class: "stock-search-box",
                IconSearch { size: 15, color: "#64748b".to_string() }
                input {
                    r#type: "text",
                    class: "stock-search-input",
                    placeholder: "Buscar material ou produto...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value()),
                }
            }

            div { class: "stock-actions",
                button {
                    r#type: "button",
                    class: "btn-movement",
                    onclick: move |_| on_new_movement.call(()),
                    "📦 Registrar Movimentação"
                }
                button {
                    r#type: "button",
                    class: "btn-add-item",
                    onclick: move |_| on_new_item.call(()),
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "+ Novo Produto" }
                }
            }
        }
    }
}
