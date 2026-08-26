pub mod components;

use crate::api::stock::StockApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::stock::{
    CreateInventoryItemRequest, CreateStockMovementRequest, InventoryItem, ItemType, MovementType,
    StockQuery, UpdateInventoryItemRequest,
};
use dioxus::prelude::*;

pub use components::{ModalItem, ModalMovement, StockTable, StockToolbar};

const STYLE: Asset = asset!("/src/pages/stock/style.css");

#[component]
pub fn StockView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut stock_items = use_signal(Vec::<InventoryItem>::new);
    let mut search_query = use_signal(String::new);
    let mut type_filter = use_signal(|| "ALL".to_string());
    let mut show_new_item_modal = use_signal(|| false);
    let mut show_movement_modal = use_signal(|| false);
    let mut editing_item_id = use_signal(|| None::<String>);
    let mut reload_trigger = use_signal(|| 0);

    let mut name = use_signal(String::new);
    let mut item_type = use_signal(|| "material".to_string());
    let mut unit_type = use_signal(|| "un".to_string());
    let mut current_stock_str = use_signal(|| "10".to_string());
    let mut min_stock_str = use_signal(|| "5".to_string());
    let mut manufacturer = use_signal(String::new);
    let mut cost_price_str = use_signal(|| "0.00".to_string());
    let mut expiration_date = use_signal(String::new);
    let mut batch_number = use_signal(String::new);

    let mut movement_item_id = use_signal(String::new);
    let mut movement_type = use_signal(|| "ENTRY".to_string());
    let mut movement_qty_str = use_signal(|| "1".to_string());
    let mut movement_reason = use_signal(String::new);

    let cid_effect = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_effect.clone();
        let query = StockQuery {
            clinic_id: cid,
            item_type: None,
            search: None,
        };
        spawn(async move {
            if let Ok(resp) = StockApi::list_stock(query).await {
                stock_items.set(resp.items);
            }
        });
    });

    let handle_save_item = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut modal_sig = show_new_item_modal;
        let mut reload_sig = reload_trigger;
        let edit_id_sig = editing_item_id;

        let n_s = name.clone();
        let it_s = item_type.clone();
        let ut_s = unit_type.clone();
        let cs_s = current_stock_str.clone();
        let ms_s = min_stock_str.clone();
        let mfg_s = manufacturer.clone();
        let cp_s = cost_price_str.clone();
        let exp_s = expiration_date.clone();
        let bch_s = batch_number.clone();

        move |_| {
            let n = n_s.read().trim().to_string();
            if n.is_empty() {
                toast_c.show("Informe o nome do item.", ToastVariant::Error);
                return;
            }

            let it = match it_s.read().as_str() {
                "chemical" => ItemType::Chemical,
                "equipment" => ItemType::Equipment,
                _ => ItemType::Material,
            };

            let cs: i32 = cs_s.read().parse().unwrap_or(0);
            let ms: i32 = ms_s.read().parse().unwrap_or(0);
            let cost_num: f64 = cp_s.read().replace(',', ".").parse().unwrap_or(0.0);
            let cost_cents = (cost_num * 100.0) as i64;
            let mfg_val = if mfg_s.read().is_empty() { None } else { Some(mfg_s.read().clone()) };
            let exp_val = if exp_s.read().is_empty() { None } else { Some(exp_s.read().clone()) };
            let bch_val = if bch_s.read().is_empty() { None } else { Some(bch_s.read().clone()) };

            let mut toast_resp = toast_c.clone();
            let mut modal_c = modal_sig;
            let mut reload_c = reload_sig;
            let edit_opt = edit_id_sig.read().clone();
            let cid_clone = cid.clone();

            spawn(async move {
                if let Some(ref edit_id) = edit_opt {
                    let req = UpdateInventoryItemRequest {
                        clinic_id: cid_clone,
                        item_type: it,
                        name: n,
                        unit_type: ut_s.read().clone(),
                        current_stock: cs,
                        min_stock: ms,
                        cost_price_cents: cost_cents,
                        manufacturer: mfg_val,
                        attachments: vec![],
                        expiration_date: exp_val,
                        batch_number: bch_val,
                        serial_number: None,
                        warranty_until: None,
                        next_maintenance_date: None,
                        equipment_status: None,
                    };
                    match StockApi::update_item(edit_id, req).await {
                        Ok(_) => {
                            toast_resp.show("Item atualizado no estoque!", ToastVariant::Success);
                            modal_c.set(false);
                            reload_c.set(reload_c() + 1);
                        }
                        Err(err) => toast_resp.show(err, ToastVariant::Error),
                    }
                } else {
                    let req = CreateInventoryItemRequest {
                        clinic_id: cid_clone,
                        item_type: it,
                        name: n,
                        unit_type: ut_s.read().clone(),
                        current_stock: cs,
                        min_stock: ms,
                        cost_price_cents: cost_cents,
                        manufacturer: mfg_val,
                        attachments: vec![],
                        expiration_date: exp_val,
                        batch_number: bch_val,
                        serial_number: None,
                        warranty_until: None,
                        next_maintenance_date: None,
                        equipment_status: None,
                    };
                    match StockApi::create_item(req).await {
                        Ok(_) => {
                            toast_resp.show("Item cadastrado com sucesso!", ToastVariant::Success);
                            modal_c.set(false);
                            reload_c.set(reload_c() + 1);
                        }
                        Err(err) => toast_resp.show(err, ToastVariant::Error),
                    }
                }
            });
        }
    };

    let handle_save_movement = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut modal_sig = show_movement_modal;
        let mut reload_sig = reload_trigger;
        let it_sig = movement_item_id.clone();
        let mt_sig = movement_type.clone();
        let qty_sig = movement_qty_str.clone();
        let reason_sig = movement_reason.clone();

        move |_| {
            let iid = it_sig.read().trim().to_string();
            if iid.is_empty() {
                toast_c.show("Selecione o item para movimentação.", ToastVariant::Error);
                return;
            }

            let qty: i32 = qty_sig.read().parse().unwrap_or(1);
            let mtype = match mt_sig.read().as_str() {
                "EXIT" => MovementType::ManualOut,
                "ADJUSTMENT" => MovementType::Adjustment,
                _ => MovementType::PurchaseIn,
            };

            let req = CreateStockMovementRequest {
                clinic_id: cid.clone(),
                item_id: iid,
                quantity_change: if mtype == MovementType::ManualOut { -qty } else { qty },
                movement_type: mtype,
                unit_cost_cents: None,
                invoice_number: None,
                notes: if reason_sig.read().is_empty() { None } else { Some(reason_sig.read().clone()) },
            };

            let mut toast_resp = toast_c.clone();
            let mut modal_c = modal_sig;
            let mut reload_c = reload_sig;

            spawn(async move {
                match StockApi::create_movement(req).await {
                    Ok(_) => {
                        toast_resp.show("Movimentação registrada!", ToastVariant::Success);
                        modal_c.set(false);
                        reload_c.set(reload_c() + 1);
                    }
                    Err(err) => toast_resp.show(err, ToastVariant::Error),
                }
            });
        }
    };

    let filtered_items: Vec<InventoryItem> = stock_items.read().iter().filter(|it| {
        let tf = type_filter.read().clone();
        if tf == "material" && it.item_type != ItemType::Material { return false; }
        if tf == "chemical" && it.item_type != ItemType::Chemical { return false; }
        if tf == "equipment" && it.item_type != ItemType::Equipment { return false; }

        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        it.name.to_lowercase().contains(&q)
            || it.manufacturer.as_deref().unwrap_or("").to_lowercase().contains(&q)
            || it.batch_number.as_deref().unwrap_or("").to_lowercase().contains(&q)
    }).cloned().collect();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "stock-page",
            div { class: "stock-header-row",
                div {
                    h1 { class: "stock-title", "Inventário & Controle de Estoque" }
                    p { style: "font-size: 13.5px; color: #94a3b8; margin: 4px 0 0 0;",
                        "Gerencie insumos clínicos, medicamentos, equipamentos e alertas de reposição."
                    }
                }
            }

            StockToolbar {
                search_query,
                type_filter,
                on_new_item: move |_| {
                    editing_item_id.set(None);
                    name.set(String::new());
                    manufacturer.set(String::new());
                    cost_price_str.set("0.00".to_string());
                    current_stock_str.set("10".to_string());
                    min_stock_str.set("5".to_string());
                    expiration_date.set(String::new());
                    batch_number.set(String::new());
                    show_new_item_modal.set(true);
                },
                on_movement: move |_| {
                    movement_item_id.set(String::new());
                    movement_qty_str.set("1".to_string());
                    movement_reason.set(String::new());
                    show_movement_modal.set(true);
                },
            }

            StockTable {
                items: filtered_items,
                on_edit: move |iid: String| {
                    if let Some(item) = stock_items.read().iter().find(|i| i.id == iid) {
                        editing_item_id.set(Some(iid));
                        name.set(item.name.clone());
                        unit_type.set(item.unit_type.clone());
                        current_stock_str.set(item.current_stock.to_string());
                        min_stock_str.set(item.min_stock.to_string());
                        manufacturer.set(item.manufacturer.clone().unwrap_or_default());
                        cost_price_str.set(format!("{:.2}", item.cost_price_cents as f64 / 100.0));
                        expiration_date.set(item.expiration_date.clone().unwrap_or_default());
                        batch_number.set(item.batch_number.clone().unwrap_or_default());
                        show_new_item_modal.set(true);
                    }
                },
                on_movement: move |iid: String| {
                    movement_item_id.set(iid);
                    movement_qty_str.set("1".to_string());
                    movement_reason.set(String::new());
                    show_movement_modal.set(true);
                },
                on_delete: move |iid: String| {
                    let mut toast_d = toast.clone();
                    let mut reload_c = reload_trigger;
                    spawn(async move {
                        if let Ok(_) = StockApi::delete_item(&iid).await {
                            toast_d.show("Item removido do estoque.", ToastVariant::Success);
                            reload_c.set(reload_c() + 1);
                        }
                    });
                },
            }

            ModalItem {
                is_open: show_new_item_modal(),
                is_editing: editing_item_id.read().is_some(),
                name,
                item_type,
                unit_type,
                current_stock: current_stock_str,
                min_stock: min_stock_str,
                cost_price: cost_price_str,
                manufacturer,
                expiration_date,
                batch_number,
                on_close: move |_| show_new_item_modal.set(false),
                on_submit: handle_save_item,
            }

            ModalMovement {
                is_open: show_movement_modal(),
                items: stock_items(),
                selected_item_id: movement_item_id,
                movement_type,
                quantity: movement_qty_str,
                reason: movement_reason,
                on_close: move |_| show_movement_modal.set(false),
                on_submit: handle_save_movement,
            }
        }
    }
}
