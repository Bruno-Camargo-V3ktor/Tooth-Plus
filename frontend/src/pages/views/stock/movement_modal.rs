//! # Modal de Registro de Movimentação de Estoque (Frontend)

use crate::api::create_stock_movement;
use crate::components::icons::{IconAlertTriangle, IconRefresh, IconUpload};
use dioxus::prelude::*;
use shared::stock::{CreateStockMovementRequest, InventoryItem, MovementType};

fn str_to_mov_type(s: &str) -> MovementType {
    match s {
        "manual_out" => MovementType::ManualOut,
        "appointment_out" => MovementType::AppointmentOut,
        "adjustment" => MovementType::Adjustment,
        "loss" => MovementType::Loss,
        _ => MovementType::PurchaseIn,
    }
}

#[component]
pub fn StockMovementModal(
    token: String,
    clinic_id: String,
    items: Vec<InventoryItem>,
    target_item: Option<InventoryItem>,
    is_open: Signal<bool>,
    reload_counter: Signal<i32>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    if !is_open() {
        return rsx! {};
    }

    let default_item_id = target_item
        .as_ref()
        .map(|i| i.id.clone())
        .or_else(|| items.first().map(|i| i.id.clone()))
        .unwrap_or_default();

    let mut form_item_id = use_signal(|| default_item_id);
    let mut form_mov_type = use_signal(|| "purchase_in".to_string());
    let mut form_qty = use_signal(|| 1);
    let mut form_unit_cost = use_signal(|| "R$ 0,00".to_string());
    let mut form_invoice = use_signal(String::new);
    let mut form_notes = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let item_id = form_item_id();
        if item_id.is_empty() {
            let mut err = error_toast;
            err.set(Some("Selecione o item de estoque.".into()));
            return;
        }

        let qty: i32 = form_qty();
        if qty <= 0 {
            let mut err = error_toast;
            err.set(Some("A quantidade deve ser maior que zero.".into()));
            return;
        }

        let mov_type = str_to_mov_type(&form_mov_type());
        let qty_change: i32 = if mov_type == MovementType::PurchaseIn || mov_type == MovementType::Adjustment {
            qty.abs()
        } else {
            -qty.abs()
        };

        let unit_cost_clean = form_unit_cost()
            .replace("R$", "")
            .replace(".", "")
            .replace(",", ".")
            .trim()
            .to_string();
        let unit_cost_cents = unit_cost_clean
            .parse::<f64>()
            .map(|v| (v * 100.0).round() as i64)
            .ok();

        let invoice_opt = if form_invoice().trim().is_empty() {
            None
        } else {
            Some(form_invoice().trim().to_string())
        };

        let notes_opt = if form_notes().trim().is_empty() {
            None
        } else {
            Some(form_notes().trim().to_string())
        };

        let req = CreateStockMovementRequest {
            clinic_id: cid.clone(),
            item_id: item_id.clone(),
            movement_type: mov_type,
            quantity_change: qty_change,
            unit_cost_cents,
            invoice_number: invoice_opt,
            notes: notes_opt,
        };

        let t = tok.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_counter;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        sub_sig.set(true);
        spawn(async move {
            match create_stock_movement(&t, &item_id, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Movimentação registrada!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao registrar movimentação: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal stock-custom-modal",
                div { class: "settings-header",
                    h2 { class: "settings-title", "Registrar Movimentação de Estoque" }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }
                div { class: "settings-content",
                    div { class: "form-grid",
                        // 1. Item Selecionado
                        div { class: "input-group-wrapper full-width",
                            label { "Item *" }
                            select {
                                class: "modern-input-field modern-select",
                                value: "{form_item_id}",
                                onchange: move |e: FormEvent| form_item_id.set(e.value()),
                                for it in &items {
                                    option { value: "{it.id}", "{it.name} (Atual: {it.current_stock} {it.unit_type})" }
                                }
                            }
                        }

                        // 2. Tipo de Movimentação (Cards com botões selecionáveis)
                        div { class: "input-group-wrapper full-width",
                            label { "Tipo de Movimentação *" }
                            div { class: "stock-modal-type-grid",
                                button {
                                    r#type: "button",
                                    class: if form_mov_type() == "purchase_in" { "stock-type-card active" } else { "stock-type-card" },
                                    onclick: move |_| form_mov_type.set("purchase_in".to_string()),
                                    span { class: "type-card-arrow-icon", "↓" }
                                    span { "Entrada / Compra" }
                                }
                                button {
                                    r#type: "button",
                                    class: if form_mov_type() == "manual_out" { "stock-type-card active" } else { "stock-type-card" },
                                    onclick: move |_| form_mov_type.set("manual_out".to_string()),
                                    span { class: "type-card-arrow-icon", "↑" }
                                    div { class: "type-card-text",
                                        span { "Saída Manual" }
                                        span { class: "type-card-sub", "(Consumo)" }
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: if form_mov_type() == "loss" { "stock-type-card active" } else { "stock-type-card" },
                                    onclick: move |_| form_mov_type.set("loss".to_string()),
                                    IconAlertTriangle { size: 14, color: "currentColor".to_string() }
                                    span { "Avaria / Perda" }
                                }
                                button {
                                    r#type: "button",
                                    class: if form_mov_type() == "adjustment" { "stock-type-card active" } else { "stock-type-card" },
                                    onclick: move |_| form_mov_type.set("adjustment".to_string()),
                                    IconRefresh { size: 14, color: "currentColor".to_string() }
                                    span { "Ajuste de Balanço" }
                                }
                            }
                        }

                        // 3. Quantidade e Custo Unitário
                        div { class: "input-group-wrapper",
                            label { "Quantidade *" }
                            input {
                                class: "modern-input-field font-mono",
                                r#type: "number",
                                min: "1",
                                value: "{form_qty}",
                                oninput: move |e: FormEvent| form_qty.set(e.value().parse::<i32>().unwrap_or(1))
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Custo Unitário (R$)" }
                            input {
                                class: "modern-input-field font-mono",
                                placeholder: "R$ 0,00",
                                value: "{form_unit_cost}",
                                oninput: move |e: FormEvent| form_unit_cost.set(e.value())
                            }
                        }

                        // 4. Nota Fiscal / Recibo e Observações
                        div { class: "input-group-wrapper",
                            label { "Nota Fiscal / Recibo" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: NF-10492...",
                                value: "{form_invoice}",
                                oninput: move |e: FormEvent| form_invoice.set(e.value())
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Observações" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: Reposição periódica, quebra de frasc...",
                                value: "{form_notes}",
                                oninput: move |e: FormEvent| form_notes.set(e.value())
                            }
                        }

                        // 5. Upload de Nota Fiscal
                        div { class: "input-group-wrapper full-width",
                            label { "Comprovante / PDF de Nota Fiscal" }
                            div { class: "stock-upload-dropzone",
                                IconUpload { size: 16, color: "#0052cc".to_string() }
                                span { class: "stock-upload-title", "Clique para anexar arquivo da NF" }
                            }
                        }
                    }
                }
                div { class: "modal-footer-actions",
                    button { class: "btn-secondary", onclick: move |_| is_open.set(false), "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        span { if is_submitting() { "Confirmando..." } else { "Confirmar Movimentação" } }
                    }
                }
            }
        }
    }
}
