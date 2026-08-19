//! # Modal de Cadastro e Edição de Item de Estoque (Frontend)

use crate::api::{create_stock_item, update_stock_item};
use crate::components::icons::{IconBox, IconFlask, IconTool, IconUpload};
use dioxus::prelude::*;
use shared::stock::{
    CreateInventoryItemRequest, EquipmentStatus, InventoryItem, ItemType, UpdateInventoryItemRequest,
};

#[component]
pub fn StockItemModal(
    token: String,
    clinic_id: String,
    editing_item: Option<InventoryItem>,
    is_open: Signal<bool>,
    reload_counter: Signal<i32>,
    toast_msg: Signal<Option<String>>,
) -> Element {
    if !is_open() {
        return rsx! {};
    }

    let is_editing = editing_item.is_some();
    let modal_title = if is_editing {
        "Editar Item de Estoque"
    } else {
        "Cadastrar Novo Item / Equipamento"
    };

    let submit_label = if is_editing {
        "Salvar Alterações"
    } else {
        "Cadastrar Item"
    };

    let initial_type = editing_item
        .as_ref()
        .map(|i| match i.item_type {
            ItemType::Material => "material",
            ItemType::Chemical => "chemical",
            ItemType::Equipment => "equipment",
        })
        .unwrap_or("material");

    let initial_eq_status = editing_item
        .as_ref()
        .and_then(|i| i.equipment_status)
        .map(|s| match s {
            EquipmentStatus::Active => "active",
            EquipmentStatus::InMaintenance => "in_maintenance",
            EquipmentStatus::Broken => "broken",
        })
        .unwrap_or("active");

    let mut form_name = use_signal(|| {
        editing_item
            .as_ref()
            .map(|i| i.name.clone())
            .unwrap_or_default()
    });
    let mut form_type = use_signal(|| initial_type.to_string());
    let mut form_unit = use_signal(|| {
        editing_item
            .as_ref()
            .map(|i| i.unit_type.clone())
            .unwrap_or_else(|| "unidade".into())
    });
    let mut form_current_stock =
        use_signal(|| editing_item.as_ref().map(|i| i.current_stock).unwrap_or(0));
    let mut form_min_stock = use_signal(|| editing_item.as_ref().map(|i| i.min_stock).unwrap_or(5));
    let mut form_cost = use_signal(|| {
        editing_item
            .as_ref()
            .map(|i| format!("R$ {:.2}", (i.cost_price_cents as f64) / 100.0).replace('.', ","))
            .unwrap_or_else(|| "R$ 0,00".into())
    });
    let mut form_manufacturer = use_signal(|| {
        editing_item
            .as_ref()
            .and_then(|i| i.manufacturer.clone())
            .unwrap_or_default()
    });
    let mut form_batch = use_signal(|| {
        editing_item
            .as_ref()
            .and_then(|i| i.batch_number.clone())
            .unwrap_or_default()
    });
    let mut form_expiration = use_signal(|| {
        editing_item
            .as_ref()
            .and_then(|i| i.expiration_date.clone())
            .map(|d| d.chars().take(10).collect::<String>())
            .unwrap_or_default()
    });
    let mut form_serial = use_signal(|| {
        editing_item
            .as_ref()
            .and_then(|i| i.serial_number.clone())
            .unwrap_or_default()
    });
    let mut form_warranty = use_signal(|| {
        editing_item
            .as_ref()
            .and_then(|i| i.warranty_until.clone())
            .map(|d| d.chars().take(10).collect::<String>())
            .unwrap_or_default()
    });
    let mut form_maintenance = use_signal(|| {
        editing_item
            .as_ref()
            .and_then(|i| i.next_maintenance_date.clone())
            .map(|d| d.chars().take(10).collect::<String>())
            .unwrap_or_default()
    });
    let mut form_eq_status = use_signal(|| initial_eq_status.to_string());
    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let cid = clinic_id.clone();
    let item_opt = editing_item.clone();

    let mut handle_submit = move |_| {
        let name = form_name().trim().to_string();
        if name.is_empty() {
            let mut toast = toast_msg;
            toast.set(Some("Informe o nome do item.".into()));
            return;
        }

        let item_type = match form_type().as_str() {
            "chemical" => ItemType::Chemical,
            "equipment" => ItemType::Equipment,
            _ => ItemType::Material,
        };

        let cost_clean = form_cost()
            .replace("R$", "")
            .replace(".", "")
            .replace(",", ".")
            .trim()
            .to_string();
        let cost_cents = match cost_clean.parse::<f64>() {
            Ok(v) => (v * 100.0).round() as i64,
            Err(_) => 0,
        };

        let manufacturer_opt = if form_manufacturer().trim().is_empty() {
            None
        } else {
            Some(form_manufacturer().trim().to_string())
        };

        let batch_opt = if form_batch().trim().is_empty() {
            None
        } else {
            Some(form_batch().trim().to_string())
        };

        let expiration_opt = if form_expiration().trim().is_empty() {
            None
        } else {
            Some(form_expiration().trim().to_string())
        };

        let serial_opt = if form_serial().trim().is_empty() {
            None
        } else {
            Some(form_serial().trim().to_string())
        };

        let warranty_opt = if form_warranty().trim().is_empty() {
            None
        } else {
            Some(form_warranty().trim().to_string())
        };

        let maintenance_opt = if form_maintenance().trim().is_empty() {
            None
        } else {
            Some(form_maintenance().trim().to_string())
        };

        let eq_status_opt = if item_type == ItemType::Equipment {
            match form_eq_status().as_str() {
                "in_maintenance" => Some(EquipmentStatus::InMaintenance),
                "broken" => Some(EquipmentStatus::Broken),
                _ => Some(EquipmentStatus::Active),
            }
        } else {
            None
        };

        let t = tok.clone();
        let c = cid.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_counter;
        let mut sub_sig = is_submitting;
        let mut toast = toast_msg;
        let item_opt_clone = item_opt.clone();

        sub_sig.set(true);
        spawn(async move {
            if let Some(ref existing) = item_opt_clone {
                let req = UpdateInventoryItemRequest {
                    clinic_id: c.clone(),
                    item_type,
                    name,
                    unit_type: form_unit(),
                    current_stock: form_current_stock(),
                    min_stock: form_min_stock(),
                    cost_price_cents: cost_cents,
                    manufacturer: manufacturer_opt,
                    attachments: vec![],
                    expiration_date: expiration_opt,
                    batch_number: batch_opt,
                    serial_number: serial_opt,
                    warranty_until: warranty_opt,
                    next_maintenance_date: maintenance_opt,
                    equipment_status: eq_status_opt,
                };
                match update_stock_item(&t, &existing.id, req).await {
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
                    clinic_id: c.clone(),
                    item_type,
                    name,
                    unit_type: form_unit(),
                    current_stock: form_current_stock(),
                    min_stock: form_min_stock(),
                    cost_price_cents: cost_cents,
                    manufacturer: manufacturer_opt,
                    attachments: vec![],
                    expiration_date: expiration_opt,
                    batch_number: batch_opt,
                    serial_number: serial_opt,
                    warranty_until: warranty_opt,
                    next_maintenance_date: maintenance_opt,
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
            div { class: "action-modal stock-custom-modal",
                div { class: "settings-header",
                    h2 { class: "settings-title", "{modal_title}" }
                    button { class: "close-btn", onclick: move |_| is_open.set(false), "×" }
                }
                div { class: "settings-content",
                    div { class: "form-grid",
                        // 1. Categoria do Item (Cards Selecionáveis)
                        div { class: "input-group-wrapper full-width",
                            label { "Categoria do Item *" }
                            div { class: "stock-category-selector-grid",
                                button {
                                    r#type: "button",
                                    class: if form_type() == "material" { "stock-category-card active" } else { "stock-category-card" },
                                    onclick: move |_| form_type.set("material".to_string()),
                                    IconBox { size: 16, color: "currentColor".to_string() }
                                    span { "Material / Insumo" }
                                }
                                button {
                                    r#type: "button",
                                    class: if form_type() == "chemical" { "stock-category-card active" } else { "stock-category-card" },
                                    onclick: move |_| form_type.set("chemical".to_string()),
                                    IconFlask { size: 16, color: "currentColor".to_string() }
                                    span { "Químico / Cosmético" }
                                }
                                button {
                                    r#type: "button",
                                    class: if form_type() == "equipment" { "stock-category-card active" } else { "stock-category-card" },
                                    onclick: move |_| form_type.set("equipment".to_string()),
                                    IconTool { size: 16, color: "currentColor".to_string() }
                                    div { class: "category-card-text",
                                        span { "Equipamento / " }
                                        span { "Patrimônio" }
                                    }
                                }
                            }
                        }

                        // 2. Nome e Fabricante
                        div { class: "input-group-wrapper",
                            label { "Nome do Item / Equipamento *" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: Resina Z350 XT, Autoclave Vitale 12L...",
                                value: "{form_name}",
                                oninput: move |e| form_name.set(e.value())
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Fabricante / Marca" }
                            input {
                                class: "modern-input-field",
                                placeholder: "Ex: 3M, Cristófoli, DFL...",
                                value: "{form_manufacturer}",
                                oninput: move |e| form_manufacturer.set(e.value())
                            }
                        }

                        // 3. Unidade de Medida e Preço de Custo
                        div { class: "input-group-wrapper",
                            label { "Unidade de Medida" }
                            select {
                                class: "modern-input-field modern-select",
                                value: "{form_unit}",
                                onchange: move |e: FormEvent| form_unit.set(e.value()),
                                option { value: "unidade", "Unidade (un)" }
                                option { value: "par", "Par (pr)" }
                                option { value: "caixa", "Caixa (cx)" }
                                option { value: "frasco", "Frasco (fr)" }
                                option { value: "pacote", "Pacote (pct)" }
                                option { value: "tubete", "Tubete (tb)" }
                                option { value: "grama", "Grama (g)" }
                                option { value: "ml", "Mililitro (ml)" }
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Preço de Custo Unitário (R$)" }
                            input {
                                class: "modern-input-field font-mono",
                                placeholder: "R$ 0,00",
                                value: "{form_cost}",
                                oninput: move |e| form_cost.set(e.value())
                            }
                        }

                        // 4. Estoque Inicial e Mínimo
                        div { class: "input-group-wrapper",
                            label { "Estoque Inicial / Atual" }
                            input {
                                class: "modern-input-field font-mono",
                                r#type: "number",
                                min: "0",
                                value: "{form_current_stock}",
                                oninput: move |e: FormEvent| form_current_stock.set(e.value().parse::<i32>().unwrap_or(0))
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Estoque Mínimo de Segurança" }
                            input {
                                class: "modern-input-field font-mono",
                                r#type: "number",
                                min: "1",
                                value: "{form_min_stock}",
                                oninput: move |e: FormEvent| form_min_stock.set(e.value().parse::<i32>().unwrap_or(1))
                            }
                        }

                        // 5. Campos Condicionais de Químico
                        if form_type() == "chemical" {
                            div { class: "input-group-wrapper",
                                label { "Lote de Fabricação" }
                                input {
                                    class: "modern-input-field",
                                    placeholder: "Ex: LT-2026B...",
                                    value: "{form_batch}",
                                    oninput: move |e| form_batch.set(e.value())
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Data de Validade *" }
                                input {
                                    class: "modern-input-field",
                                    r#type: "date",
                                    value: "{form_expiration}",
                                    oninput: move |e| form_expiration.set(e.value())
                                }
                            }
                        }

                        // 6. Campos Condicionais de Equipamento
                        if form_type() == "equipment" {
                            div { class: "input-group-wrapper",
                                label { "Número de Série (S/N)" }
                                input {
                                    class: "modern-input-field",
                                    placeholder: "Ex: SCH-99881...",
                                    value: "{form_serial}",
                                    oninput: move |e| form_serial.set(e.value())
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Status Operacional" }
                                select {
                                    class: "modern-input-field modern-select",
                                    value: "{form_eq_status}",
                                    onchange: move |e: FormEvent| form_eq_status.set(e.value()),
                                    option { value: "active", "Operacional / Ativo" }
                                    option { value: "in_maintenance", "Em Manutenção" }
                                    option { value: "broken", "Inoperante / Danificado" }
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Garantia até" }
                                input {
                                    class: "modern-input-field",
                                    r#type: "date",
                                    value: "{form_warranty}",
                                    oninput: move |e| form_warranty.set(e.value())
                                }
                            }
                            div { class: "input-group-wrapper",
                                label { "Próxima Manutenção / Calibração" }
                                input {
                                    class: "modern-input-field",
                                    r#type: "date",
                                    value: "{form_maintenance}",
                                    oninput: move |e| form_maintenance.set(e.value())
                                }
                            }
                        }

                        // 7. Área de Upload de Documentos
                        div { class: "input-group-wrapper full-width",
                            label { "Documentos & Comprovantes (Fotos / PDFs / Nota Fiscal)" }
                            div { class: "stock-upload-dropzone",
                                IconUpload { size: 18, color: "#0052cc".to_string() }
                                span { class: "stock-upload-title", "Clique para anexar Foto ou PDF" }
                                span { class: "stock-upload-sub", "Suporta imagens (PNG, JPG) e documentos PDF de notas fiscais, manuais ou certificados" }
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
                        span { if is_submitting() { "Salvando..." } else { "{submit_label}" } }
                    }
                }
            }
        }
    }
}
