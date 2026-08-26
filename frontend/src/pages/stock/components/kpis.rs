use shared::stock::InventoryItem;
use dioxus::prelude::*;

#[component]
pub fn StockKpis(items: Vec<InventoryItem>) -> Element {
    let total_items = items.len();
    let low_stock_items: Vec<&InventoryItem> = items
        .iter()
        .filter(|i| i.current_stock <= i.min_stock)
        .collect();
    let low_stock_count = low_stock_items.len();
    let total_units: i32 = items.iter().map(|i| i.current_stock).sum();

    rsx! {
        div { class: "stock-kpi-grid",
            div { class: "stock-kpi-card",
                span { class: "stock-kpi-label", "Total de Itens Cadastrados" }
                span { class: "stock-kpi-value", "{total_items}" }
            }
            div { class: if low_stock_count > 0 { "stock-kpi-card kpi-alert" } else { "stock-kpi-card" },
                span { class: "stock-kpi-label", "Itens em Estoque Baixo / Crítico" }
                span { class: "stock-kpi-value", style: if low_stock_count > 0 { "color: #f87171;" } else { "" }, "{low_stock_count}" }
            }
            div { class: "stock-kpi-card",
                span { class: "stock-kpi-label", "Volume Total em Estoque" }
                span { class: "stock-kpi-value", "{total_units} un" }
            }
        }

        if low_stock_count > 0 {
            div { class: "stock-alerts-container",
                div { class: "stock-alert-banner alert-warning",
                    span { "⚠️ Atenção: {low_stock_count} item(ns) estão abaixo do estoque mínimo de segurança!" }
                }
            }
        }
    }
}
