//! # Tabelas de Itens, Alertas e Histórico de Estoque (Frontend)
//!
//! Exibe as tabelas de itens de estoque com alertas visuais de nível mínimo,
//! painel de alertas inteligentes e histórico detalhado de movimentações recentes.

use crate::components::icons::{
    IconAlertTriangle, IconArrowDown, IconArrowUp, IconBox, IconEdit, IconFlask, IconTool,
    IconTrash,
};
use dioxus::prelude::*;
use shared::stock::{
    EquipmentStatus, InventoryItem, ItemType, MovementType, StockAlertItem, StockAlertSeverity,
    StockMovement,
};

/// Formata valor em centavos para moeda BRL.
fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;
    if is_negative {
        format!("-R$ {}.{:02}", reals, centavos)
    } else {
        format!("R$ {}.{:02}", reals, centavos)
    }
}

/// Seção da tabela de itens de inventário.
#[component]
pub fn StockItemsSection(
    items: Vec<InventoryItem>,
    can_write: bool,
    can_delete: bool,
    can_movement: bool,
    on_movement: EventHandler<InventoryItem>,
    on_edit: EventHandler<InventoryItem>,
    on_delete: EventHandler<InventoryItem>,
) -> Element {
    if items.is_empty() {
        return rsx! {
            div { class: "empty-state-card",
                IconBox { size: 48, color: "var(--text-muted, #8c8c8c)".to_string() }
                h3 { "Nenhum item localizado no estoque" }
                p { "Utilize a busca ou cadastre um novo material no botão acima." }
            }
        };
    }

    rsx! {
        div { class: "table-responsive",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Item / Descrição" }
                        th { "Categoria" }
                        th { "Estoque Atual" }
                        th { "Nível Mínimo" }
                        th { "Custo Unitário" }
                        th { "Valor Total" }
                        th { "Validade / Status" }
                        th { "Ações" }
                    }
                }
                tbody {
                    for item in &items {
                        {
                            let item_clone = item.clone();
                            let item_clone_mov = item.clone();
                            let item_clone_del = item.clone();
                            let is_low_stock = item.item_type != ItemType::Equipment && item.current_stock <= item.min_stock;
                            let total_val = (item.current_stock.max(0) as i64) * item.cost_price_cents;

                            let (cat_label, cat_badge) = match item.item_type {
                                ItemType::Chemical => ("Químico", "badge-warning"),
                                ItemType::Equipment => ("Equipamento", "badge-primary"),
                                _ => ("Material", "badge-outline"),
                            };

                            rsx! {
                                tr { key: "{item.id}",
                                    td {
                                        strong { "{item.name}" }
                                        if let Some(ref mfg) = item.manufacturer {
                                            div { class: "text-muted font-xs", "Fabricante: {mfg}" }
                                        }
                                    }
                                    td {
                                        span { class: "{cat_badge}", "{cat_label}" }
                                    }
                                    td {
                                        if is_low_stock {
                                            span { class: "badge-danger font-mono font-weight-bold",
                                                "{item.current_stock} {item.unit_type}"
                                            }
                                        } else {
                                            span { class: "font-mono font-weight-bold",
                                                "{item.current_stock} {item.unit_type}"
                                            }
                                        }
                                    }
                                    td {
                                        span { class: "text-muted font-mono font-xs",
                                            "{item.min_stock} {item.unit_type}"
                                        }
                                    }
                                    td { class: "font-mono font-xs", "{format_currency(item.cost_price_cents)}" }
                                    td { class: "font-mono font-weight-bold", "{format_currency(total_val)}" }
                                    td {
                                        if let Some(ref exp) = item.expiration_date {
                                            div { class: "font-xs", "Val: {exp.chars().take(10).collect::<String>()}" }
                                        } else if let Some(ref st) = item.equipment_status {
                                            match st {
                                                EquipmentStatus::InMaintenance => rsx! { span { class: "badge-warning font-xs", "Em Manutenção" } },
                                                EquipmentStatus::Broken => rsx! { span { class: "badge-danger font-xs", "Inoperante" } },
                                                _ => rsx! { span { class: "badge-success font-xs", "Ativo" } },
                                            }
                                        } else {
                                            span { class: "text-muted font-xs", "-" }
                                        }
                                    }
                                    td { class: "actions-cell",
                                        if can_movement {
                                            button {
                                                class: "btn-secondary btn-sm",
                                                title: "Lançar Entrada ou Saída",
                                                onclick: move |_| on_movement.call(item_clone_mov.clone()),
                                                "Movimentar"
                                            }
                                        }
                                        if can_write {
                                            button {
                                                class: "btn-icon",
                                                title: "Editar Item",
                                                onclick: move |_| on_edit.call(item_clone.clone()),
                                                IconEdit { size: 14, color: "currentColor".to_string() }
                                            }
                                        }
                                        if can_delete {
                                            button {
                                                class: "btn-icon text-danger",
                                                title: "Excluir Item",
                                                onclick: move |_| on_delete.call(item_clone_del.clone()),
                                                IconTrash { size: 14, color: "currentColor".to_string() }
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
}

/// Seção de alertas inteligentes de estoque.
#[component]
pub fn StockAlertsSection(alerts: Vec<StockAlertItem>) -> Element {
    if alerts.is_empty() {
        return rsx! {
            div { class: "empty-state-card",
                IconBox { size: 48, color: "#10b981".to_string() }
                h3 { "Nenhum alerta de estoque pendente!" }
                p { "Todos os materiais e produtos químicos estão dentro dos níveis ideais de validade e saldo." }
            }
        };
    }

    rsx! {
        div { class: "alerts-grid",
            for alert in &alerts {
                {
                    let is_critical = alert.severity == StockAlertSeverity::Critical;
                    let card_class = if is_critical { "alert-card critical-alert" } else { "alert-card warning-alert" };

                    rsx! {
                        div { key: "{alert.id}", class: "{card_class}",
                            div { class: "alert-header",
                                div { class: "alert-icon-wrap",
                                    IconAlertTriangle { size: 18, color: "currentColor".to_string() }
                                }
                                h4 { class: "alert-title", "{alert.title}" }
                            }
                            p { class: "alert-message", "{alert.message}" }
                            div { class: "alert-meta-row",
                                span { class: "alert-meta-item", "Item: ", strong { "{alert.item_name}" } }
                                span { class: "alert-meta-badge", "{alert.current_value}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Seção de histórico de movimentações de estoque.
#[component]
pub fn StockMovementsSection(movements: Vec<StockMovement>) -> Element {
    if movements.is_empty() {
        return rsx! {
            div { class: "empty-state-card",
                IconBox { size: 48, color: "var(--text-muted, #8c8c8c)".to_string() }
                h3 { "Nenhuma movimentação registrada" }
                p { "As entradas e saídas de estoque realizadas serão listadas aqui." }
            }
        };
    }

    rsx! {
        div { class: "table-responsive",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Data / Hora" }
                        th { "Tipo de Movimentação" }
                        th { "Variação de Saldo" }
                        th { "Nota Fiscal" }
                        th { "Observações" }
                    }
                }
                tbody {
                    for mov in &movements {
                        {
                            let is_in = mov.quantity_change > 0;
                            let change_label = if is_in { format!("+{}", mov.quantity_change) } else { mov.quantity_change.to_string() };
                            let change_badge = if is_in { "badge-success font-mono" } else { "badge-danger font-mono" };

                            let type_label = match mov.movement_type {
                                MovementType::PurchaseIn => "Entrada por Compra",
                                MovementType::ManualOut => "Saída Manual",
                                MovementType::AppointmentOut => "Consumo em Atendimento",
                                MovementType::Adjustment => "Ajuste de Inventário",
                                MovementType::Loss => "Perda / Descarte",
                            };

                            rsx! {
                                tr { key: "{mov.id}",
                                    td { class: "font-mono font-xs",
                                        "{mov.created_at.chars().take(16).collect::<String>().replace('T', \" \")}"
                                    }
                                    td {
                                        span { class: "badge-outline", "{type_label}" }
                                    }
                                    td {
                                        span { class: "{change_badge}", "{change_label}" }
                                    }
                                    td {
                                        span { class: "font-mono font-xs", "{mov.invoice_number.as_deref().unwrap_or(\"-\")}" }
                                    }
                                    td { class: "text-muted font-xs",
                                        "{mov.notes.as_deref().unwrap_or(\"-\")}"
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
