use crate::icons::{IconArrowUpDown, IconBox, IconEdit, IconTrash};
use shared::stock::{InventoryItem, ItemType};
use dioxus::prelude::*;

#[component]
pub fn StockTable(
    items: Vec<InventoryItem>,
    on_edit: EventHandler<String>,
    on_movement: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    if items.is_empty() {
        return rsx! {
            div { class: "empty-debits-box",
                div { class: "empty-debits-icon",
                    IconBox { size: 48, color: "#475569".to_string() }
                }
                h3 { class: "empty-debits-title", "Nenhum item encontrado no estoque" }
                p { class: "empty-debits-desc", "Cadastre materiais, medicamentos e equipamentos para controle de reposição." }
            }
        };
    }

    rsx! {
        div { class: "treatments-grid",
            for item in items {
                {
                    let iid = item.id.clone();
                    let iid_mov = item.id.clone();
                    let iid_del = item.id.clone();

                    let type_label = match item.item_type {
                        ItemType::Material => "Material / Insumo",
                        ItemType::Chemical => "Químico / Medicamento",
                        ItemType::Equipment => "Equipamento",
                    };

                    let is_low_stock = item.current_stock <= item.min_stock && item.current_stock > 0;
                    let is_out_of_stock = item.current_stock == 0;

                    let stock_badge_cls = if is_out_of_stock {
                        "badge badge-red"
                    } else if is_low_stock {
                        "badge badge-yellow"
                    } else {
                        "badge badge-green"
                    };

                    let stock_badge_text = if is_out_of_stock {
                        "Estoque Zerado"
                    } else if is_low_stock {
                        "Estoque Baixo"
                    } else {
                        "Em Estoque"
                    };

                    let cost_fmt = format!("R$ {:.2}", item.cost_price_cents as f64 / 100.0);
                    let mfg = item.manufacturer.clone().unwrap_or_else(|| "Geral".to_string());

                    rsx! {
                        div { key: "{item.id}", class: "treatment-card",
                            div { class: "treatment-card-top",
                                div {
                                    span { class: "treatment-card-cat", "{type_label}" }
                                    h4 { class: "treatment-card-name", "{item.name}" }
                                }
                                div { style: "display: flex; align-items: center; gap: 6px;",
                                    button {
                                        r#type: "button",
                                        class: "action-btn-icon",
                                        title: "Registrar Movimentação",
                                        onclick: move |_| on_movement.call(iid_mov.clone()),
                                        IconArrowUpDown { size: 14, color: "#38bdf8".to_string() }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "action-btn-icon",
                                        title: "Editar Item",
                                        onclick: move |_| on_edit.call(iid.clone()),
                                        IconEdit { size: 14, color: "#94a3b8".to_string() }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "action-btn-icon",
                                        title: "Excluir Item",
                                        onclick: move |_| on_delete.call(iid_del.clone()),
                                        IconTrash { size: 14, color: "#ef4444".to_string() }
                                    }
                                }
                            }

                            div { style: "display: flex; align-items: center; justify-content: space-between; margin-top: 4px;",
                                span { style: "font-size: 12.5px; color: #94a3b8;", "Fabricante: {mfg}" }
                                span { class: "{stock_badge_cls}", "{stock_badge_text}" }
                            }

                            div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.06); padding: 8px 12px; border-radius: 6px; display: grid; grid-template-columns: 1fr 1fr; gap: 6px; font-size: 12px;",
                                div {
                                    span { style: "color: #64748b; display: block;", "Estoque Atual" }
                                    strong { style: "color: #f1f5f9; font-size: 13.5px;", "{item.current_stock} {item.unit_type}" }
                                }
                                div {
                                    span { style: "color: #64748b; display: block;", "Estoque Mínimo" }
                                    strong { style: "color: #94a3b8; font-size: 13.5px;", "{item.min_stock} {item.unit_type}" }
                                }
                                if let Some(ref val) = item.expiration_date {
                                    div { style: "grid-column: 1 / -1; font-size: 11.5px; color: #cbd5e1;",
                                        "Validade: {val}"
                                    }
                                }
                            }

                            div { class: "treatment-card-footer",
                                span { style: "font-size: 12px; color: #64748b;", "Custo Unitário:" }
                                span { class: "treatment-price-val", "{cost_fmt}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
