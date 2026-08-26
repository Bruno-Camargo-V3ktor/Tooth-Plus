use crate::components::modal::Modal;
use crate::pages::patients::components::tab_odontogram::TabOdontogram;
use shared::stock::{InventoryItem, ItemType};
use dioxus::prelude::*;

#[component]
pub fn TemplateModal(
    is_open: bool,
    inventory_items: Vec<InventoryItem>,
    name: Signal<String>,
    category: Signal<String>,
    description: Signal<String>,
    price_str: Signal<String>,
    duration_str: Signal<String>,
    materials_list: Signal<Vec<String>>,
    equipment_list: Signal<Vec<String>>,
    post_care: Signal<String>,
    target_teeth: Signal<Vec<String>>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let mut selected_material_id = use_signal(String::new);
    let mut selected_equipment_id = use_signal(String::new);
    let mut material_qty = use_signal(|| "1".to_string());

    let available_materials: Vec<InventoryItem> = inventory_items
        .iter()
        .filter(|i| i.item_type == ItemType::Material || i.item_type == ItemType::Chemical)
        .cloned()
        .collect();

    let available_equipment: Vec<InventoryItem> = inventory_items
        .iter()
        .filter(|i| i.item_type == ItemType::Equipment)
        .cloned()
        .collect();

    rsx! {
        Modal {
            title: "Cadastrar Procedimento no Catálogo".to_string(),
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                button {
                    r#type: "button",
                    class: "btn-modal-ghost",
                    onclick: move |_| on_close.call(()),
                    "Cancelar"
                }
                button {
                    r#type: "button",
                    class: "btn-modal-primary",
                    onclick: move |_| on_submit.call(()),
                    "Salvar Procedimento"
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 14px;",
                div { class: "form-field",
                    label { class: "form-label", "Nome do Procedimento *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Restauração em Resina Composta, Profilaxia & Raspagem...",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Categoria" }
                        select {
                            class: "form-select",
                            value: "{category}",
                            onchange: move |e| category.set(e.value()),
                            option { value: "Dentística", "Dentística & Estética" }
                            option { value: "Endodontia", "Endodontia (Canal)" }
                            option { value: "Cirurgia", "Cirurgia & Exodontia" }
                            option { value: "Periodontia", "Periodontia & Profilaxia" }
                            option { value: "Ortodontia", "Ortodontia" }
                            option { value: "Prótese", "Prótese & Implante" }
                            option { value: "Diagnóstico", "Diagnóstico & Consulta" }
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Preço Base Sugerido (R$) *" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            step: "0.01",
                            placeholder: "0.00",
                            value: "{price_str}",
                            oninput: move |e| price_str.set(e.value()),
                        }
                    }
                }

                div { class: "form-field",
                    label { class: "form-label", "Duração Estimada na Cadeira" }
                    select {
                        class: "form-select",
                        value: "{duration_str}",
                        onchange: move |e| duration_str.set(e.value()),
                        option { value: "15", "15 minutos" }
                        option { value: "30", "30 minutos" }
                        option { value: "45", "45 minutos" }
                        option { value: "60", "1 hora" }
                        option { value: "90", "1h30" }
                        option { value: "120", "2 horas" }
                    }
                }

                // Odontograma Gráfico Anatômico para seleção de dentes padrão
                div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; padding: 12px;",
                    TabOdontogram {
                        selected_teeth: target_teeth,
                        on_toggle_tooth: move |t: String| {
                            let mut list = target_teeth.read().clone();
                            if list.contains(&t) {
                                list.retain(|x| x != &t);
                            } else {
                                list.push(t);
                            }
                            target_teeth.set(list);
                        },
                    }
                }

                // Insumos e Materiais selecionados do Estoque
                div { class: "form-field",
                    label { class: "form-label", "📦 Insumos / Materiais Necessários (Inventário)" }
                    div { style: "display: flex; gap: 8px; align-items: center;",
                        select {
                            class: "form-select",
                            style: "flex: 1;",
                            value: "{selected_material_id}",
                            onchange: move |e| selected_material_id.set(e.value()),
                            option { value: "", "Selecione um material do estoque..." }
                            for item in available_materials {
                                option { value: "{item.name} ({item.unit_type})", "{item.name} — Estoque: {item.current_stock} {item.unit_type}" }
                            }
                        }
                        input {
                            class: "form-input",
                            style: "width: 70px; text-align: center;",
                            r#type: "number",
                            min: "1",
                            value: "{material_qty}",
                            oninput: move |e| material_qty.set(e.value()),
                        }
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            style: "font-weight: 700; font-size: 12px;",
                            onclick: move |_| {
                                let sel = selected_material_id.read().trim().to_string();
                                if !sel.is_empty() {
                                    let qty = material_qty.read().trim().to_string();
                                    let item_entry = format!("{}x {}", qty, sel);
                                    let mut list = materials_list.read().clone();
                                    if !list.contains(&item_entry) {
                                        list.push(item_entry);
                                        materials_list.set(list);
                                    }
                                }
                            },
                            "+ Adicionar"
                        }
                    }

                    if !materials_list.read().is_empty() {
                        div { style: "display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px;",
                            for mat in materials_list.read().iter() {
                                {
                                    let m_clone = mat.clone();
                                    rsx! {
                                        span { key: "{mat}", class: "badge badge-blue", style: "display: inline-flex; align-items: center; gap: 6px; padding: 4px 8px;",
                                            span { "{mat}" }
                                            button {
                                                r#type: "button",
                                                style: "background: transparent; border: none; color: #94a3b8; cursor: pointer; font-size: 13px; padding: 0;",
                                                onclick: move |_| {
                                                    let mut list = materials_list.read().clone();
                                                    list.retain(|x| x != &m_clone);
                                                    materials_list.set(list);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Equipamentos selecionados do Estoque
                div { class: "form-field",
                    label { class: "form-label", "🛠️ Equipamentos Necessários (Inventário)" }
                    div { style: "display: flex; gap: 8px; align-items: center;",
                        select {
                            class: "form-select",
                            style: "flex: 1;",
                            value: "{selected_equipment_id}",
                            onchange: move |e| selected_equipment_id.set(e.value()),
                            option { value: "", "Selecione um equipamento do estoque..." }
                            for item in available_equipment {
                                option { value: "{item.name}", "{item.name} ({item.manufacturer.as_deref().unwrap_or(\"Geral\")})" }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            style: "font-weight: 700; font-size: 12px;",
                            onclick: move |_| {
                                let sel = selected_equipment_id.read().trim().to_string();
                                if !sel.is_empty() {
                                    let mut list = equipment_list.read().clone();
                                    if !list.contains(&sel) {
                                        list.push(sel);
                                        equipment_list.set(list);
                                    }
                                }
                            },
                            "+ Adicionar"
                        }
                    }

                    if !equipment_list.read().is_empty() {
                        div { style: "display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px;",
                            for eq in equipment_list.read().iter() {
                                {
                                    let eq_clone = eq.clone();
                                    rsx! {
                                        span { key: "{eq}", class: "badge badge-gray", style: "display: inline-flex; align-items: center; gap: 6px; padding: 4px 8px;",
                                            span { "{eq}" }
                                            button {
                                                r#type: "button",
                                                style: "background: transparent; border: none; color: #94a3b8; cursor: pointer; font-size: 13px; padding: 0;",
                                                onclick: move |_| {
                                                    let mut list = equipment_list.read().clone();
                                                    list.retain(|x| x != &eq_clone);
                                                    equipment_list.set(list);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "form-field",
                    label { class: "form-label", "Orientações Pós-Atendimento" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Evitar mastigar alimentos duros por 2 horas...",
                        value: "{post_care}",
                        oninput: move |e| post_care.set(e.value()),
                    }
                }

                div { class: "form-field",
                    label { class: "form-label", "Descrição Detalhada do Procedimento" }
                    textarea {
                        class: "form-textarea",
                        placeholder: "Descreva os passos clínicos e orientações para a equipe...",
                        rows: "2",
                        value: "{description}",
                        oninput: move |e| description.set(e.value()),
                    }
                }
            }
        }
    }
}
