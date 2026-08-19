//! # Visualização dos Itens, Alertas e Histórico de Estoque (Frontend)
//!
//! Renderiza cards modernos para Materiais, Químicos e Equipamentos,
//! lista de alertas preventivos da clínica e tabela de histórico de movimentações.

use crate::components::icons::{
    IconAlertTriangle, IconBox, IconEdit, IconFlask, IconRefresh, IconTool, IconTrash,
};
use crate::utils::format_date_br;
use chrono::{DateTime, Utc};
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
        format!("- R$ {}.{:02}", reals, centavos)
    } else {
        format!("R$ {}.{:02}", reals, centavos)
    }
}

/// Seção principal de listagem em grid de cards modernos.
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
                IconBox { size: 48, color: "#94a3b8".to_string() }
                h3 { "Nenhum item encontrado nesta categoria" }
                p { "Cadastre um novo item ou ajuste os termos de busca no filtro acima." }
            }
        };
    }

    rsx! {
        div { class: "stock-cards-grid",
            for item in &items {
                {
                    let item_clone = item.clone();
                    let item_clone_mov = item.clone();
                    let item_clone_del = item.clone();

                    let is_low_stock = item.item_type != ItemType::Equipment && item.current_stock <= item.min_stock;

                    let mut is_expiring_soon = false;
                    let mut is_expired = false;
                    if item.item_type == ItemType::Chemical {
                        if let Some(ref exp_str) = item.expiration_date {
                            if let Ok(exp_dt) = DateTime::parse_from_rfc3339(exp_str) {
                                let exp_utc = exp_dt.with_timezone(&Utc);
                                let now = Utc::now();
                                if exp_utc < now {
                                    is_expired = true;
                                } else if (exp_utc - now).num_days() <= 30 {
                                    is_expiring_soon = true;
                                }
                            }
                        }
                    }

                    let max_scale = if item.min_stock > 0 { (item.min_stock * 2) as f64 } else { 10.0 };
                    let fill_pct = if max_scale > 0.0 {
                        ((item.current_stock as f64 / max_scale) * 100.0).clamp(5.0, 100.0)
                    } else {
                        100.0
                    };

                    rsx! {
                        div { key: "{item.id}", class: "stock-item-card",
                            // 1. Header do Card (Tags na esquerda, Ações na direita)
                            div { class: "stock-card-header",
                                div { class: "stock-badges-group",
                                    match item.item_type {
                                        ItemType::Material => rsx! {
                                            span { class: "stock-tag tag-material", "MATERIAL" }
                                        },
                                        ItemType::Chemical => rsx! {
                                            span { class: "stock-tag tag-chemical", "QUÍMICO / FARMÁCIA" }
                                        },
                                        ItemType::Equipment => rsx! {
                                            span { class: "stock-tag tag-equipment", "EQUIPAMENTO" }
                                        },
                                    }

                                    if is_low_stock {
                                        span { class: "stock-tag tag-low-stock", "ESTOQUE BAIXO" }
                                    }

                                    if is_expired {
                                        span { class: "stock-tag tag-expired", "VENCIDO" }
                                    } else if is_expiring_soon {
                                        span { class: "stock-tag tag-expiring", "VALIDADE PRÓXIMA" }
                                    }

                                    if item.item_type == ItemType::Equipment {
                                        match item.equipment_status {
                                            Some(EquipmentStatus::InMaintenance) => rsx! {
                                                span { class: "stock-tag tag-maint", "EM MANUTENÇÃO" }
                                            },
                                            Some(EquipmentStatus::Broken) => rsx! {
                                                span { class: "stock-tag tag-broken", "INOPERANTE" }
                                            },
                                            _ => rsx! {
                                                span { class: "stock-tag tag-active", "OPERACIONAL" }
                                            },
                                        }
                                    }
                                }

                                div { class: "stock-card-actions",
                                    if can_movement {
                                        button {
                                            class: "stock-action-icon-btn",
                                            title: "Registrar Movimentação",
                                            onclick: move |_| on_movement.call(item_clone_mov.clone()),
                                            IconRefresh { size: 14, color: "currentColor".to_string() }
                                        }
                                    }
                                    if can_write {
                                        button {
                                            class: "stock-action-icon-btn",
                                            title: "Editar Item",
                                            onclick: move |_| on_edit.call(item_clone.clone()),
                                            IconEdit { size: 14, color: "currentColor".to_string() }
                                        }
                                    }
                                    if can_delete {
                                        button {
                                            class: "stock-action-icon-btn btn-danger-icon",
                                            title: "Excluir Item",
                                            onclick: move |_| on_delete.call(item_clone_del.clone()),
                                            IconTrash { size: 14, color: "currentColor".to_string() }
                                        }
                                    }
                                }
                            }

                            // 2. Título e Fabricante
                            div { class: "stock-card-body",
                                h3 { class: "stock-item-title", "{item.name}" }
                                div { class: "stock-item-manufacturer",
                                    "Fabricante: {item.manufacturer.as_deref().unwrap_or(\"Não informado\")}"
                                }

                                // 3. Barra de Quantidade / Progresso ou Status
                                if item.item_type != ItemType::Equipment {
                                    div { class: "stock-progress-section",
                                        div { class: "stock-progress-labels",
                                            span { class: "stock-current-qty", "{item.current_stock} {item.unit_type}" }
                                            span { class: "stock-min-qty", "Mínimo: {item.min_stock} {item.unit_type}" }
                                        }
                                        div { class: "stock-progress-track",
                                            div {
                                                class: if is_low_stock { "stock-progress-fill fill-low" } else { "stock-progress-fill fill-normal" },
                                                style: "width: {fill_pct}%;"
                                            }
                                        }
                                    }
                                } else {
                                    div { class: "stock-equipment-qty-badge",
                                        "{item.current_stock} {item.unit_type}"
                                    }
                                }
                            }

                            // 4. Detalhes de Rodapé
                            div { class: "stock-card-footer",
                                match item.item_type {
                                    ItemType::Material => rsx! {
                                        div { class: "stock-footer-row",
                                            span { class: "stock-footer-label", "Custo Unitário:" }
                                            span { class: "stock-footer-val", "{format_currency(item.cost_price_cents)}" }
                                        }
                                    },
                                    ItemType::Chemical => rsx! {
                                        div { class: "stock-footer-grid",
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Custo Unitário:" }
                                                span { class: "stock-footer-val", "{format_currency(item.cost_price_cents)}" }
                                            }
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Lote:" }
                                                span { class: "stock-footer-val", "{item.batch_number.as_deref().unwrap_or(\"-\")}" }
                                            }
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Validade:" }
                                                span { class: "stock-footer-val", "{format_date_br(item.expiration_date.as_deref().unwrap_or(\"\"))}" }
                                            }
                                        }
                                    },
                                    ItemType::Equipment => rsx! {
                                        div { class: "stock-footer-grid-2x2",
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Custo Unitário:" }
                                                span { class: "stock-footer-val", "{format_currency(item.cost_price_cents)}" }
                                            }
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Nº de Série:" }
                                                span { class: "stock-footer-val", "{item.serial_number.as_deref().unwrap_or(\"-\")}" }
                                            }
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Próx. Manutenção:" }
                                                span { class: "stock-footer-val", "{format_date_br(item.next_maintenance_date.as_deref().unwrap_or(\"\"))}" }
                                            }
                                            div { class: "stock-footer-row",
                                                span { class: "stock-footer-label", "Garantia até:" }
                                                span { class: "stock-footer-val", "{format_date_br(item.warranty_until.as_deref().unwrap_or(\"\"))}" }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Seção da Central de Alertas Inteligentes.
#[component]
pub fn StockAlertsSection(
    alerts: Vec<StockAlertItem>,
    items: Vec<InventoryItem>,
    on_resolve: EventHandler<InventoryItem>,
) -> Element {
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
        div { class: "stock-alerts-list",
            for alert in &alerts {
                {
                    let is_critical = alert.severity == StockAlertSeverity::Critical;
                    let target_matched = items.iter().find(|i| i.id == alert.item_id).cloned();
                    let badge_text = match alert.alert_type {
                        shared::stock::StockAlertType::LowStock => "Estoque Baixo",
                        shared::stock::StockAlertType::ExpiringSoon => "Validade Próxima",
                        shared::stock::StockAlertType::Expired => "Produto Vencido",
                        shared::stock::StockAlertType::MaintenanceDue => "Revisão Próxima",
                        shared::stock::StockAlertType::MaintenanceOverdue => "Manutenção Atrasada",
                    };

                    rsx! {
                        div {
                            key: "{alert.id}",
                            class: if is_critical { "stock-alert-row alert-critical" } else { "stock-alert-row alert-warning" },
                            div { class: "stock-alert-icon-box",
                                IconAlertTriangle { size: 18, color: "currentColor".to_string() }
                            }
                            div { class: "stock-alert-main",
                                div { class: "stock-alert-title-row",
                                    span { class: "stock-alert-heading", "{alert.title}" }
                                    span { class: "stock-alert-pill", "{badge_text}" }
                                }
                                p { class: "stock-alert-desc", "{alert.message}" }
                            }
                            div { class: "stock-alert-actions-side",
                                div { class: "stock-alert-meta-ref",
                                    span { class: "stock-alert-meta-lbl", "Meta/Ref: " }
                                    span { class: "stock-alert-meta-val", "{alert.target_value}" }
                                }
                                button {
                                    class: "btn-secondary btn-sm btn-resolver-alert",
                                    onclick: move |_| {
                                        if let Some(ref it) = target_matched {
                                            on_resolve.call(it.clone());
                                        }
                                    },
                                    "Movimentar / Resolver"
                                }
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
            div { class: "stock-empty-history-card",
                IconRefresh { size: 40, color: "#94a3b8".to_string() }
                h3 { "Nenhuma movimentação registrada no histórico." }
                p { "Clique em 'Registrar Movimentação' para lançar compras, entradas ou baixas manuais." }
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
