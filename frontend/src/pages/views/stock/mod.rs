//! # Módulo de Visualização de Estoque e Almoxarifado Clínico (Frontend)
//!
//! Controla a visão consolidada de materiais, químicos, equipamentos e instrumentais,
//! alertas inteligentes de reposição/validade e movimentações de compra/consumo.

pub mod item_modal;
pub mod items_table;
pub mod movement_modal;

pub use item_modal::*;
pub use items_table::*;
pub use movement_modal::*;

use crate::api::{delete_stock_item, fetch_stock_data};
use crate::components::icons::{
    IconAlertTriangle, IconBox, IconFile, IconFlask, IconPlus, IconRefresh, IconSearch, IconTool,
};
use crate::permissions::has_permission;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::stock::{InventoryItem, ItemType, StockKPIs, StockMovement, StockResponse};

/// Formata moeda BRL para exibição em KPIs.
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

/// Abas de navegação da tela de estoque.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StockTab {
    Materials,
    Chemicals,
    Equipments,
    Alerts,
    Movements,
}

/// Componente principal da tela de Gestão de Estoque e Suprimentos.
#[component]
pub fn StockView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = has_permission(&sess, &clinic, "stock:read");
    let can_write = has_permission(&sess, &clinic, "stock:write");
    let can_delete = has_permission(&sess, &clinic, "stock:delete");
    let can_movement = has_permission(&sess, &clinic, "stock:movement") || can_write;

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar o estoque desta clínica." }
            }
        };
    }

    let mut active_tab = use_signal(|| StockTab::Materials);
    let mut search_query = use_signal(String::new);
    let mut reload_counter = use_signal(|| 0usize);
    let mut toast_msg = use_signal(|| None::<String>);

    let mut is_item_modal_open = use_signal(|| false);
    let mut editing_item = use_signal(|| None::<InventoryItem>);

    let mut is_movement_modal_open = use_signal(|| false);
    let mut movement_target_item = use_signal(|| None::<InventoryItem>);

    let mut is_delete_modal_open = use_signal(|| false);
    let mut delete_target_item = use_signal(|| None::<InventoryItem>);
    let mut is_deleting = use_signal(|| false);

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();
    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let cid_res = clinic_id.clone();
    let tok_res = token.clone();

    let stock_resource = use_resource(move || {
        let cid = cid_res.clone();
        let tok = tok_res.clone();
        let _ = reload_counter();

        async move {
            if cid.is_empty() || tok.is_empty() {
                return Err("Sessão inválida.".into());
            }
            fetch_stock_data(&tok, &cid, None, None).await
        }
    });

    let open_create_modal = move |_| {
        editing_item.set(None);
        is_item_modal_open.set(true);
    };

    let open_movement_modal_global = move |_| {
        movement_target_item.set(None);
        is_movement_modal_open.set(true);
    };

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();
    let mut handle_confirm_delete = move |_| {
        let Some(ref item) = delete_target_item() else {
            return;
        };
        let i_id = item.id.clone();
        let t = tok_del.clone();
        let c = cid_del.clone();
        let mut del_sig = delete_target_item;
        let mut rel_sig = reload_counter;
        let mut toast = toast_msg;
        let mut is_del = is_deleting;

        is_del.set(true);
        spawn(async move {
            match delete_stock_item(&t, &i_id, &c).await {
                Ok(_) => {
                    del_sig.set(None);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Item excluído com sucesso!".into()));
                }
                Err(e) => {
                    toast.set(Some(format!("Erro ao excluir item: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    rsx! {
        div { class: "stock-page-container",
            if let Some(ref err) = *toast_msg.read() {
                div { class: "toast-error",
                    span { "{err}" }
                    button { class: "toast-close-btn", onclick: move |_| toast_msg.set(None), "×" }
                }
            }

            div { class: "stock-top-bar",
                div { class: "stock-presets-group",
                    button {
                        class: if active_tab() == StockTab::Materials { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Materials),
                        IconBox { size: 14, color: "currentColor".to_string() }
                        span { " Materiais & Insumos" }
                    }
                    button {
                        class: if active_tab() == StockTab::Chemicals { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Chemicals),
                        IconFlask { size: 14, color: "currentColor".to_string() }
                        span { " Químicos & Medicamentos" }
                    }
                    button {
                        class: if active_tab() == StockTab::Equipments { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Equipments),
                        IconTool { size: 14, color: "currentColor".to_string() }
                        span { " Equipamentos & Patrimônio" }
                    }
                    button {
                        class: if active_tab() == StockTab::Alerts { "stock-preset-chip active tab-chip-alert" } else { "stock-preset-chip tab-chip-alert" },
                        onclick: move |_| active_tab.set(StockTab::Alerts),
                        IconAlertTriangle { size: 14, color: "currentColor".to_string() }
                        span { " Central de Alertas" }
                        if let Some(Ok(data)) = stock_resource.read().as_ref() {
                            if !data.alerts.is_empty() {
                                span { class: "alert-badge-count", "{data.alerts.len()}" }
                            }
                        }
                    }
                    button {
                        class: if active_tab() == StockTab::Movements { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Movements),
                        IconRefresh { size: 14, color: "currentColor".to_string() }
                        span { " Histórico de Movimentações" }
                    }
                }

                div { class: "header-actions-group",
                    if can_movement {
                        button { class: "btn-secondary", onclick: open_movement_modal_global,
                            IconRefresh { size: 16, color: "#1e293b".to_string() }
                            span { " Registrar Movimentação" }
                        }
                    }
                    if can_write {
                        button { class: "btn-primary", onclick: open_create_modal,
                            IconPlus { size: 16, color: "white".to_string() }
                            span { " Novo Item / Equipamento" }
                        }
                    }
                }
            }

            match stock_resource.read().as_ref() {
                None => rsx! {
                    div { class: "agenda-loading-box",
                        p { "Carregando inventário de estoque e patrimônio..." }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "agenda-error-box",
                        p { "{e}" }
                        button { class: "btn-secondary", onclick: move |_| reload_counter.set(reload_counter() + 1), "Tentar Novamente" }
                    }
                },
                Some(Ok(data)) => {
                    let kpis = &data.kpis;
                    let all_items = &data.items;
                    let alerts = &data.alerts;
                    let movements = &data.recent_movements;

                    let filtered_items: Vec<InventoryItem> = all_items.iter().filter(|item| {
                        match active_tab() {
                            StockTab::Materials => item.item_type == ItemType::Material,
                            StockTab::Chemicals => item.item_type == ItemType::Chemical,
                            StockTab::Equipments => item.item_type == ItemType::Equipment,
                            _ => true,
                        }
                    }).filter(|item| {
                        let query = search_query().to_lowercase();
                        if query.is_empty() { return true; }
                        item.name.to_lowercase().contains(&query)
                            || item.manufacturer.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || item.batch_number.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || item.serial_number.as_deref().unwrap_or("").to_lowercase().contains(&query)
                    }).cloned().collect();

                    let filtered_movements: Vec<StockMovement> = movements.iter().filter(|m| {
                        let query = search_query().to_lowercase();
                        if query.is_empty() { return true; }
                        m.notes.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || m.invoice_number.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || m.item_id.to_lowercase().contains(&query)
                    }).cloned().collect();

                    rsx! {
                        div { class: "stock-kpi-row",
                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Total de Itens Cadastrados" }
                                    span { class: "stock-kpi-badge badge-blue", "Ativos" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value", "{kpis.total_items_count}" }
                                    div { class: "stock-kpi-icon-box bg-blue",
                                        IconBox { size: 20, color: "#0052cc".to_string() }
                                    }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "{kpis.materials_count} materiais, {kpis.chemicals_count} químicos" }
                                }
                            }

                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Valor Total em Estoque" }
                                    span { class: "stock-kpi-badge badge-green", "Patrimônio" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value", "{format_currency(kpis.total_inventory_value_cents)}" }
                                    div { class: "stock-kpi-icon-box bg-green",
                                        IconBox { size: 20, color: "#10b981".to_string() }
                                    }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Avaliação de custo unitário médio" }
                                }
                            }

                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Alertas de Reposição" }
                                    span { class: "stock-kpi-badge badge-amber", "Estoque Baixo" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value kpi-val-amber", "{kpis.low_stock_alerts_count}" }
                                    div { class: "stock-kpi-icon-box bg-amber",
                                        IconAlertTriangle { size: 20, color: "#f59e0b".to_string() }
                                    }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Abaixo do ponto de pedido mínimo" }
                                }
                            }

                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Validade & Vencimento" }
                                    span { class: "stock-kpi-badge badge-red", "Crítico" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value kpi-val-red", "{kpis.expiring_alerts_count}" }
                                    div { class: "stock-kpi-icon-box bg-red",
                                        IconFlask { size: 20, color: "#ef4444".to_string() }
                                    }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Vencendo nos próximos 30 dias" }
                                }
                            }
                        }

                        div { class: "stock-filter-row",
                            div { class: "stock-search-bar",
                                div { class: "search-icon", IconSearch { size: 16, color: "currentColor".to_string() } }
                                input {
                                    class: "search-input",
                                    placeholder: "Filtrar por nome, lote, fabricante, série...",
                                    value: "{search_query}",
                                    oninput: move |e| search_query.set(e.value())
                                }
                            }
                        }

                        match active_tab() {
                            StockTab::Alerts => rsx! {
                                StockAlertsSection { alerts: alerts.clone() }
                            },
                            StockTab::Movements => rsx! {
                                StockMovementsSection { movements: filtered_movements }
                            },
                            _ => rsx! {
                                StockItemsSection {
                                    items: filtered_items,
                                    can_write,
                                    can_delete,
                                    can_movement,
                                    on_movement: move |item: InventoryItem| {
                                        movement_target_item.set(Some(item));
                                        is_movement_modal_open.set(true);
                                    },
                                    on_edit: move |item: InventoryItem| {
                                        editing_item.set(Some(item));
                                        is_item_modal_open.set(true);
                                    },
                                    on_delete: move |item: InventoryItem| {
                                        delete_target_item.set(Some(item));
                                        is_delete_modal_open.set(true);
                                    },
                                }
                            }
                        }
                    }
                }
            }

            if is_item_modal_open() {
                StockItemModal {
                    is_open: is_item_modal_open,
                    editing_item: editing_item(),
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    reload_counter,
                    toast_msg,
                }
            }

            if is_movement_modal_open() {
                {
                    let all_stock_items = match stock_resource.read().as_ref() {
                        Some(Ok(data)) => data.items.clone(),
                        _ => vec![],
                    };
                    rsx! {
                        StockMovementModal {
                            is_open: is_movement_modal_open,
                            target_item: movement_target_item(),
                            items: all_stock_items,
                            token: token.clone(),
                            clinic_id: clinic_id.clone(),
                            reload_counter,
                            toast_msg,
                        }
                    }
                }
            }

            if is_delete_modal_open() {
                if let Some(ref item) = delete_target_item() {
                    div { class: "modal-overlay",
                        div { class: "action-modal delete-modal-card",
                            div { class: "modal-header",
                                h2 { class: "modal-title text-danger", "Excluir Item de Estoque" }
                                button { class: "modal-close", onclick: move |_| is_delete_modal_open.set(false), "×" }
                            }
                            div { class: "modal-body",
                                p { "Tem certeza que deseja excluir o item ", strong { "{item.name}" }, "?" }
                                p { class: "text-muted font-xs mt-2", "Esta ação não pode ser desfeita e removerá o histórico do item." }
                            }
                            div { class: "modal-footer-actions",
                                button { class: "btn-secondary", onclick: move |_| is_delete_modal_open.set(false), "Cancelar" }
                                button {
                                    class: "btn-danger",
                                    disabled: is_deleting(),
                                    onclick: move |e| handle_confirm_delete(e),
                                    if is_deleting() { "Excluindo..." } else { "Confirmar Exclusão" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
