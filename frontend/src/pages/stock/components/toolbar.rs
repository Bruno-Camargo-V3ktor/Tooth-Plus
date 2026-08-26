use crate::icons::{IconArrowUpDown, IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn StockToolbar(
    search_query: Signal<String>,
    type_filter: Signal<String>,
    on_new_item: EventHandler<()>,
    on_movement: EventHandler<()>,
) -> Element {
    let current_filter = type_filter.read().clone();

    rsx! {
        div { class: "stock-toolbar-row",
            div { class: "stock-search-box",
                IconSearch { size: 16, color: "#64748b".to_string() }
                input {
                    r#type: "text",
                    placeholder: "Buscar por nome, fabricante ou lote...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value()),
                }
            }

            div { class: "stock-category-filters",
                button {
                    class: if current_filter == "ALL" { "btn-filter-pill active" } else { "btn-filter-pill" },
                    onclick: move |_| type_filter.set("ALL".to_string()),
                    "Todos"
                }
                button {
                    class: if current_filter == "material" { "btn-filter-pill active" } else { "btn-filter-pill" },
                    onclick: move |_| type_filter.set("material".to_string()),
                    "Materiais & Insumos"
                }
                button {
                    class: if current_filter == "chemical" { "btn-filter-pill active" } else { "btn-filter-pill" },
                    onclick: move |_| type_filter.set("chemical".to_string()),
                    "Químicos / Medicamentos"
                }
                button {
                    class: if current_filter == "equipment" { "btn-filter-pill active" } else { "btn-filter-pill" },
                    onclick: move |_| type_filter.set("equipment".to_string()),
                    "Equipamentos"
                }
            }

            div { style: "display: flex; align-items: center; gap: 8px;",
                button {
                    r#type: "button",
                    class: "btn-secondary",
                    style: "font-weight: 700;",
                    onclick: move |_| on_movement.call(()),
                    IconArrowUpDown { size: 15, color: "#94a3b8".to_string() }
                    span { "Movimentação" }
                }

                button {
                    r#type: "button",
                    class: "btn-new-patient-green",
                    onclick: move |_| on_new_item.call(()),
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "NOVO ITEM" }
                }
            }
        }
    }
}
