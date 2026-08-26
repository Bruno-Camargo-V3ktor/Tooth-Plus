pub mod components;

use crate::api::mock_db::DB;
use crate::api::stock::StockApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::stock::{CreateInventoryItemRequest, InventoryItem, ItemType, StockQuery};
use dioxus::prelude::*;

pub use components::{ModalMovement, ModalNewItem, StockKpis, StockTable, StockToolbar};

const STYLE: Asset = asset!("/src/pages/stock/style.css");

#[component]
pub fn StockView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut stock_items = use_signal(Vec::<InventoryItem>::new);
    let mut search_query = use_signal(String::new);
    let mut show_new_item_modal = use_signal(|| false);
    let mut show_movement_modal = use_signal(|| false);

    // Form: Novo Produto
    let mut name = use_signal(String::new);
    let mut category = use_signal(|| "material".to_string());
    let mut unit_type = use_signal(|| "un".to_string());
    let mut current_stock_str = use_signal(|| "10".to_string());
    let mut min_stock_str = use_signal(|| "5".to_string());
    let mut manufacturer = use_signal(String::new);
    let mut cost_price_str = use_signal(|| "0.00".to_string());

    // Form: Movimentação
    let mut selected_item_id = use_signal(String::new);
    let mut movement_type = use_signal(|| "ENTRY".to_string());
    let mut quantity_str = use_signal(|| "1".to_string());
    let mut movement_reason = use_signal(String::new);

    let load_stock = {
        let cid = clinic_id.clone();
        let mut list_sig = stock_items;
        let mut sel_sig = selected_item_id;

        move || {
            let cid = cid.clone();
            let query = StockQuery {
                clinic_id: cid,
                item_type: None,
                search: None,
            };

            spawn(async move {
                if let Ok(resp) = StockApi::list_stock(query).await {
                    if !resp.items.is_empty() && sel_sig.read().is_empty() {
                        sel_sig.set(resp.items[0].id.clone());
                    }
                    list_sig.set(resp.items);
                }
            });
        }
    };

    use_effect({
        let mut loader = load_stock.clone();
        move || loader()
    });

    let handle_create_item = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut loader = load_stock.clone();
        let mut modal_sig = show_new_item_modal;
        let name_s = name.clone();
        let cat_s = category.clone();
        let unit_s = unit_type.clone();
        let curr_s = current_stock_str.clone();
        let min_s = min_stock_str.clone();
        let man_s = manufacturer.clone();
        let cost_s = cost_price_str.clone();

        move |_| {
            let n = name_s.read().trim().to_string();
            if n.is_empty() {
                toast_c.show("Informe o nome do produto.", ToastVariant::Error);
                return;
            }
            let curr: i32 = curr_s.read().parse().unwrap_or(0);
            let min: i32 = min_s.read().parse().unwrap_or(0);
            let cost: f64 = cost_s.read().replace(',', ".").parse().unwrap_or(0.0);
            let cost_cents = (cost * 100.0) as i64;

            let item_type = match cat_s.read().as_str() {
                "chemical" => ItemType::Chemical,
                "equipment" => ItemType::Equipment,
                _ => ItemType::Material,
            };

            let req = CreateInventoryItemRequest {
                clinic_id: cid.clone(),
                item_type,
                name: n,
                unit_type: unit_s.read().clone(),
                current_stock: curr,
                min_stock: min,
                cost_price_cents: cost_cents,
                manufacturer: if man_s.read().is_empty() { None } else { Some(man_s.read().clone()) },
                attachments: vec![],
                expiration_date: None,
                batch_number: None,
                serial_number: None,
                warranty_until: None,
                next_maintenance_date: None,
                equipment_status: None,
            };

            let mut toast_resp = toast_c.clone();
            let mut loader_c = loader.clone();
            let mut modal_c = modal_sig;

            spawn(async move {
                match StockApi::create_item(req).await {
                    Ok(_) => {
                        toast_resp.show("Produto cadastrado com sucesso!", ToastVariant::Success);
                        modal_c.set(false);
                        loader_c();
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let handle_movement = {
        let mut toast_c = toast.clone();
        let mut loader = load_stock.clone();
        let mut modal_sig = show_movement_modal;
        let item_id_s = selected_item_id.clone();
        let mov_type_s = movement_type.clone();
        let qty_s = quantity_str.clone();

        move |_| {
            let target_id = item_id_s.read().clone();
            let qty: i32 = qty_s.read().parse().unwrap_or(0);
            let is_entry = *mov_type_s.read() == "ENTRY";

            if qty <= 0 {
                toast_c.show("Informe uma quantidade válida.", ToastVariant::Error);
                return;
            }

            if let Ok(mut db) = DB.lock() {
                if let Some(item) = db.inventory_items.iter_mut().find(|i| i.id == target_id) {
                    if is_entry {
                        item.current_stock += qty;
                    } else {
                        item.current_stock = item.current_stock.saturating_sub(qty);
                    }
                }
            }
            toast_c.show("Movimentação registrada com sucesso!", ToastVariant::Success);
            modal_sig.set(false);
            loader();
        }
    };

    let filtered_items: Vec<InventoryItem> = stock_items.read().iter().filter(|i| {
        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        i.name.to_lowercase().contains(&q)
    }).cloned().collect();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "stock-page",
            StockKpis { items: stock_items() }

            StockToolbar {
                search_query,
                on_new_movement: move |_| show_movement_modal.set(true),
                on_new_item: move |_| show_new_item_modal.set(true),
            }

            StockTable { items: filtered_items }

            ModalNewItem {
                is_open: show_new_item_modal(),
                name,
                category,
                unit_type,
                current_stock_str,
                min_stock_str,
                manufacturer,
                cost_price_str,
                on_close: move |_| show_new_item_modal.set(false),
                on_submit: handle_create_item,
            }

            ModalMovement {
                is_open: show_movement_modal(),
                items: stock_items(),
                selected_item_id,
                movement_type,
                quantity_str,
                reason: movement_reason,
                on_close: move |_| show_movement_modal.set(false),
                on_submit: handle_movement,
            }
        }
    }
}
