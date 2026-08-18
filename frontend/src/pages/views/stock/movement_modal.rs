//! # Modal de Movimentação de Estoque (Frontend)
//!
//! Controla os lançamentos de entrada por compra, saídas manuais, perdas e
//! ajustes de saldo com justificativa e número de nota fiscal.

use crate::api::create_stock_movement;
use crate::components::icons::IconCheck;
use dioxus::prelude::*;
use shared::stock::{CreateStockMovementRequest, InventoryItem, MovementType};

/// Converte enum `MovementType` em string.
fn mov_type_to_str(m: &MovementType) -> &'static str {
    match m {
        MovementType::PurchaseIn => "purchase_in",
        MovementType::ManualOut => "manual_out",
        MovementType::AppointmentOut => "appointment_out",
        MovementType::Adjustment => "adjustment",
        MovementType::Loss => "loss",
    }
}

/// Converte string em enum `MovementType`.
fn str_to_mov_type(s: &str) -> MovementType {
    match s {
        "manual_out" => MovementType::ManualOut,
        "appointment_out" => MovementType::AppointmentOut,
        "adjustment" => MovementType::Adjustment,
        "loss" => MovementType::Loss,
        _ => MovementType::PurchaseIn,
    }
}

/// Modal para registro de movimentação de entrada/saída no estoque.
#[component]
pub fn StockMovementModal(
    token: String,
    clinic_id: String,
    items: Vec<InventoryItem>,
    target_item: Option<InventoryItem>,
    is_open: Signal<bool>,
    reload_counter: Signal<usize>,
    toast_msg: Signal<Option<String>>,
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
    let mut form_unit_cost = use_signal(|| "0,00".to_string());
    let mut form_invoice = use_signal(String::new);
    let mut form_notes = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let item_id = form_item_id();
        if item_id.is_empty() {
            let mut toast = toast_msg;
            toast.set(Some("Selecione o item de estoque.".into()));
            return;
        }

        let qty: i32 = form_qty();
        if qty == 0 {
            let mut toast = toast_msg;
            toast.set(Some("A quantidade deve ser diferente de zero.".into()));
            return;
        }

        let mov_type = str_to_mov_type(&form_mov_type());
        let qty_change: i32 = if mov_type == MovementType::PurchaseIn {
            qty.abs()
        } else {
            -qty.abs()
        };

        let cost_clean = form_unit_cost().replace("R$", "").replace(".", "").replace(",", "").trim().to_string();
        let cost_cents = if cost_clean.is_empty() {
            None
        } else {
            Some(cost_clean.parse::<i64>().unwrap_or(0))
        };

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
            quantity_change: qty_change,
            movement_type: mov_type,
            unit_cost_cents: cost_cents,
            invoice_number: invoice_opt,
            notes: notes_opt,
        };

        let t = tok.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_counter;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;

        sub_sig.set(true);
        spawn(async move {
            match create_stock_movement(&t, &item_id, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Movimentação registrada com sucesso!".into()));
                }
                Err(e) => {
                    toast.set(Some(format!("Erro ao registrar movimentação: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "Registrar Movimentação de Estoque" }
                        p { class: "modal-subtitle", "Entrada de compras, saídas manuais, perdas ou ajustes de balanço." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }
                div { class: "modal-body",
                    div { class: "form-group",
                        label { "Item de Estoque *" }
                        select {
                            class: "form-input",
                            value: "{form_item_id}",
                            onchange: move |e| form_item_id.set(e.value()),
                            for it in &items {
                                option { value: "{it.id}", "{it.name} (Atual: {it.current_stock} {it.unit_type})" }
                            }
                        }
                    }

                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Tipo de Movimentação *" }
                            select {
                                class: "form-input",
                                value: "{form_mov_type}",
                                onchange: move |e| form_mov_type.set(e.value()),
                                option { value: "purchase_in", "Entrada por Compra (+)" }
                                option { value: "manual_out", "Saída Manual / Consumo (-)" }
                                option { value: "loss", "Perda / Avaria / Descarte (-)" }
                                option { value: "adjustment", "Ajuste de Balanço" }
                            }
                        }
                        div { class: "form-group",
                            label { "Quantidade *" }
                            input {
                                class: "form-input",
                                r#type: "number",
                                min: "1",
                                value: "{form_qty}",
                                oninput: move |e| form_qty.set(e.value().parse::<i32>().unwrap_or(1))
                            }
                        }
                    }

                    if form_mov_type() == "purchase_in" {
                        div { class: "form-grid-2",
                            div { class: "form-group",
                                label { "Custo Unitário (R$)" }
                                input {
                                    class: "form-input",
                                    placeholder: "0,00",
                                    value: "{form_unit_cost}",
                                    oninput: move |e| form_unit_cost.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Nº da Nota Fiscal" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: NF-e 12345",
                                    value: "{form_invoice}",
                                    oninput: move |e| form_invoice.set(e.value())
                                }
                            }
                        }
                    }

                    div { class: "form-group",
                        label { "Observações / Justificativa" }
                        textarea {
                            class: "form-textarea",
                            placeholder: "Ex: Reposição de estoque semanal ou motivo de descarte...",
                            value: "{form_notes}",
                            oninput: move |e| form_notes.set(e.value())
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "btn-secondary", onclick: move |_| { let mut o = is_open; o.set(false); }, "Cancelar" }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        IconCheck { size: 16, color: "currentColor".to_string() }
                        span { if is_submitting() { "Registrando..." } else { "Salvar Movimentação" } }
                    }
                }
            }
        }
    }
}
