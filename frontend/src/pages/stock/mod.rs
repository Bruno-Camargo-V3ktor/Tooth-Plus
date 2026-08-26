//! # Módulo de Gestão de Inventário e Estoque (Tooth Plus V2)
//!
//! Controle de materiais de consumo, lotes, validade e movimentações de entrada/saída.

use crate::api::stock::StockApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconBox, IconPlus, IconSearch};
use dioxus::prelude::*;
use shared::stock::{
    CreateInventoryItemRequest, CreateStockMovementRequest, InventoryItem, ItemType,
    MovementType, StockAlertItem, StockKPIs, StockQuery, StockResponse,
};

const STYLE: Asset = asset!("/src/pages/stock/style.css");

fn format_currency_br(cents: i64) -> String {
    let reais = cents as f64 / 100.0;
    format!("R$ {:.2}", reais).replace('.', ",")
}

#[component]
pub fn StockView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut items = use_signal(Vec::<InventoryItem>::new);
    let mut alerts = use_signal(Vec::<StockAlertItem>::new);
    let mut kpis = use_signal(StockKPIs::default);
    let mut is_loading = use_signal(|| true);

    let mut search_query = use_signal(String::new);
    let mut type_filter = use_signal(|| "all".to_string());

    // Modais
    let mut show_new_item_modal = use_signal(|| false);
    let mut show_movement_modal = use_signal(|| false);

    // Campos novo item
    let mut new_name = use_signal(String::new);
    let mut new_sku = use_signal(String::new);
    let mut new_unit = use_signal(|| "unidade".to_string());
    let mut new_stock_qty = use_signal(|| 10i32);
    let mut new_min_stock = use_signal(|| 5i32);
    let mut new_cost_str = use_signal(|| "25,00".to_string());
    let mut new_expiry = use_signal(|| "2027-12-31".to_string());

    // Campos movimentação
    let mut mov_item_id = use_signal(String::new);
    let mut mov_qty = use_signal(|| 5i32);
    let mut mov_type = use_signal(|| MovementType::PurchaseIn);
    let mut mov_notes = use_signal(String::new);

    // Carrega dados do estoque
    let load_stock = {
        let cid = clinic_id.clone();
        let mut items_sig = items.clone();
        let mut alerts_sig = alerts.clone();
        let mut kpis_sig = kpis.clone();
        let mut load_sig = is_loading.clone();
        let mut toast_sig = toast.clone();

        move || {
            let clinic_key = cid.clone();
            let mut it_s = items_sig.clone();
            let mut al_s = alerts_sig.clone();
            let mut kp_s = kpis_sig.clone();
            let mut l_s = load_sig.clone();
            let mut t_c = toast_sig.clone();

            spawn(async move {
                l_s.set(true);
                let q = StockQuery {
                    clinic_id: clinic_key,
                    item_type: None,
                    search: None,
                };
                match StockApi::list_stock(q).await {
                    Ok(resp) => {
                        it_s.set(resp.items);
                        al_s.set(resp.alerts);
                        kp_s.set(resp.kpis);
                    }
                    Err(e) => {
                        t_c.show(format!("Erro ao carregar estoque: {}", e), ToastVariant::Error);
                    }
                }
                l_s.set(false);
            });
        }
    };

    use_effect({
        let mut ls = load_stock.clone();
        move || {
            ls();
        }
    });

    // Filtra itens
    let q_term = search_query.read().to_lowercase();
    let t_term = type_filter.read().clone();

    let filtered_items: Vec<InventoryItem> = items
        .read()
        .iter()
        .filter(|i| {
            if t_term == "material" && i.item_type != ItemType::Material { return false; }
            if t_term == "chemical" && i.item_type != ItemType::Chemical { return false; }
            if t_term == "equipment" && i.item_type != ItemType::Equipment { return false; }
            if !q_term.is_empty() && !i.name.to_lowercase().contains(&q_term) && !i.manufacturer.as_deref().unwrap_or("").to_lowercase().contains(&q_term) {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    // Handler para criar novo item
    let handle_create_item = {
        let cid = clinic_id.clone();
        let mut show_m = show_new_item_modal.clone();
        let mut toast_c = toast.clone();
        let mut ls = load_stock.clone();

        move |_| {
            let name_val = new_name.read().trim().to_string();
            if name_val.is_empty() {
                toast_c.show("Informe o nome do produto.", ToastVariant::Error);
                return;
            }

            let cost_val: f64 = new_cost_str.read().replace(',', ".").parse().unwrap_or(0.0);
            let cost_cents = (cost_val * 100.0) as i64;

            let req = CreateInventoryItemRequest {
                clinic_id: cid.clone(),
                name: name_val,
                item_type: ItemType::Material,
                unit_type: new_unit.read().clone(),
                min_stock: *new_min_stock.read(),
                current_stock: *new_stock_qty.read(),
                cost_price_cents: cost_cents,
                manufacturer: if new_sku.read().is_empty() { None } else { Some(new_sku.read().clone()) },
                attachments: vec![],
                expiration_date: Some(new_expiry.read().clone()),
                batch_number: Some("LOTE-2026".to_string()),
                serial_number: None,
                warranty_until: None,
                next_maintenance_date: None,
                equipment_status: None,
            };

            let mut t_c = toast_c.clone();
            let mut s_m = show_m.clone();
            let mut ls_c = ls.clone();

            spawn(async move {
                match StockApi::create_item(req).await {
                    Ok(_) => {
                        t_c.show("Item adicionado ao estoque!", ToastVariant::Success);
                        s_m.set(false);
                        ls_c();
                    }
                    Err(e) => {
                        t_c.show(format!("Erro ao criar item: {}", e), ToastVariant::Error);
                    }
                }
            });
        }
    };

    // Handler para movimentação de estoque
    let handle_create_movement = {
        let cid = clinic_id.clone();
        let mut show_m = show_movement_modal.clone();
        let mut toast_c = toast.clone();
        let mut ls = load_stock.clone();

        move |_| {
            let i_id = mov_item_id.read().clone();
            if i_id.is_empty() {
                toast_c.show("Selecione um item para movimentar.", ToastVariant::Error);
                return;
            }

            let m_type = *mov_type.read();
            let qty = *mov_qty.read();
            let signed_qty = match m_type {
                MovementType::PurchaseIn => qty,
                MovementType::ManualOut | MovementType::AppointmentOut | MovementType::Loss => -qty,
                MovementType::Adjustment => qty,
            };

            let req = CreateStockMovementRequest {
                clinic_id: cid.clone(),
                item_id: i_id,
                quantity_change: signed_qty,
                movement_type: m_type,
                unit_cost_cents: None,
                invoice_number: None,
                notes: if mov_notes.read().is_empty() { None } else { Some(mov_notes.read().clone()) },
            };

            let mut t_c = toast_c.clone();
            let mut s_m = show_m.clone();
            let mut ls_c = ls.clone();

            spawn(async move {
                match StockApi::create_movement(req).await {
                    Ok(_) => {
                        t_c.show("Movimentação de estoque realizada!", ToastVariant::Success);
                        s_m.set(false);
                        ls_c();
                    }
                    Err(e) => {
                        t_c.show(format!("Erro ao movimentar estoque: {}", e), ToastVariant::Error);
                    }
                }
            });
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "stock-page",

            // 1. KPI Cards
            div { class: "stock-kpi-grid",
                div { class: "stock-kpi-card",
                    span { class: "stock-kpi-label", "Total de Itens" }
                    span { class: "stock-kpi-value", "{kpis.read().total_items_count}" }
                }
                div { class: "stock-kpi-card",
                    span { class: "stock-kpi-label", "Valor em Estoque" }
                    span { class: "stock-kpi-value", "{format_currency_br(kpis.read().total_inventory_value_cents)}" }
                }
                div { class: "stock-kpi-card kpi-alert",
                    span { class: "stock-kpi-label", "Estoque Baixo" }
                    span { class: "stock-kpi-value", style: "color: #dc2626;", "{kpis.read().low_stock_alerts_count}" }
                }
                div { class: "stock-kpi-card kpi-alert-warn",
                    span { class: "stock-kpi-label", "Vencimento Próximo" }
                    span { class: "stock-kpi-value", style: "color: #d97706;", "{kpis.read().expiring_alerts_count}" }
                }
            }

            // 2. Alertas Críticos
            if !alerts.read().is_empty() {
                div { class: "stock-alerts-container",
                    for alert in alerts.read().iter().take(2) {
                        div {
                            key: "{alert.id}",
                            class: "stock-alert-banner alert-warning",
                            span { "⚠ {alert.title}: {alert.message}" }
                        }
                    }
                }
            }

            // 3. Toolbar & Filtros
            div { class: "stock-toolbar",
                div { class: "stock-search-box",
                    IconSearch { size: 16, color: "#94a3b8".to_string() }
                    input {
                        class: "stock-search-input",
                        r#type: "text",
                        placeholder: "Buscar produto por nome ou código...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                select {
                    class: "finance-select",
                    value: "{type_filter}",
                    onchange: move |e| type_filter.set(e.value()),
                    option { value: "all", "Todas as Categorias" }
                    option { value: "material", "Materiais & Resinas" }
                    option { value: "chemical", "Anestésicos & Químicos" }
                    option { value: "equipment", "Equipamentos & Peças" }
                }

                div { class: "stock-actions",
                    button {
                        class: "btn-movement",
                        onclick: move |_| {
                            if let Some(first) = items.read().first() {
                                mov_item_id.set(first.id.clone());
                            }
                            show_movement_modal.set(true);
                        },
                        "↕ Entrada / Saída"
                    }
                    button {
                        class: "btn-add-item",
                        onclick: move |_| show_new_item_modal.set(true),
                        IconPlus { size: 16, color: "#ffffff".to_string() }
                        span { "Novo Produto" }
                    }
                }
            }

            // 4. Tabela de Estoque
            div { class: "stock-table-container",
                if is_loading() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "📦" }
                        p { class: "empty-state-title", "Carregando estoque..." }
                    }
                } else if filtered_items.is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-state-icon", "🔍" }
                        p { class: "empty-state-title", "Nenhum produto cadastrado" }
                        p { class: "empty-state-desc", "Cadastre materiais e insumos da clínica clicando em 'Novo Produto'." }
                    }
                } else {
                    table { class: "stock-table",
                        thead {
                            tr {
                                th { "Produto / Material" }
                                th { "Lote / Validade" }
                                th { "Localização" }
                                th { "Estoque Atual" }
                                th { "Estoque Mínimo" }
                                th { "Status" }
                            }
                        }
                        tbody {
                            for item in filtered_items.iter() {
                                {
                                    let is_low = item.current_stock <= item.min_stock;
                                    let exp_fmt = item.expiration_date.as_deref().unwrap_or("Indeterminada").to_string();
                                    let batch_fmt = item.batch_number.as_deref().unwrap_or("S/L").to_string();
                                    let mfg_fmt = item.manufacturer.clone().unwrap_or_else(|| "-".to_string());
                                    let unit_fmt = item.unit_type.clone();
                                    let status_text = if is_low { "Repor Estoque" } else { "Normal" };

                                    rsx! {
                                        tr {
                                            key: "{item.id}",
                                            td {
                                                div { style: "font-weight: 700; color: #0f172a;", "{item.name}" }
                                                div { style: "font-size: 11.5px; color: #64748b;", "Fabr: {mfg_fmt}" }
                                            }
                                            td {
                                                div { "Lote: {batch_fmt}" }
                                                div { style: "font-size: 11.5px; color: #64748b;", "Val: {exp_fmt}" }
                                            }
                                            td { "Armário Central" }
                                            td {
                                                span { style: "font-weight: 800; font-size: 14px;", "{item.current_stock}" }
                                                span { style: "font-size: 12px; color: #64748b; margin-left: 4px;", "{unit_fmt}" }
                                            }
                                            td { "{item.min_stock} {unit_fmt}" }
                                            td {
                                                span {
                                                    class: if is_low { "badge-stock-low" } else { "badge-stock-ok" },
                                                    "{status_text}"
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

            // 5. Modal Novo Produto
            if *show_new_item_modal.read() {
                div { class: "modal-overlay",
                    onclick: move |_| show_new_item_modal.set(false),

                    div { class: "modal-box", onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            span { class: "modal-title", "Cadastrar Novo Produto / Material" }
                            button { class: "modal-close-btn", onclick: move |_| show_new_item_modal.set(false), "✕" }
                        }

                        div { class: "modal-body",
                            div { class: "form-field",
                                label { class: "form-label", "Nome do Produto *" }
                                input {
                                    class: "form-input",
                                    r#type: "text",
                                    placeholder: "Ex: Resina Composta Filtek Z350 A2",
                                    value: "{new_name}",
                                    oninput: move |e| new_name.set(e.value()),
                                }
                            }

                            div { class: "form-row-2 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Código / SKU" }
                                    input {
                                        class: "form-input",
                                        r#type: "text",
                                        placeholder: "Ex: RES-Z350",
                                        value: "{new_sku}",
                                        oninput: move |e| new_sku.set(e.value()),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Unidade de Medida" }
                                    select {
                                        class: "form-select",
                                        value: "{new_unit}",
                                        onchange: move |e| new_unit.set(e.value()),
                                        option { "unidade" }
                                        option { "caixa" }
                                        option { "tubo" }
                                        option { "frasco" }
                                        option { "kit" }
                                    }
                                }
                            }

                            div { class: "form-row-3 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Quantidade Inicial" }
                                    input {
                                        class: "form-input",
                                        r#type: "number",
                                        value: "{new_stock_qty}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<i32>() { new_stock_qty.set(v); }
                                        }
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Estoque Mínimo" }
                                    input {
                                        class: "form-input",
                                        r#type: "number",
                                        value: "{new_min_stock}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<i32>() { new_min_stock.set(v); }
                                        }
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Custo Unitário (R$)" }
                                    input {
                                        class: "form-input",
                                        r#type: "text",
                                        value: "{new_cost_str}",
                                        oninput: move |e| new_cost_str.set(e.value()),
                                    }
                                }
                            }

                            div { class: "form-field",
                                label { class: "form-label", "Data de Validade" }
                                input {
                                    class: "form-input",
                                    r#type: "date",
                                    value: "{new_expiry}",
                                    oninput: move |e| new_expiry.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-modal-ghost", onclick: move |_| show_new_item_modal.set(false), "Cancelar" }
                            button {
                                class: "btn-modal-primary",
                                onclick: handle_create_item,
                                "Cadastrar Item"
                            }
                        }
                    }
                }
            }

            // 6. Modal Movimentação de Estoque
            if *show_movement_modal.read() {
                div { class: "modal-overlay",
                    onclick: move |_| show_movement_modal.set(false),

                    div { class: "modal-box modal-sm", onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            span { class: "modal-title", "Movimentação de Estoque" }
                            button { class: "modal-close-btn", onclick: move |_| show_movement_modal.set(false), "✕" }
                        }

                        div { class: "modal-body",
                            div { class: "form-field",
                                label { class: "form-label", "Selecione o Produto *" }
                                select {
                                    class: "form-select",
                                    value: "{mov_item_id}",
                                    onchange: move |e| mov_item_id.set(e.value()),
                                    for it in items.read().iter() {
                                        option { value: "{it.id}", "{it.name} (Atual: {it.current_stock})" }
                                    }
                                }
                            }

                            div { class: "form-row-2 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Tipo de Movimentação" }
                                    select {
                                        class: "form-select",
                                        onchange: move |e| {
                                            match e.value().as_str() {
                                                "in" => mov_type.set(MovementType::PurchaseIn),
                                                "out" => mov_type.set(MovementType::ManualOut),
                                                _ => mov_type.set(MovementType::Adjustment),
                                            }
                                        },
                                        option { value: "in", "Entrada (Compra / Reposição)" }
                                        option { value: "out", "Saída (Consumo Manual)" }
                                        option { value: "adj", "Ajuste de Balanço" }
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Quantidade *" }
                                    input {
                                        class: "form-input",
                                        r#type: "number",
                                        min: "1",
                                        value: "{mov_qty}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<i32>() { mov_qty.set(v); }
                                        }
                                    }
                                }
                            }

                            div { class: "form-field",
                                label { class: "form-label", "Observação / Motivo" }
                                input {
                                    class: "form-input",
                                    r#type: "text",
                                    placeholder: "Ex: Reposição lote semanal",
                                    value: "{mov_notes}",
                                    oninput: move |e| mov_notes.set(e.value()),
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-modal-ghost", onclick: move |_| show_movement_modal.set(false), "Cancelar" }
                            button {
                                class: "btn-modal-primary",
                                onclick: handle_create_movement,
                                "Confirmar Movimentação"
                            }
                        }
                    }
                }
            }
        }
    }
}
