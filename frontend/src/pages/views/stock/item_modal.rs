//! # Modal de Cadastro e Edição de Itens de Estoque (Frontend)
//!
//! Controla os formulários de inserção e atualização de materiais odontológicos,
//! produtos químicos perecíveis e equipamentos clínicos com calibração.

use crate::api::{create_stock_item, update_stock_item};
use crate::components::icons::IconCheck;
use dioxus::prelude::*;
use shared::stock::{
    CreateInventoryItemRequest, EquipmentStatus, InventoryItem, ItemType,
    UpdateInventoryItemRequest,
};

/// Modal para inserção ou modificação de itens do estoque da clínica.
#[component]
pub fn StockItemModal(
    token: String,
    clinic_id: String,
    editing_item: Option<InventoryItem>,
    is_open: Signal<bool>,
    reload_counter: Signal<usize>,
    toast_msg: Signal<Option<String>>,
) -> Element {
    let initial_item = editing_item.clone();
    let is_editing = initial_item.is_some();
    let edit_id = initial_item.as_ref().map(|i| i.id.clone()).unwrap_or_default();

    let mut form_name = use_signal(|| initial_item.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let mut form_type = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| match i.item_type {
                ItemType::Chemical => "chemical",
                ItemType::Equipment => "equipment",
                _ => "material",
            })
            .unwrap_or("material")
            .to_string()
    });
    let mut form_unit = use_signal(|| initial_item.as_ref().map(|i| i.unit_type.clone()).unwrap_or_else(|| "unidade".into()));
    let mut form_current_stock = use_signal(|| initial_item.as_ref().map(|i| i.current_stock).unwrap_or(0));
    let mut form_min_stock = use_signal(|| initial_item.as_ref().map(|i| i.min_stock).unwrap_or(5));
    let mut form_cost = use_signal(|| {
        initial_item
            .as_ref()
            .map(|i| format!("{:.2}", (i.cost_price_cents as f64) / 100.0))
            .unwrap_or_else(|| "0,00".into())
    });
    let mut form_manufacturer = use_signal(|| initial_item.as_ref().and_then(|i| i.manufacturer.clone()).unwrap_or_default());
    let mut form_batch = use_signal(|| initial_item.as_ref().and_then(|i| i.batch_number.clone()).unwrap_or_default());
    let mut form_serial = use_signal(|| initial_item.as_ref().and_then(|i| i.serial_number.clone()).unwrap_or_default());
    let mut form_expiration = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.expiration_date.clone())
            .map(|d| d.chars().take(10).collect::<String>())
            .unwrap_or_default()
    });
    let mut form_warranty = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.warranty_until.clone())
            .map(|d| d.chars().take(10).collect::<String>())
            .unwrap_or_default()
    });
    let mut form_maintenance = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.next_maintenance_date.clone())
            .map(|d| d.chars().take(10).collect::<String>())
            .unwrap_or_default()
    });
    let mut form_eq_status = use_signal(|| {
        initial_item
            .as_ref()
            .and_then(|i| i.equipment_status.clone())
            .map(|s| match s {
                EquipmentStatus::InMaintenance => "in_maintenance",
                EquipmentStatus::Broken => "broken",
                _ => "active",
            })
            .unwrap_or("active")
            .to_string()
    });
    let mut is_submitting = use_signal(|| false);

    if !is_open() {
        return rsx! {};
    }

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let name = form_name().trim().to_string();
        if name.is_empty() {
            let mut toast = toast_msg;
            toast.set(Some("Informe o nome do item.".into()));
            return;
        }

        let item_type_enum = match form_type().as_str() {
            "chemical" => ItemType::Chemical,
            "equipment" => ItemType::Equipment,
            _ => ItemType::Material,
        };

        let cost_clean = form_cost().replace("R$", "").replace(".", "").replace(",", "").trim().to_string();
        let cost_cents = cost_clean.parse::<i64>().unwrap_or(0);

        let exp_opt = if form_expiration().trim().is_empty() {
            None
        } else {
            Some(format!("{}T00:00:00Z", form_expiration().trim()))
        };

        let war_opt = if form_warranty().trim().is_empty() {
            None
        } else {
            Some(format!("{}T00:00:00Z", form_warranty().trim()))
        };

        let maint_opt = if form_maintenance().trim().is_empty() {
            None
        } else {
            Some(format!("{}T00:00:00Z", form_maintenance().trim()))
        };

        let eq_status_opt = if item_type_enum == ItemType::Equipment {
            Some(match form_eq_status().as_str() {
                "in_maintenance" => EquipmentStatus::InMaintenance,
                "broken" => EquipmentStatus::Broken,
                _ => EquipmentStatus::Active,
            })
        } else {
            None
        };

        let t = tok.clone();
        let c = cid.clone();
        let e_id = edit_id.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_counter;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;

        sub_sig.set(true);
        spawn(async move {
            if is_editing {
                let req = UpdateInventoryItemRequest {
                    clinic_id: c,
                    item_type: item_type_enum,
                    name,
                    unit_type: form_unit(),
                    current_stock: form_current_stock(),
                    min_stock: form_min_stock(),
                    cost_price_cents: cost_cents,
                    manufacturer: if form_manufacturer().trim().is_empty() { None } else { Some(form_manufacturer().trim().to_string()) },
                    attachments: vec![],
                    expiration_date: exp_opt,
                    batch_number: if form_batch().trim().is_empty() { None } else { Some(form_batch().trim().to_string()) },
                    serial_number: if form_serial().trim().is_empty() { None } else { Some(form_serial().trim().to_string()) },
                    warranty_until: war_opt,
                    next_maintenance_date: maint_opt,
                    equipment_status: eq_status_opt,
                };
                match update_stock_item(&t, &e_id, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        rel_sig.set(rel_sig() + 1);
                        toast.set(Some("Item atualizado com sucesso!".into()));
                    }
                    Err(e) => {
                        toast.set(Some(format!("Erro ao atualizar item: {}", e)));
                    }
                }
            } else {
                let req = CreateInventoryItemRequest {
                    clinic_id: c,
                    item_type: item_type_enum,
                    name,
                    unit_type: form_unit(),
                    current_stock: form_current_stock(),
                    min_stock: form_min_stock(),
                    cost_price_cents: cost_cents,
                    manufacturer: if form_manufacturer().trim().is_empty() { None } else { Some(form_manufacturer().trim().to_string()) },
                    attachments: vec![],
                    expiration_date: exp_opt,
                    batch_number: if form_batch().trim().is_empty() { None } else { Some(form_batch().trim().to_string()) },
                    serial_number: if form_serial().trim().is_empty() { None } else { Some(form_serial().trim().to_string()) },
                    warranty_until: war_opt,
                    next_maintenance_date: maint_opt,
                    equipment_status: eq_status_opt,
                };
                match create_stock_item(&t, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        rel_sig.set(rel_sig() + 1);
                        toast.set(Some("Item cadastrado com sucesso!".into()));
                    }
                    Err(e) => {
                        toast.set(Some(format!("Erro ao cadastrar item: {}", e)));
                    }
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal modal-large",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", if is_editing { "Editar Item de Estoque" } else { "Novo Item de Estoque" } }
                        p { class: "modal-subtitle", "Cadastre materiais, medicamentos ou instrumentais com controle de estoque mínimo." }
                    }
                    button { class: "modal-close", onclick: move |_| { let mut o = is_open; o.set(false); }, "×" }
                }
                div { class: "modal-body scrollable",
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Nome do Item *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Resina Composta A2, Anestésico Lidocaína...",
                                value: "{form_name}",
                                oninput: move |e| form_name.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Categoria do Item *" }
                            select {
                                class: "form-input",
                                value: "{form_type}",
                                onchange: move |e| form_type.set(e.value()),
                                option { value: "material", "Material Odontológico Geral" }
                                option { value: "chemical", "Produto Químico / Medicamento" }
                                option { value: "equipment", "Equipamento / Instrumental" }
                            }
                        }
                    }

                    div { class: "form-grid-3",
                        div { class: "form-group",
                            label { "Unidade de Medida" }
                            select {
                                class: "form-input",
                                value: "{form_unit}",
                                onchange: move |e| form_unit.set(e.value()),
                                option { value: "unidade", "Unidade (un)" }
                                option { value: "caixa", "Caixa (cx)" }
                                option { value: "frasco", "Frasco (fr)" }
                                option { value: "pacote", "Pacote (pct)" }
                                option { value: "tubete", "Tubete (tb)" }
                                option { value: "grama", "Grama (g)" }
                                option { value: "ml", "Mililitro (ml)" }
                            }
                        }
                        div { class: "form-group",
                            label { "Estoque Atual" }
                            input {
                                class: "form-input",
                                r#type: "number",
                                min: "0",
                                value: "{form_current_stock}",
                                oninput: move |e| form_current_stock.set(e.value().parse::<i32>().unwrap_or(0))
                            }
                        }
                        div { class: "form-group",
                            label { "Estoque Mínimo de Alerta" }
                            input {
                                class: "form-input",
                                r#type: "number",
                                min: "1",
                                value: "{form_min_stock}",
                                oninput: move |e| form_min_stock.set(e.value().parse::<i32>().unwrap_or(1))
                            }
                        }
                    }

                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Preço de Custo Unitário (R$)" }
                            input {
                                class: "form-input",
                                placeholder: "0,00",
                                value: "{form_cost}",
                                oninput: move |e| form_cost.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Fabricante / Marca" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: 3M, Dentsply, FGM...",
                                value: "{form_manufacturer}",
                                oninput: move |e| form_manufacturer.set(e.value())
                            }
                        }
                    }

                    if form_type() == "chemical" {
                        div { class: "form-section-title mt-4", "Rastreabilidade de Químicos & Medicamentos" }
                        div { class: "form-grid-2",
                            div { class: "form-group",
                                label { "Data de Validade *" }
                                input {
                                    class: "form-input",
                                    r#type: "date",
                                    value: "{form_expiration}",
                                    oninput: move |e| form_expiration.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Número do Lote" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: LOT-2026-X9",
                                    value: "{form_batch}",
                                    oninput: move |e| form_batch.set(e.value())
                                }
                            }
                        }
                    }

                    if form_type() == "equipment" {
                        div { class: "form-section-title mt-4", "Manutenção e Patrimônio de Equipamentos" }
                        div { class: "form-grid-2",
                            div { class: "form-group",
                                label { "Número de Série" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: SN-8839210",
                                    value: "{form_serial}",
                                    oninput: move |e| form_serial.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Status Operacional" }
                                select {
                                    class: "form-input",
                                    value: "{form_eq_status}",
                                    onchange: move |e| form_eq_status.set(e.value()),
                                    option { value: "active", "Operacional / Ativo" }
                                    option { value: "in_maintenance", "Em Manutenção" }
                                    option { value: "broken", "Inoperante / Danificado" }
                                }
                            }
                            div { class: "form-group",
                                label { "Garantia até" }
                                input {
                                    class: "form-input",
                                    r#type: "date",
                                    value: "{form_warranty}",
                                    oninput: move |e| form_warranty.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Próxima Revisão Preventiva" }
                                input {
                                    class: "form-input",
                                    r#type: "date",
                                    value: "{form_maintenance}",
                                    oninput: move |e| form_maintenance.set(e.value())
                                }
                            }
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
                        span { if is_submitting() { "Salvando..." } else { "Salvar Item" } }
                    }
                }
            }
        }
    }
}
