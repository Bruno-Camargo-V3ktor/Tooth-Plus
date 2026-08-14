use crate::api;
use crate::components::icons::{
    IconAlertTriangle, IconArrowDown, IconArrowUp, IconBox, IconCheck, IconEdit, IconExternalLink,
    IconFile, IconFlask, IconPaperclip, IconPlus, IconRefresh, IconSearch, IconTool, IconTrash,
    IconUpload,
};
use crate::permissions::has_permission;
use crate::{ActiveClinicState, SessionState};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use shared::files::FileUploadRequest;
use shared::stock::{
    CreateInventoryItemRequest, CreateStockMovementRequest, EquipmentStatus, InventoryItem,
    ItemType, MovementType, StockAlertSeverity, StockAlertType, StockMovement,
    UpdateInventoryItemRequest,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StockTab {
    Materials,
    Chemicals,
    Equipments,
    Alerts,
    Movements,
}

fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;

    let reals_str = reals.to_string();
    let mut formatted_reals = String::new();
    let len = reals_str.len();

    for (i, c) in reals_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_reals.push('.');
        }
        formatted_reals.push(c);
    }

    if is_negative {
        format!("-R$ {:0>1},{:02}", formatted_reals, centavos)
    } else {
        format!("R$ {:0>1},{:02}", formatted_reals, centavos)
    }
}

fn parse_currency_input(input: &str) -> i64 {
    let cleaned: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
    cleaned.parse::<i64>().unwrap_or(0)
}

fn format_date_br(date_str: &str) -> String {
    crate::utils::format_date_br(date_str)
}

fn format_datetime_br(date_str: &str) -> String {
    crate::utils::format_datetime_br(date_str)
}

fn extract_filename(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or("documento.pdf")
        .to_string()
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
    let can_movement = has_permission(&sess, &clinic, "stock:movement") || can_write;

    if !can_read {
        return rsx! {
            div { class: "access-denied-container",
                div { class: "access-denied-card",
                    h2 { "Acesso Restrito" }
                    p { "Você não possui privilégios de acesso para visualizar o estoque desta clínica." }
                }
            }
        };
    }

    let mut active_tab = use_signal(|| StockTab::Materials);
    let mut search_query = use_signal(|| String::new());
    let mut reload_counter = use_signal(|| 0);
    let mut action_error = use_signal(|| None::<String>);

    let mut is_item_modal_open = use_signal(|| false);
    let mut editing_item = use_signal(|| None::<InventoryItem>);

    let mut is_movement_modal_open = use_signal(|| false);
    let mut movement_target_item = use_signal(|| None::<InventoryItem>);

    let mut is_delete_modal_open = use_signal(|| false);
    let mut delete_target_item = use_signal(|| None::<InventoryItem>);

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
            api::fetch_stock_data(&tok, &cid, None, None).await
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

    rsx! {
        div { class: "stock-page-container",
            if let Some(err) = action_error() {
                div { class: "toast-error",
                    span { "{err}" }
                    button { class: "toast-close-btn", onclick: move |_| action_error.set(None), "×" }
                }
            }

            div { class: "stock-top-bar",
                div { class: "stock-presets-group",
                    button {
                        class: if active_tab() == StockTab::Materials { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Materials),
                        IconBox { size: 14, color: "currentColor".to_string() }
                        span { "Materiais & Insumos" }
                    }
                    button {
                        class: if active_tab() == StockTab::Chemicals { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Chemicals),
                        IconFlask { size: 14, color: "currentColor".to_string() }
                        span { "Químicos & Medicamentos" }
                    }
                    button {
                        class: if active_tab() == StockTab::Equipments { "stock-preset-chip active" } else { "stock-preset-chip" },
                        onclick: move |_| active_tab.set(StockTab::Equipments),
                        IconTool { size: 14, color: "currentColor".to_string() }
                        span { "Equipamentos & Patrimônio" }
                    }
                    button {
                        class: if active_tab() == StockTab::Alerts { "stock-preset-chip active tab-chip-alert" } else { "stock-preset-chip tab-chip-alert" },
                        onclick: move |_| active_tab.set(StockTab::Alerts),
                        IconAlertTriangle { size: 14, color: "currentColor".to_string() }
                        span { "Central de Alertas" }
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
                        span { "Histórico de Movimentações" }
                    }
                }

                div { class: "header-actions-group",
                    if can_movement {
                        button { class: "btn-secondary", onclick: open_movement_modal_global,
                            IconRefresh { size: 16, color: "#1e293b".to_string() }
                            span { "Registrar Movimentação" }
                        }
                    }
                    if can_write {
                        button { class: "btn-primary", onclick: open_create_modal,
                            IconPlus { size: 16, color: "white".to_string() }
                            span { "Novo Item / Equipamento" }
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

                    let filtered_items: Vec<&InventoryItem> = all_items.iter().filter(|item| {
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
                    }).collect();

                    let filtered_movements: Vec<&StockMovement> = movements.iter().filter(|m| {
                        let query = search_query().to_lowercase();
                        if query.is_empty() { return true; }
                        m.notes.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || m.invoice_number.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || m.item_id.to_lowercase().contains(&query)
                    }).collect();

                    rsx! {
                        div { class: "stock-kpi-row",
                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Patrimônio Total" }
                                    span { class: "stock-kpi-badge badge-neutral", "{kpis.total_items_count} Itens" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value text-primary", "{format_currency(kpis.total_inventory_value_cents)}" }
                                    div { class: "stock-kpi-icon-box icon-primary-box", IconBox { size: 18, color: "currentColor".to_string() } }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Valor investido em estoque ativo" }
                                }
                            }

                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Insumos & Químicos" }
                                    span { class: "stock-kpi-badge badge-neutral", "{kpis.materials_count + kpis.chemicals_count} Cadastrados" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value", "{kpis.materials_count} M / {kpis.chemicals_count} Q" }
                                    div { class: "stock-kpi-icon-box icon-chemical-box", IconFlask { size: 18, color: "currentColor".to_string() } }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Materiais de consumo e farmácia" }
                                }
                            }

                            div { class: "stock-kpi-card",
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Equipamentos" }
                                    span { class: "stock-kpi-badge badge-neutral", "{kpis.equipments_count} Ativos" }
                                }
                                div { class: "stock-kpi-body",
                                    div { class: "stock-kpi-value", "{kpis.equipments_count}" }
                                    div { class: "stock-kpi-icon-box icon-equipment-box", IconTool { size: 18, color: "currentColor".to_string() } }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Aparelhos, compressores e autoclaves" }
                                }
                            }

                            div {
                                class: if !alerts.is_empty() { "stock-kpi-card card-alert-active" } else { "stock-kpi-card" },
                                div { class: "stock-kpi-header",
                                    span { class: "stock-kpi-title", "Central de Alertas" }
                                    span {
                                        class: if !alerts.is_empty() { "stock-kpi-badge badge-alert-warn" } else { "stock-kpi-badge badge-neutral" },
                                        "{alerts.len()} Alertas"
                                    }
                                }
                                div { class: "stock-kpi-body",
                                    div {
                                        class: if !alerts.is_empty() { "stock-kpi-value text-alert" } else { "stock-kpi-value" },
                                        "{kpis.low_stock_alerts_count} Baixo / {kpis.expiring_alerts_count} Venc. / {kpis.maintenance_alerts_count} Manut."
                                    }
                                    div { class: "stock-kpi-icon-box icon-alert-box", IconAlertTriangle { size: 18, color: "currentColor".to_string() } }
                                }
                                div { class: "stock-kpi-footer",
                                    span { "Reposição, validade e manutenção" }
                                }
                            }
                        }

                        div { class: "stock-controls-toolbar",
                            div { class: "stock-search-wrapper",
                                span { class: "stock-search-icon", IconSearch { size: 16, color: "#94a3b8".to_string() } }
                                input {
                                    class: "stock-search-input",
                                    placeholder: "Buscar por nome, fabricante, lote, n° de série ou nota fiscal...",
                                    value: "{search_query()}",
                                    oninput: move |e| search_query.set(e.value())
                                }
                            }
                        }

                        if active_tab() == StockTab::Alerts {
                            if alerts.is_empty() {
                                div { class: "stock-empty-state",
                                    div { class: "empty-icon-box success-icon-box", IconCheck { size: 28, color: "#059669".to_string() } }
                                    p { class: "empty-title", "Nenhum alerta pendente no momento!" }
                                    p { class: "empty-subtitle", "Todos os níveis de estoque estão regulares, sem produtos vencidos ou manutenções atrasadas." }
                                }
                            } else {
                                div { class: "stock-alerts-list",
                                    for alert in alerts {
                                        div {
                                            class: match alert.severity {
                                                StockAlertSeverity::Critical => "stock-alert-card alert-critical",
                                                StockAlertSeverity::Warning => "stock-alert-card alert-warning",
                                                StockAlertSeverity::Info => "stock-alert-card alert-info",
                                            },
                                            key: "{alert.id}",
                                            div { class: "alert-card-left",
                                                div { class: "alert-icon-wrapper",
                                                    IconAlertTriangle { size: 20, color: "currentColor".to_string() }
                                                }
                                                div { class: "alert-info-group",
                                                    div { class: "alert-title-row",
                                                        span { class: "alert-card-title", "{alert.title}" }
                                                        span { class: "alert-badge-tag",
                                                            match alert.alert_type {
                                                                StockAlertType::LowStock => "Estoque Baixo",
                                                                StockAlertType::ExpiringSoon => "Validade Próxima",
                                                                StockAlertType::Expired => "Vencido",
                                                                StockAlertType::MaintenanceDue => "Manutenção Próxima",
                                                                StockAlertType::MaintenanceOverdue => "Manutenção Atrasada",
                                                            }
                                                        }
                                                    }
                                                    p { class: "alert-message", "{alert.message}" }
                                                }
                                            }
                                            div { class: "alert-card-right",
                                                div { class: "alert-target-pill",
                                                    span { class: "target-label", "Meta/Ref:" }
                                                    span { class: "target-val", "{alert.target_value}" }
                                                }
                                                if can_movement {
                                                    button {
                                                        class: "btn-secondary btn-sm",
                                                        onclick: {
                                                            let item_match = all_items.iter().find(|i| i.id == alert.item_id).cloned();
                                                            move |_| {
                                                                movement_target_item.set(item_match.clone());
                                                                is_movement_modal_open.set(true);
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
                        } else if active_tab() == StockTab::Movements {
                            if filtered_movements.is_empty() {
                                div { class: "stock-empty-state",
                                    p { class: "empty-title", "Nenhuma movimentação registrada no histórico." }
                                    p { class: "empty-subtitle", "Clique em 'Registrar Movimentação' para lançar compras, entradas ou baixas manuais." }
                                }
                            } else {
                                div { class: "stock-ledger-container",
                                    div { class: "ledger-table-header",
                                        div { class: "col-mov-date", "Data / Hora" }
                                        div { class: "col-mov-type", "Tipo" }
                                        div { class: "col-mov-item", "Item / Identificador" }
                                        div { class: "col-mov-qty", "Quantidade" }
                                        div { class: "col-mov-cost", "Custo Unitário" }
                                        div { class: "col-mov-notes", "Documento / Observação" }
                                    }
                                    for mov in filtered_movements {
                                        {
                                            let is_in = mov.quantity_change > 0;
                                            let item_lookup = all_items.iter().find(|i| i.id == mov.item_id);
                                            let display_name = item_lookup.map(|i| i.name.as_str()).unwrap_or(&mov.item_id);
                                            let unit_name = item_lookup.map(|i| i.unit_type.as_str()).unwrap_or("un");
                                            let qty_text = if is_in {
                                                format!("+{} {}", mov.quantity_change, unit_name)
                                            } else {
                                                format!("{} {}", mov.quantity_change, unit_name)
                                            };

                                            rsx! {
                                                div { class: "ledger-row-item", key: "{mov.id}",
                                                    div { class: "col-mov-date", "{format_datetime_br(&mov.created_at)}" }
                                                    div { class: "col-mov-type",
                                                        span {
                                                            class: if is_in { "mov-badge badge-in" } else { "mov-badge badge-out" },
                                                            match mov.movement_type {
                                                                MovementType::PurchaseIn => "Entrada / Compra",
                                                                MovementType::ManualOut => "Saída Manual",
                                                                MovementType::AppointmentOut => "Consumo Agenda",
                                                                MovementType::Adjustment => "Ajuste de Saldo",
                                                                MovementType::Loss => "Avaria / Perda",
                                                            }
                                                        }
                                                    }
                                                    div { class: "col-mov-item font-semibold", "{display_name}" }
                                                    div {
                                                        class: if is_in { "col-mov-qty text-income font-bold" } else { "col-mov-qty text-expense font-bold" },
                                                        "{qty_text}"
                                                    }
                                                    div { class: "col-mov-cost",
                                                        if let Some(c) = mov.unit_cost_cents {
                                                            "{format_currency(c)}"
                                                        } else {
                                                            "-"
                                                        }
                                                    }
                                                    div { class: "col-mov-notes",
                                                        if let Some(ref nf) = mov.invoice_number {
                                                            span { class: "nf-tag", "NF: {nf}" }
                                                        }
                                                        span { "{mov.notes.as_deref().unwrap_or(\"-\")}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if filtered_items.is_empty() {
                                div { class: "stock-empty-state",
                                    p { class: "empty-title", "Nenhum item cadastrado nesta categoria." }
                                    p { class: "empty-subtitle", "Clique em 'Novo Item / Equipamento' no canto superior direito para adicionar." }
                                }
                            } else {
                                div { class: "stock-items-grid",
                                    for item in filtered_items {
                                        {
                                            let is_low = item.item_type != ItemType::Equipment && item.current_stock <= item.min_stock;
                                            let has_exp_warning = item.expiration_date.as_ref().map(|d| {
                                                if let Ok(dt) = DateTime::parse_from_rfc3339(d) {
                                                    dt.with_timezone(&Utc) < (Utc::now() + chrono::Duration::days(30))
                                                } else { false }
                                            }).unwrap_or(false);
                                            let pct = if item.min_stock > 0 {
                                                ((item.current_stock as f32 / (item.min_stock * 2) as f32) * 100.0).clamp(5.0, 100.0) as i32
                                            } else { 100 };

                                            rsx! {
                                                div { class: "stock-item-card", key: "{item.id}",
                                                    div { class: "item-card-top",
                                                        div { class: "item-type-badge",
                                                            match item.item_type {
                                                                ItemType::Material => rsx! { span { class: "tag-material", "Material" } },
                                                                ItemType::Chemical => rsx! { span { class: "tag-chemical", "Químico / Farmácia" } },
                                                                ItemType::Equipment => rsx! { span { class: "tag-equipment", "Equipamento" } },
                                                            }
                                                            if is_low {
                                                                span { class: "stock-alert-pill pill-danger", "Estoque Baixo" }
                                                            }
                                                            if has_exp_warning {
                                                                span { class: "stock-alert-pill pill-warning", "Validade Próxima" }
                                                            }
                                                            if let Some(st) = item.equipment_status {
                                                                match st {
                                                                    EquipmentStatus::Active => rsx! { span { class: "stock-alert-pill pill-success", "Operacional" } },
                                                                    EquipmentStatus::InMaintenance => rsx! { span { class: "stock-alert-pill pill-warning", "Em Revisão" } },
                                                                    EquipmentStatus::Broken => rsx! { span { class: "stock-alert-pill pill-danger", "Inoperante" } },
                                                                }
                                                            }
                                                        }

                                                        div { class: "item-actions-menu",
                                                            if can_movement {
                                                                button {
                                                                    class: "item-action-icon-btn",
                                                                    title: "Movimentar Estoque",
                                                                    onclick: {
                                                                        let it = item.clone();
                                                                        move |_| {
                                                                            movement_target_item.set(Some(it.clone()));
                                                                            is_movement_modal_open.set(true);
                                                                        }
                                                                    },
                                                                    IconRefresh { size: 15, color: "#475569".to_string() }
                                                                }
                                                            }
                                                            if can_write {
                                                                button {
                                                                    class: "item-action-icon-btn",
                                                                    title: "Editar Item",
                                                                    onclick: {
                                                                        let it = item.clone();
                                                                        move |_| {
                                                                            editing_item.set(Some(it.clone()));
                                                                            is_item_modal_open.set(true);
                                                                        }
                                                                    },
                                                                    IconEdit { size: 15, color: "#475569".to_string() }
                                                                }
                                                            }
                                                            if can_delete {
                                                                button {
                                                                    class: "item-action-icon-btn btn-danger-icon",
                                                                    title: "Excluir Item",
                                                                    onclick: {
                                                                        let it = item.clone();
                                                                        move |_| {
                                                                            delete_target_item.set(Some(it.clone()));
                                                                            is_delete_modal_open.set(true);
                                                                        }
                                                                    },
                                                                    IconTrash { size: 15, color: "#dc2626".to_string() }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    div { class: "item-card-main-info",
                                                        h3 { class: "item-title", "{item.name}" }
                                                        if let Some(ref mfg) = item.manufacturer {
                                                            span { class: "item-manufacturer", "Fabricante: {mfg}" }
                                                        }
                                                    }

                                                    div { class: "item-stock-meter-box",
                                                        div { class: "meter-labels-row",
                                                            span { class: "stock-cur-val",
                                                                strong { "{item.current_stock}" }
                                                                span { " {item.unit_type}" }
                                                            }
                                                            if item.item_type != ItemType::Equipment {
                                                                span { class: "stock-min-label", "Mínimo: {item.min_stock} {item.unit_type}" }
                                                            }
                                                        }
                                                        if item.item_type != ItemType::Equipment {
                                                            div { class: "stock-progress-track",
                                                                div {
                                                                    class: if is_low { "stock-progress-fill fill-low" } else { "stock-progress-fill fill-ok" },
                                                                    style: "width: {pct}%;"
                                                                }
                                                            }
                                                        }
                                                    }

                                                    div { class: "item-details-metadata",
                                                        div { class: "meta-item",
                                                            span { class: "meta-k", "Custo Unitário:" }
                                                            span { class: "meta-v", "{format_currency(item.cost_price_cents)}" }
                                                        }
                                                        if let Some(ref lot) = item.batch_number {
                                                            div { class: "meta-item",
                                                                span { class: "meta-k", "Lote:" }
                                                                span { class: "meta-v", "{lot}" }
                                                            }
                                                        }
                                                        if let Some(ref exp) = item.expiration_date {
                                                            div { class: "meta-item",
                                                                span { class: "meta-k", "Validade:" }
                                                                span { class: "meta-v font-bold", "{format_date_br(exp)}" }
                                                            }
                                                        }
                                                        if let Some(ref sn) = item.serial_number {
                                                            div { class: "meta-item",
                                                                span { class: "meta-k", "N° de Série:" }
                                                                span { class: "meta-v", "{sn}" }
                                                            }
                                                        }
                                                        if let Some(ref maint) = item.next_maintenance_date {
                                                            div { class: "meta-item",
                                                                span { class: "meta-k", "Próx. Manutenção:" }
                                                                span { class: "meta-v font-bold", "{format_date_br(maint)}" }
                                                            }
                                                        }
                                                        if let Some(ref war) = item.warranty_until {
                                                            div { class: "meta-item",
                                                                span { class: "meta-k", "Garantia até:" }
                                                                span { class: "meta-v", "{format_date_br(war)}" }
                                                            }
                                                        }
                                                        if !item.attachments.is_empty() {
                                                            div { class: "meta-item",
                                                                span { class: "meta-k", "Documentos / NF:" }
                                                                div { class: "flex items-center gap-1 flex-wrap",
                                                                    for (idx, doc_url) in item.attachments.iter().enumerate() {
                                                                        a {
                                                                            class: "item-docs-pill",
                                                                            href: "{doc_url}",
                                                                            target: "_blank",
                                                                            IconPaperclip { size: 12, color: "currentColor".to_string() }
                                                                            span { "Doc #{idx + 1}" }
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
                        }
                    }
                }
            }

            if is_item_modal_open() {
                InventoryItemModal {
                    item: editing_item(),
                    clinic_id: clinic_id.clone(),
                    token: token.clone(),
                    on_close: move |_| {
                        is_item_modal_open.set(false);
                        editing_item.set(None);
                    },
                    on_saved: move |_| {
                        is_item_modal_open.set(false);
                        editing_item.set(None);
                        reload_counter.set(reload_counter() + 1);
                    }
                }
            }

            if is_movement_modal_open() {
                if let Some(Ok(data)) = stock_resource.read().as_ref() {
                    StockMovementModal {
                        all_items: data.items.clone(),
                        target_item: movement_target_item(),
                        clinic_id: clinic_id.clone(),
                        token: token.clone(),
                        on_close: move |_| {
                            is_movement_modal_open.set(false);
                            movement_target_item.set(None);
                        },
                        on_saved: move |_| {
                            is_movement_modal_open.set(false);
                            movement_target_item.set(None);
                            reload_counter.set(reload_counter() + 1);
                        }
                    }
                }
            }

            if is_delete_modal_open() {
                if let Some(target) = delete_target_item() {
                    DeleteConfirmModal {
                        item: target,
                        clinic_id: clinic_id.clone(),
                        token: token.clone(),
                        on_close: move |_| {
                            is_delete_modal_open.set(false);
                            delete_target_item.set(None);
                        },
                        on_deleted: move |_| {
                            is_delete_modal_open.set(false);
                            delete_target_item.set(None);
                            reload_counter.set(reload_counter() + 1);
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InventoryItemModal(
    item: Option<InventoryItem>,
    clinic_id: String,
    token: String,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let is_editing = item.is_some();
    let initial_item = item.clone();

    let mut item_type = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| i.item_type)
            .unwrap_or(ItemType::Material)
    });
    let mut name = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| i.name.clone())
            .unwrap_or_default()
    });
    let mut unit_type = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| i.unit_type.clone())
            .unwrap_or_else(|| "unidade".to_string())
    });
    let mut current_stock = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| i.current_stock.to_string())
            .unwrap_or_else(|| "0".to_string())
    });
    let mut min_stock = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| i.min_stock.to_string())
            .unwrap_or_else(|| "5".to_string())
    });
    let mut cost_price_input = use_signal(|| {
        if let Some(ref it) = initial_item {
            format_currency(it.cost_price_cents)
        } else {
            "R$ 0,00".to_string()
        }
    });
    let mut manufacturer = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.manufacturer.clone())
            .unwrap_or_default()
    });

    let mut expiration_date = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.expiration_date.as_ref())
            .map(|s| {
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    dt.format("%Y-%m-%d").to_string()
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default()
    });
    let mut batch_number = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.batch_number.clone())
            .unwrap_or_default()
    });

    let mut serial_number = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.serial_number.clone())
            .unwrap_or_default()
    });
    let mut warranty_until = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.warranty_until.as_ref())
            .map(|s| {
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    dt.format("%Y-%m-%d").to_string()
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default()
    });
    let mut next_maintenance_date = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.next_maintenance_date.as_ref())
            .map(|s| {
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    dt.format("%Y-%m-%d").to_string()
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default()
    });
    let mut equipment_status = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.equipment_status)
            .unwrap_or(EquipmentStatus::Active)
    });
    let mut attachments = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| i.attachments.clone())
            .unwrap_or_default()
    });

    let mut is_uploading_doc = use_signal(|| false);
    let mut is_submitting = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let item_edit_id = item.as_ref().map(|i| i.id.clone());
    let tok = token.clone();
    let cid = clinic_id.clone();

    let tok_up = token.clone();
    let cid_up = clinic_id.clone();
    let on_file_upload = move |evt: FormEvent| {
        for file in evt.files() {
            is_uploading_doc.set(true);
            let t = tok_up.clone();
            let c = cid_up.clone();
            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let base64_content = general_purpose::STANDARD.encode(&bytes);
                    let fname = file.name();
                    let mime = if fname.ends_with(".pdf") {
                        "application/pdf".to_string()
                    } else if fname.ends_with(".png") {
                        "image/png".to_string()
                    } else {
                        "image/jpeg".to_string()
                    };

                    let req = FileUploadRequest {
                        filename: fname,
                        mime_type: mime,
                        base64_content,
                    };

                    match api::upload_stock_document(&t, &c, req).await {
                        Ok(new_url) => {
                            let mut current = attachments();
                            current.push(new_url);
                            attachments.set(current);
                        }
                        Err(e) => {
                            form_error.set(Some(e));
                        }
                    }
                }
                is_uploading_doc.set(false);
            });
        }
    };

    let handle_submit = move |_| {
        if name().trim().is_empty() {
            form_error.set(Some("O nome do item é obrigatório.".into()));
            return;
        }

        let cur_stock_num = current_stock().parse::<i32>().unwrap_or(0);
        let min_stock_num = min_stock().parse::<i32>().unwrap_or(0);
        let cost_cents = parse_currency_input(&cost_price_input());

        let exp_rfc = if !expiration_date().is_empty() {
            chrono::NaiveDate::parse_from_str(&expiration_date(), "%Y-%m-%d")
                .ok()
                .map(|d| d.and_hms_opt(12, 0, 0).unwrap().and_utc().to_rfc3339())
        } else {
            None
        };

        let war_rfc = if !warranty_until().is_empty() {
            chrono::NaiveDate::parse_from_str(&warranty_until(), "%Y-%m-%d")
                .ok()
                .map(|d| d.and_hms_opt(12, 0, 0).unwrap().and_utc().to_rfc3339())
        } else {
            None
        };

        let maint_rfc = if !next_maintenance_date().is_empty() {
            chrono::NaiveDate::parse_from_str(&next_maintenance_date(), "%Y-%m-%d")
                .ok()
                .map(|d| d.and_hms_opt(12, 0, 0).unwrap().and_utc().to_rfc3339())
        } else {
            None
        };

        let mfg_opt = if manufacturer().trim().is_empty() {
            None
        } else {
            Some(manufacturer().trim().to_string())
        };
        let batch_opt = if batch_number().trim().is_empty() {
            None
        } else {
            Some(batch_number().trim().to_string())
        };
        let serial_opt = if serial_number().trim().is_empty() {
            None
        } else {
            Some(serial_number().trim().to_string())
        };

        is_submitting.set(true);
        form_error.set(None);

        let tok_spawn = tok.clone();
        let cid_spawn = cid.clone();
        let edit_id_opt = item_edit_id.clone();
        let attach_list = attachments();

        spawn(async move {
            if let Some(edit_id) = edit_id_opt {
                let req = UpdateInventoryItemRequest {
                    clinic_id: cid_spawn,
                    item_type: item_type(),
                    name: name().trim().to_string(),
                    unit_type: unit_type(),
                    current_stock: cur_stock_num,
                    min_stock: min_stock_num,
                    cost_price_cents: cost_cents,
                    manufacturer: mfg_opt,
                    attachments: attach_list,
                    expiration_date: exp_rfc,
                    batch_number: batch_opt,
                    serial_number: serial_opt,
                    warranty_until: war_rfc,
                    next_maintenance_date: maint_rfc,
                    equipment_status: if item_type() == ItemType::Equipment {
                        Some(equipment_status())
                    } else {
                        None
                    },
                };

                match api::update_stock_item(&tok_spawn, &edit_id, req).await {
                    Ok(_) => on_saved.call(()),
                    Err(e) => {
                        form_error.set(Some(e));
                        is_submitting.set(false);
                    }
                }
            } else {
                let req = CreateInventoryItemRequest {
                    clinic_id: cid_spawn,
                    item_type: item_type(),
                    name: name().trim().to_string(),
                    unit_type: unit_type(),
                    current_stock: cur_stock_num,
                    min_stock: min_stock_num,
                    cost_price_cents: cost_cents,
                    manufacturer: mfg_opt,
                    attachments: attach_list,
                    expiration_date: exp_rfc,
                    batch_number: batch_opt,
                    serial_number: serial_opt,
                    warranty_until: war_rfc,
                    next_maintenance_date: maint_rfc,
                    equipment_status: if item_type() == ItemType::Equipment {
                        Some(equipment_status())
                    } else {
                        None
                    },
                };

                match api::create_stock_item(&tok_spawn, req).await {
                    Ok(_) => on_saved.call(()),
                    Err(e) => {
                        form_error.set(Some(e));
                        is_submitting.set(false);
                    }
                }
            }
        });
    };

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "action-modal stock-form-modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "settings-header",
                    h2 { class: "settings-title",
                        if is_editing { "Editar Item de Estoque" } else { "Cadastrar Novo Item / Equipamento" }
                    }
                    button { class: "close-btn", onclick: move |_| on_close.call(()), "×" }
                }

                div { class: "settings-content",
                    if let Some(err) = form_error() {
                        div { class: "modal-alert-error", "{err}" }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Categoria do Item *" }
                        div { class: "modal-type-switcher",
                            button {
                                r#type: "button",
                                class: if item_type() == ItemType::Material { "type-pill-btn active" } else { "type-pill-btn" },
                                onclick: move |_| item_type.set(ItemType::Material),
                                IconBox { size: 15, color: "currentColor".to_string() }
                                span { "Material / Insumo" }
                            }
                            button {
                                r#type: "button",
                                class: if item_type() == ItemType::Chemical { "type-pill-btn active" } else { "type-pill-btn" },
                                onclick: move |_| item_type.set(ItemType::Chemical),
                                IconFlask { size: 15, color: "currentColor".to_string() }
                                span { "Químico / Cosmético" }
                            }
                            button {
                                r#type: "button",
                                class: if item_type() == ItemType::Equipment { "type-pill-btn active" } else { "type-pill-btn" },
                                onclick: move |_| item_type.set(ItemType::Equipment),
                                IconTool { size: 15, color: "currentColor".to_string() }
                                span { "Equipamento / Patrimônio" }
                            }
                        }
                    }

                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Nome do Item / Equipamento *" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: Resina Z350 XT, Autoclave Vitale 12L...",
                                value: "{name()}",
                                oninput: move |e| name.set(e.value())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Fabricante / Marca" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: 3M, Cristófoli, DFL...",
                                value: "{manufacturer()}",
                                oninput: move |e| manufacturer.set(e.value())
                            }
                        }
                    }

                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Unidade de Medida" }
                            select {
                                class: "modern-input-field",
                                value: "{unit_type()}",
                                onchange: move |e| unit_type.set(e.value()),
                                option { value: "unidade", "Unidade (un)" }
                                option { value: "caixa", "Caixa (cx)" }
                                option { value: "par", "Par (pr)" }
                                option { value: "tubete", "Tubete" }
                                option { value: "frasco", "Frasco" }
                                option { value: "litro", "Litro (L)" }
                                option { value: "kg", "Quilograma (kg)" }
                                option { value: "pacote", "Pacote (pct)" }
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Preço de Custo Unitário (R$)" }
                            input {
                                class: "modern-input-field font-semibold",
                                value: "{cost_price_input()}",
                                oninput: move |e| {
                                    let cents = parse_currency_input(&e.value());
                                    cost_price_input.set(format_currency(cents));
                                }
                            }
                        }
                    }

                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Estoque Inicial / Atual" }
                            input {
                                class: "modern-input-field",
                                r#type: "number",
                                min: "0",
                                value: "{current_stock()}",
                                oninput: move |e| current_stock.set(e.value())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Estoque Mínimo de Segurança" }
                            input {
                                class: "modern-input-field",
                                r#type: "number",
                                min: "0",
                                value: "{min_stock()}",
                                oninput: move |e| min_stock.set(e.value())
                            }
                        }
                    }

                    if item_type() == ItemType::Chemical {
                        div { class: "chemical-custom-fields-box",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Lote de Fabricação" }
                                    input {
                                        class: "modern-input-field",
                                        placeholder: "Ex: LT-2026B...",
                                        value: "{batch_number()}",
                                        oninput: move |e| batch_number.set(e.value())
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label", "Data de Validade *" }
                                    input {
                                        class: "modern-input-field",
                                        r#type: "date",
                                        value: "{expiration_date()}",
                                        oninput: move |e| expiration_date.set(e.value())
                                    }
                                }
                            }
                        }
                    }

                    if item_type() == ItemType::Equipment {
                        div { class: "equipment-custom-fields-box",
                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Número de Série (S/N)" }
                                    input {
                                        class: "modern-input-field",
                                        placeholder: "Ex: SCH-99881...",
                                        value: "{serial_number()}",
                                        oninput: move |e| serial_number.set(e.value())
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label", "Status Operacional" }
                                    select {
                                        class: "modern-input-field",
                                        value: match equipment_status() {
                                            EquipmentStatus::Active => "active",
                                            EquipmentStatus::InMaintenance => "in_maintenance",
                                            EquipmentStatus::Broken => "broken",
                                        },
                                        onchange: move |e| {
                                            match e.value().as_str() {
                                                "in_maintenance" => equipment_status.set(EquipmentStatus::InMaintenance),
                                                "broken" => equipment_status.set(EquipmentStatus::Broken),
                                                _ => equipment_status.set(EquipmentStatus::Active),
                                            }
                                        },
                                        option { value: "active", "Operacional / Ativo" }
                                        option { value: "in_maintenance", "Em Manutenção / Assistência" }
                                        option { value: "broken", "Inoperante / Danificado" }
                                    }
                                }
                            }

                            div { class: "form-row-2",
                                div { class: "form-group",
                                    label { class: "form-label", "Garantia até" }
                                    input {
                                        class: "modern-input-field",
                                        r#type: "date",
                                        value: "{warranty_until()}",
                                        oninput: move |e| warranty_until.set(e.value())
                                    }
                                }

                                div { class: "form-group",
                                    label { class: "form-label", "Próxima Manutenção / Calibração" }
                                    input {
                                        class: "modern-input-field",
                                        r#type: "date",
                                        value: "{next_maintenance_date()}",
                                        oninput: move |e| next_maintenance_date.set(e.value())
                                    }
                                }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Documentos & Comprovantes (Fotos / PDFs / Nota Fiscal)" }
                        div { class: "attachments-section",
                            label { class: "attachment-dropzone",
                                input {
                                    r#type: "file",
                                    accept: "image/*,.pdf",
                                    multiple: true,
                                    onchange: on_file_upload,
                                }
                                div { class: "dropzone-label",
                                    IconUpload { size: 18, color: "currentColor".to_string() }
                                    span { if is_uploading_doc() { "Fazendo upload do arquivo..." } else { "Clique para anexar Foto ou PDF" } }
                                }
                                span { class: "dropzone-hint", "Suporta imagens (PNG, JPG) e documentos PDF de notas fiscais, manuais ou certificados" }
                            }

                            if !attachments().is_empty() {
                                div { class: "attachments-list",
                                    for (idx, url) in attachments().iter().enumerate() {
                                        div { class: "attachment-item-chip", key: "{url}",
                                            div { class: "attachment-chip-left",
                                                IconFile { size: 16, color: "#64748b".to_string() }
                                                span { "{extract_filename(url)}" }
                                            }
                                            div { class: "attachment-chip-right",
                                                a {
                                                    class: "attachment-view-link",
                                                    href: "{url}",
                                                    target: "_blank",
                                                    IconExternalLink { size: 13, color: "currentColor".to_string() }
                                                    span { "Visualizar" }
                                                }
                                                button {
                                                    class: "attachment-remove-btn",
                                                    r#type: "button",
                                                    title: "Remover Anexo",
                                                    onclick: move |_| {
                                                        let mut list = attachments();
                                                        if idx < list.len() {
                                                            list.remove(idx);
                                                            attachments.set(list);
                                                        }
                                                    },
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

                div { class: "modal-footer-actions",
                    button {
                        class: "btn-secondary",
                        r#type: "button",
                        onclick: move |_| on_close.call(()),
                        "Cancelar"
                    }
                    button {
                        class: "btn-primary",
                        r#type: "button",
                        disabled: is_submitting() || is_uploading_doc(),
                        onclick: handle_submit,
                        if is_submitting() { "Salvando..." } else if is_editing { "Salvar Alterações" } else { "Cadastrar Item" }
                    }
                }
            }
        }
    }
}

#[component]
fn StockMovementModal(
    all_items: Vec<InventoryItem>,
    target_item: Option<InventoryItem>,
    clinic_id: String,
    token: String,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
) -> Element {
    let initial_item_id = target_item
        .as_ref()
        .map(|i| i.id.clone())
        .unwrap_or_else(|| all_items.first().map(|i| i.id.clone()).unwrap_or_default());

    let mut selected_item_id = use_signal(|| initial_item_id);
    let mut movement_type = use_signal(|| MovementType::PurchaseIn);
    let mut quantity = use_signal(|| "1".to_string());
    let mut unit_cost_input = use_signal(|| "R$ 0,00".to_string());
    let mut invoice_number = use_signal(|| String::new());
    let mut notes = use_signal(|| String::new());
    let mut doc_attachment_url = use_signal(|| None::<String>);

    let mut is_uploading_doc = use_signal(|| false);
    let mut is_submitting = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let tok_up = token.clone();
    let cid_up = clinic_id.clone();
    let on_doc_upload = move |evt: FormEvent| {
        for file in evt.files() {
            is_uploading_doc.set(true);
            let t = tok_up.clone();
            let c = cid_up.clone();
            spawn(async move {
                if let Ok(bytes) = file.read_bytes().await {
                    let base64_content = general_purpose::STANDARD.encode(&bytes);
                    let fname = file.name();
                    let mime = if fname.ends_with(".pdf") {
                        "application/pdf".to_string()
                    } else {
                        "image/jpeg".to_string()
                    };

                    let req = FileUploadRequest {
                        filename: fname,
                        mime_type: mime,
                        base64_content,
                    };

                    match api::upload_stock_document(&t, &c, req).await {
                        Ok(new_url) => {
                            doc_attachment_url.set(Some(new_url));
                        }
                        Err(e) => {
                            form_error.set(Some(e));
                        }
                    }
                }
                is_uploading_doc.set(false);
            });
        }
    };

    let handle_submit = move |_| {
        let qty_num = quantity().parse::<i32>().unwrap_or(0);
        if qty_num <= 0 {
            form_error.set(Some("Informe uma quantidade maior que zero.".into()));
            return;
        }

        let item_id_val = selected_item_id();
        if item_id_val.is_empty() {
            form_error.set(Some("Selecione um item para movimentar.".into()));
            return;
        }

        let is_in = movement_type() == MovementType::PurchaseIn
            || movement_type() == MovementType::Adjustment;
        let qty_signed = if is_in { qty_num } else { -qty_num };

        let cost_cents = parse_currency_input(&unit_cost_input());
        let cost_opt = if cost_cents > 0 {
            Some(cost_cents)
        } else {
            None
        };
        let inv_opt = if invoice_number().trim().is_empty() {
            None
        } else {
            Some(invoice_number().trim().to_string())
        };

        let mut final_notes = notes().trim().to_string();
        if let Some(ref doc_url) = doc_attachment_url() {
            if !final_notes.is_empty() {
                final_notes.push_str(&format!(" [Comprovante: {}]", doc_url));
            } else {
                final_notes = format!("[Comprovante: {}]", doc_url);
            }
        }
        let notes_opt = if final_notes.is_empty() {
            None
        } else {
            Some(final_notes)
        };

        is_submitting.set(true);
        form_error.set(None);

        let tok_spawn = tok.clone();
        let cid_spawn = cid.clone();

        spawn(async move {
            let req = CreateStockMovementRequest {
                clinic_id: cid_spawn,
                item_id: item_id_val.clone(),
                quantity_change: qty_signed,
                movement_type: movement_type(),
                unit_cost_cents: cost_opt,
                invoice_number: inv_opt,
                notes: notes_opt,
            };

            match api::create_stock_movement(&tok_spawn, &item_id_val, req).await {
                Ok(_) => on_saved.call(()),
                Err(e) => {
                    form_error.set(Some(e));
                    is_submitting.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "action-modal stock-movement-modal",
                onclick: move |e| e.stop_propagation(),
                div { class: "settings-header",
                    h2 { class: "settings-title", "Registrar Movimentação de Estoque" }
                    button { class: "close-btn", onclick: move |_| on_close.call(()), "×" }
                }

                div { class: "settings-content",
                    if let Some(err) = form_error() {
                        div { class: "modal-alert-error", "{err}" }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Item *" }
                        select {
                            class: "modern-input-field",
                            value: "{selected_item_id()}",
                            onchange: move |e| selected_item_id.set(e.value()),
                            for it in &all_items {
                                option { value: "{it.id}", "{it.name} (Atual: {it.current_stock} {it.unit_type})" }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Tipo de Movimentação *" }
                        div { class: "modal-type-switcher",
                            button {
                                r#type: "button",
                                class: if movement_type() == MovementType::PurchaseIn { "type-pill-btn active text-income" } else { "type-pill-btn" },
                                onclick: move |_| movement_type.set(MovementType::PurchaseIn),
                                IconArrowDown { size: 15, color: "currentColor".to_string() }
                                span { "Entrada / Compra" }
                            }
                            button {
                                r#type: "button",
                                class: if movement_type() == MovementType::ManualOut { "type-pill-btn active text-expense" } else { "type-pill-btn" },
                                onclick: move |_| movement_type.set(MovementType::ManualOut),
                                IconArrowUp { size: 15, color: "currentColor".to_string() }
                                span { "Saída Manual (Consumo)" }
                            }
                            button {
                                r#type: "button",
                                class: if movement_type() == MovementType::Loss { "type-pill-btn active text-expense" } else { "type-pill-btn" },
                                onclick: move |_| movement_type.set(MovementType::Loss),
                                IconAlertTriangle { size: 15, color: "currentColor".to_string() }
                                span { "Avaria / Perda" }
                            }
                            button {
                                r#type: "button",
                                class: if movement_type() == MovementType::Adjustment { "type-pill-btn active" } else { "type-pill-btn" },
                                onclick: move |_| movement_type.set(MovementType::Adjustment),
                                IconRefresh { size: 15, color: "currentColor".to_string() }
                                span { "Ajuste de Balanço" }
                            }
                        }
                    }

                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Quantidade *" }
                            input {
                                class: "modern-input-field font-bold",
                                r#type: "number",
                                min: "1",
                                value: "{quantity()}",
                                oninput: move |e| quantity.set(e.value())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Custo Unitário (R$)" }
                            input {
                                class: "modern-input-field",
                                value: "{unit_cost_input()}",
                                oninput: move |e| {
                                    let cents = parse_currency_input(&e.value());
                                    unit_cost_input.set(format_currency(cents));
                                }
                            }
                        }
                    }

                    div { class: "form-row-2",
                        div { class: "form-group",
                            label { class: "form-label", "Nota Fiscal / Recibo" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: NF-10492...",
                                value: "{invoice_number()}",
                                oninput: move |e| invoice_number.set(e.value())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Observações" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: Reposição periódica, quebra de frasco...",
                                value: "{notes()}",
                                oninput: move |e| notes.set(e.value())
                            }
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Comprovante / PDF de Nota Fiscal" }
                        label { class: "attachment-dropzone",
                            input {
                                r#type: "file",
                                accept: "image/*,.pdf",
                                onchange: on_doc_upload,
                            }
                            div { class: "dropzone-label",
                                IconUpload { size: 16, color: "currentColor".to_string() }
                                span { if is_uploading_doc() { "Enviando arquivo..." } else { "Clique para anexar arquivo da NF" } }
                            }
                        }
                        if let Some(ref doc_url) = doc_attachment_url() {
                            div { class: "attachment-item-chip mt-2",
                                div { class: "attachment-chip-left",
                                    IconFile { size: 16, color: "#64748b".to_string() }
                                    span { "{extract_filename(doc_url)}" }
                                }
                                div { class: "attachment-chip-right",
                                    a {
                                        class: "attachment-view-link",
                                        href: "{doc_url}",
                                        target: "_blank",
                                        IconExternalLink { size: 13, color: "currentColor".to_string() }
                                        span { "Ver" }
                                    }
                                    button {
                                        class: "attachment-remove-btn",
                                        r#type: "button",
                                        onclick: move |_| doc_attachment_url.set(None),
                                        IconTrash { size: 14, color: "currentColor".to_string() }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "modal-footer-actions",
                    button {
                        class: "btn-secondary",
                        r#type: "button",
                        onclick: move |_| on_close.call(()),
                        "Cancelar"
                    }
                    button {
                        class: "btn-primary",
                        r#type: "button",
                        disabled: is_submitting() || is_uploading_doc(),
                        onclick: handle_submit,
                        if is_submitting() { "Registrando..." } else { "Confirmar Movimentação" }
                    }
                }
            }
        }
    }
}

#[component]
fn DeleteConfirmModal(
    item: InventoryItem,
    clinic_id: String,
    token: String,
    on_close: EventHandler<()>,
    on_deleted: EventHandler<()>,
) -> Element {
    let mut is_submitting = use_signal(|| false);
    let mut form_error = use_signal(|| None::<String>);

    let item_id = item.id.clone();
    let item_name = item.name.clone();
    let tok = token.clone();
    let cid = clinic_id.clone();

    let handle_confirm = move |_| {
        is_submitting.set(true);
        form_error.set(None);

        let tok_spawn = tok.clone();
        let cid_spawn = cid.clone();
        let id_spawn = item_id.clone();

        spawn(async move {
            match api::delete_stock_item(&tok_spawn, &cid_spawn, &id_spawn).await {
                Ok(_) => on_deleted.call(()),
                Err(e) => {
                    form_error.set(Some(e));
                    is_submitting.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "action-modal modal-sm",
                onclick: move |e| e.stop_propagation(),
                div { class: "settings-header",
                    h2 { class: "settings-title text-danger", "Excluir Item do Estoque" }
                    button { class: "close-btn", onclick: move |_| on_close.call(()), "×" }
                }

                div { class: "settings-content",
                    if let Some(err) = form_error() {
                        div { class: "modal-alert-error", "{err}" }
                    }

                    p { class: "delete-confirm-text",
                        "Tem certeza que deseja excluir o item "
                        strong { "{item_name}" }
                        "? Esta ação removerá o registro e seu saldo do inventário."
                    }
                }

                div { class: "modal-footer-actions",
                    button {
                        class: "btn-secondary",
                        r#type: "button",
                        onclick: move |_| on_close.call(()),
                        "Cancelar"
                    }
                    button {
                        class: "btn-danger",
                        r#type: "button",
                        disabled: is_submitting(),
                        onclick: handle_confirm,
                        if is_submitting() { "Excluindo..." } else { "Sim, Excluir" }
                    }
                }
            }
        }
    }
}
