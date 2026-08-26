use shared::stock::InventoryItem;
use dioxus::prelude::*;

#[component]
pub fn StockTable(items: Vec<InventoryItem>) -> Element {
    rsx! {
        div { class: "stock-table-container",
            table { class: "stock-table",
                thead {
                    tr {
                        th { "Nome do Produto / Material" }
                        th { "Tipo / Categoria" }
                        th { "Fabricante" }
                        th { "Estoque Atual" }
                        th { "Mínimo" }
                        th { "Status" }
                    }
                }
                tbody {
                    for item in items {
                        {
                            let is_low = item.current_stock <= item.min_stock;
                            let manufacturer_display = item.manufacturer.clone().unwrap_or_else(|| "Geral".to_string());
                            let type_str = format!("{:?}", item.item_type);

                            rsx! {
                                tr {
                                    key: "{item.id}",
                                    td {
                                        strong { style: "color: #f1f5f9;", "{item.name}" }
                                    }
                                    td { "{type_str}" }
                                    td { "{manufacturer_display}" }
                                    td {
                                        strong {
                                            style: if is_low { "color: #f87171;" } else { "color: #34d399;" },
                                            "{item.current_stock} {item.unit_type}"
                                        }
                                    }
                                    td { "{item.min_stock} {item.unit_type}" }
                                    td {
                                        span {
                                            class: if is_low { "badge-stock-low" } else { "badge-stock-ok" },
                                            if is_low { "Estoque Baixo" } else { "Regular" }
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
