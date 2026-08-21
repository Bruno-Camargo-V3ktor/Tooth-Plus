//! # Módulo de Gestão de Estoque e Patrimônio (Frontend)
//!
//! Controla materiais de consumo, fármacos/anestésicos, equipamentos,
//! alertas inteligentes de validade/manutenção e histórico de movimentações.

pub mod item_modal;
pub mod items_table;
pub mod movement_modal;

pub use item_modal::*;
pub use items_table::*;
pub use movement_modal::*;

use crate::api::{delete_stock_item, fetch_stock_data};
use crate::components::icons::{
    IconAlertTriangle, IconBox, IconFlask, IconPlus, IconRefresh, IconSearch, IconTool,
};
use crate::permissions::has_permission;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::stock::{InventoryItem, ItemType, StockMovement};

/// Abas de navegação do módulo de Estoque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockTab {
    Materials,
    Chemicals,
    Equipments,
    Alerts,
    Movements,
}

/// Formata valor em centavos para moeda BRL com separador de milhar.
fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;

    let reals_str = reals.to_string();
    let mut formatted_reals = String::new();
    let len = reals_str.len();
    for (i, ch) in reals_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_reals.push('.');
        }
        formatted_reals.push(ch);
    }

    if is_negative {
        format!("- R$ {},{:02}", formatted_reals, centavos)
    } else {
        format!("R$ {},{:02}", formatted_reals, centavos)
    }
}

#[component]
pub fn StockView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = has_permission(&sess, &clinic, "stock:read");
    let can_write = has_permission(&sess, &clinic, "stock:write");
    let can_delete = has_permission(&sess, &clinic, "stock:delete");
    let can_movement = has_permission(&sess, &clinic, "stock:movement");


    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para visualizar o estoque desta clínica." }
            }
        };
    }

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();
    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let mut active_tab = use_signal(|| StockTab::Materials);
    let mut search_query = use_signal(String::new);

    let mut is_item_modal_open = use_signal(|| false);
    let mut editing_item = use_signal(|| None::<InventoryItem>);

    let mut is_movement_modal_open = use_signal(|| false);
    let mut movement_target_item = use_signal(|| None::<InventoryItem>);

    let mut is_delete_modal_open = use_signal(|| false);
    let mut delete_target_item = use_signal(|| None::<InventoryItem>);
    let mut is_deleting = use_signal(|| false);

    let mut reload_counter = use_signal(|| 0);
    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let stock_resource = use_resource(move || {
        let t = tok.clone();
        let c = cid.clone();
        let _rel = reload_counter();
        async move {
            if t.is_empty() || c.is_empty() {
                return None;
            }
            fetch_stock_data(&t, &c, None, None).await.ok()
        }
    });

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();

    let mut handle_confirm_delete = move |_| {
        let Some(ref item) = *delete_target_item.read() else {
            return;
        };
        let item_id = item.id.clone();
        let t = tok_del.clone();
        let c = cid_del.clone();
        let mut open_sig = is_delete_modal_open;
        let mut rel_sig = reload_counter;
        let mut is_del = is_deleting;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        is_del.set(true);
        spawn(async move {
            match delete_stock_item(&t, &c, &item_id).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Item excluído!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir item: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    rsx! {
        div { class: "documents-view-container",
            if let Some(ref msg) = *toast_msg.read() {
                div { class: "toast toast-success",
                    span { "{msg}" }
                    button { class: "toast-close", onclick: move |_| toast_msg.set(None), "✕" }
                }
            }
            if let Some(ref err) = *error_toast.read() {
                div { class: "toast toast-error",
                    span { "{err}" }
                    button { class: "toast-close", onclick: move |_| error_toast.set(None), "✕" }
                }
            }

            match &*stock_resource.read() {
                None => rsx! {
                    div { class: "loading-card",
                        div { class: "loading-spinner" }
                        p { "Carregando inventário de estoque e patrimônio..." }
                    }
                },
                Some(None) => rsx! {
                    div { class: "empty-state-card",
                        IconBox { size: 48, color: "#94a3b8".to_string() }
                        h3 { "Falha ao carregar estoque" }
                        p { "Verifique sua conexão ou privilégios de acesso." }
                    }
                },
                Some(Some(data)) => {
                    let kpis = &data.kpis;
                    let items = &data.items;
                    let alerts = &data.alerts;
                    let movements = &data.recent_movements;

                    let total_alerts = alerts.len();

                    let materials_count = items.iter().filter(|i| i.item_type == ItemType::Material).count();
                    let chemicals_count = items.iter().filter(|i| i.item_type == ItemType::Chemical).count();
                    let equipments_count = items.iter().filter(|i| i.item_type == ItemType::Equipment).count();
                    let movements_count = movements.len();

                    let q = search_query().trim().to_lowercase();
                    let filtered_items: Vec<InventoryItem> = items
                        .iter()
                        .filter(|i| {
                            let match_tab = match active_tab() {
                                StockTab::Materials => i.item_type == ItemType::Material,
                                StockTab::Chemicals => i.item_type == ItemType::Chemical,
                                StockTab::Equipments => i.item_type == ItemType::Equipment,
                                _ => true,
                            };

                            let match_search = if q.is_empty() {
                                true
                            } else {
                                i.name.to_lowercase().contains(&q)
                                    || i.manufacturer.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                    || i.batch_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                    || i.serial_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                            };

                            match_tab && match_search
                        })
                        .cloned()
                        .collect();

                    let filtered_movements: Vec<StockMovement> = movements
                        .iter()
                        .filter(|m| {
                            if q.is_empty() {
                                true
                            } else {
                                m.item_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                    || m.invoice_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
                                    || m.notes.as_deref().unwrap_or("").to_lowercase().contains(&q)
                            }
                        })
                        .cloned()
                        .collect();

                    rsx! {
                        // 1. Main Tabs Switcher (Igual ao Documentos)
                        div { class: "documents-tab-bar",
                            button {
                                class: if active_tab() == StockTab::Materials { "doc-main-tab active" } else { "doc-main-tab" },
                                onclick: move |_| active_tab.set(StockTab::Materials),
                                IconBox { size: 16, color: "currentColor".to_string() }
                                span { " Materiais & Insumos ({materials_count})" }
                            }
                            button {
                                class: if active_tab() == StockTab::Chemicals { "doc-main-tab active" } else { "doc-main-tab" },
                                onclick: move |_| active_tab.set(StockTab::Chemicals),
                                IconFlask { size: 16, color: "currentColor".to_string() }
                                span { " Químicos & Medicamentos ({chemicals_count})" }
                            }
                            button {
                                class: if active_tab() == StockTab::Equipments { "doc-main-tab active" } else { "doc-main-tab" },
                                onclick: move |_| active_tab.set(StockTab::Equipments),
                                IconTool { size: 16, color: "currentColor".to_string() }
                                span { " Equipamentos & Patrimônio ({equipments_count})" }
                            }
                            button {
                                class: if active_tab() == StockTab::Alerts { "doc-main-tab active" } else { "doc-main-tab" },
                                onclick: move |_| active_tab.set(StockTab::Alerts),
                                IconAlertTriangle { size: 16, color: "currentColor".to_string() }
                                span { " Central de Alertas ({total_alerts})" }
                            }
                            button {
                                class: if active_tab() == StockTab::Movements { "doc-main-tab active" } else { "doc-main-tab" },
                                onclick: move |_| active_tab.set(StockTab::Movements),
                                IconRefresh { size: 16, color: "currentColor".to_string() }
                                span { " Histórico de Movimentações ({movements_count})" }
                            }
                        }

                        // 2. Compact Horizontal KPIs com Informações Financeiras e Métricas Completas
                        div { class: "agenda-kpi-row",
                            // 1. PATRIMÔNIO TOTAL (VALOR EM R$)
                            div { class: "agenda-kpi-card",
                                div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                                    IconBox { size: 16, color: "currentColor".to_string() }
                                }
                                div { class: "agenda-kpi-text-col",
                                    span { class: "agenda-kpi-lbl", "Patrimônio Total" }
                                    span { class: "agenda-kpi-sublbl", "{kpis.total_items_count} itens em estoque" }
                                }
                                div { class: "agenda-kpi-val", "{format_currency(kpis.total_inventory_value_cents)}" }
                            }

                            // 2. INSUMOS & QUÍMICOS
                            div { class: "agenda-kpi-card",
                                div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                                    IconFlask { size: 16, color: "currentColor".to_string() }
                                }
                                div { class: "agenda-kpi-text-col",
                                    span { class: "agenda-kpi-lbl", "Insumos & Químicos" }
                                    span { class: "agenda-kpi-sublbl", "{kpis.materials_count} M / {kpis.chemicals_count} Q" }
                                }
                                div { class: "agenda-kpi-val kpi-pending", "{kpis.materials_count + kpis.chemicals_count}" }
                            }

                            // 3. EQUIPAMENTOS
                            div { class: "agenda-kpi-card",
                                div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                                    IconTool { size: 16, color: "currentColor".to_string() }
                                }
                                div { class: "agenda-kpi-text-col",
                                    span { class: "agenda-kpi-lbl", "Equipamentos" }
                                    span { class: "agenda-kpi-sublbl", "Patrimônio ativo" }
                                }
                                div { class: "agenda-kpi-val kpi-completed", "{kpis.equipments_count}" }
                            }

                            // 4. CENTRAL DE ALERTAS
                            div { class: "agenda-kpi-card",
                                div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                                    IconAlertTriangle { size: 16, color: "currentColor".to_string() }
                                }
                                div { class: "agenda-kpi-text-col",
                                    span { class: "agenda-kpi-lbl", "Central de Alertas" }
                                    span { class: "agenda-kpi-sublbl", "{kpis.low_stock_alerts_count} Baixo • {kpis.expiring_alerts_count} Venc. • {kpis.maintenance_alerts_count} Manut." }
                                }
                                div { class: "agenda-kpi-val kpi-progress", "{total_alerts}" }
                            }
                        }

                        // 3. View Toolbar (Busca à esquerda, Ações à direita - Igual ao Documentos)
                        div { class: "view-toolbar",
                            div { class: "search-input-wrap",
                                IconSearch { size: 18, color: "#94a3b8".to_string() }
                                input {
                                    r#type: "text",
                                    class: "search-input",
                                    placeholder: "Buscar por nome, fabricante, lote, nº de série ou nota fiscal...",
                                    value: "{search_query}",
                                    oninput: move |e| search_query.set(e.value()),
                                }
                            }

                            div { class: "toolbar-actions",
                                button {
                                    class: "btn-refresh",
                                    onclick: move |_| reload_counter.set(reload_counter() + 1),
                                    title: "Recarregar estoque",
                                    IconRefresh { size: 16, color: "#475569".to_string() }
                                }

                                if can_movement {
                                    button {
                                        class: "btn-secondary",
                                        onclick: move |_| {
                                            movement_target_item.set(None);
                                            is_movement_modal_open.set(true);
                                        },
                                        IconRefresh { size: 16, color: "currentColor".to_string() }
                                        span { " Registrar Movimentação" }
                                    }
                                }

                                if can_write {
                                    button {
                                        class: "btn-primary",
                                        onclick: move |_| {
                                            editing_item.set(None);
                                            is_item_modal_open.set(true);
                                        },
                                        IconPlus { size: 16, color: "currentColor".to_string() }
                                        span { " Novo Item / Equipamento" }
                                    }
                                }
                            }
                        }

                        // 4. Conteúdo da Aba Ativa
                        match active_tab() {
                            StockTab::Alerts => rsx! {
                                StockAlertsSection {
                                    alerts: alerts.clone(),
                                    items: items.clone(),
                                    on_resolve: move |target: InventoryItem| {
                                        movement_target_item.set(Some(target));
                                        is_movement_modal_open.set(true);
                                    }
                                }
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
                    error_toast,
                }
            }

            if is_movement_modal_open() {
                {
                    let all_stock_items = match stock_resource.read().as_ref() {
                        Some(Some(data)) => data.items.clone(),
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
                            error_toast,
                        }
                    }
                }
            }

            if is_delete_modal_open() {
                if let Some(ref item) = *delete_target_item.read() {
                    div { class: "modal-overlay",
                        div { class: "action-modal modal-small delete-modal-card",
                            div { class: "settings-header",
                                h2 { class: "settings-title text-danger", "Excluir Item de Estoque" }
                                button { class: "close-btn", onclick: move |_| is_delete_modal_open.set(false), "×" }
                            }
                            div { class: "settings-content",
                                div { class: "fin-delete-info-card",
                                    div { class: "fin-delete-desc", "{item.name}" }
                                    div { class: "fin-delete-val", "{item.current_stock} {item.unit_type}" }
                                }
                                div { class: "alert-banner alert-warning mt-3",
                                    span { "Atenção: Esta ação removerá o item e seu histórico de estoque permanentemente." }
                                }
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
