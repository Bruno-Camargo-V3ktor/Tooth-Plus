//! # Modal de Procedimento Padrão Sincronizado com o Estoque (Frontend)
//!
//! Permite cadastrar e editar procedimentos com valores, dentes/regiões
//! e seleção direta de materiais (com quantidades e unidades) e equipamentos
//! integrados ao estoque da clínica.

use crate::api::{
    create_treatment_template, fetch_stock_data, update_treatment_template,
};
use crate::components::icons::{IconBox, IconClock, IconPlus, IconTool, IconTooth, IconTrash};
use dioxus::prelude::*;
use shared::stock::{InventoryItem, ItemType};
use shared::treatments::{
    CreateTreatmentTemplateRequest, TreatmentTemplate, UpdateTreatmentTemplateRequest,
};

/// Item de material com controle explícito de quantidade e unidade.
#[derive(Clone, PartialEq, Debug)]
pub struct SelectedMaterialItem {
    pub name: String,
    pub quantity: i32,
    pub unit: String,
}

fn parse_stored_material(raw: &str) -> SelectedMaterialItem {
    let raw = raw.trim();
    // Padrão: "2x Nome do Material (unidade)" ou "2x Nome" ou "Nome"
    if let Some(rest) = raw.strip_prefix(|c: char| c.is_ascii_digit()) {
        let digits_len = raw.len() - rest.len() + 1; // primeiro dígito + possíveis outros
        let parts: Vec<&str> = raw.splitn(2, "x ").collect();
        if parts.len() == 2 {
            if let Ok(qty) = parts[0].trim().parse::<i32>() {
                let name_and_unit = parts[1].trim();
                if name_and_unit.ends_with(')') && name_and_unit.contains('(') {
                    if let Some(open_idx) = name_and_unit.rfind('(') {
                        let name = name_and_unit[..open_idx].trim().to_string();
                        let unit = name_and_unit[open_idx + 1..name_and_unit.len() - 1].trim().to_string();
                        return SelectedMaterialItem { name, quantity: qty.max(1), unit };
                    }
                }
                return SelectedMaterialItem {
                    name: name_and_unit.to_string(),
                    quantity: qty.max(1),
                    unit: "unidade".to_string(),
                };
            }
        }
    }

    SelectedMaterialItem {
        name: raw.to_string(),
        quantity: 1,
        unit: "unidade".to_string(),
    }
}

#[component]
pub fn TreatmentTemplateModal(
    token: String,
    clinic_id: String,
    editing_template: Option<TreatmentTemplate>,
    is_open: Signal<bool>,
    reload_counter: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    if !is_open() {
        return rsx! {};
    }

    let is_editing = editing_template.is_some();
    let modal_title = if is_editing {
        "Editar Procedimento Padrão"
    } else {
        "Novo Procedimento Padrão"
    };

    let categories = vec![
        "Dentística",
        "Cirurgia",
        "Endodontia",
        "Ortodontia",
        "Periodontia",
        "Prótese",
        "Estética",
        "Implantodontia",
        "Odontopediatria",
        "Geral",
    ];

    let mut form_name = use_signal(|| {
        editing_template
            .as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_default()
    });
    let mut form_category = use_signal(|| {
        editing_template
            .as_ref()
            .and_then(|t| t.category.clone())
            .unwrap_or_else(|| "Dentística".to_string())
    });
    let mut form_description = use_signal(|| {
        editing_template
            .as_ref()
            .and_then(|t| t.description.clone())
            .unwrap_or_default()
    });
    let mut form_price = use_signal(|| {
        editing_template
            .as_ref()
            .map(|t| format!("{:.2}", (t.default_price_cents as f64) / 100.0).replace('.', ","))
            .unwrap_or_else(|| "0,00".into())
    });
    let mut form_duration = use_signal(|| {
        editing_template
            .as_ref()
            .and_then(|t| t.estimated_duration_minutes)
            .map(|d| d.to_string())
            .unwrap_or_else(|| "30".to_string())
    });

    let mut selected_regions = use_signal(|| {
        editing_template
            .as_ref()
            .map(|t| t.dental_regions.clone())
            .unwrap_or_default()
    });
    let mut selected_teeth = use_signal(|| {
        editing_template
            .as_ref()
            .map(|t| t.target_teeth.clone())
            .unwrap_or_default()
    });
    let mut custom_tooth_input = use_signal(String::new);

    // Lista de materiais com quantidades
    let mut selected_materials = use_signal(|| {
        editing_template
            .as_ref()
            .map(|t| t.required_materials.iter().map(|s| parse_stored_material(s)).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    // Inputs de adição de material
    let mut chosen_stock_mat_idx = use_signal(|| "".to_string());
    let mut chosen_mat_qty = use_signal(|| 1i32);
    let mut custom_mat_name = use_signal(String::new);
    let mut custom_mat_unit = use_signal(|| "unidade".to_string());

    // Equipamentos selecionados
    let mut selected_equipment = use_signal(|| {
        editing_template
            .as_ref()
            .map(|t| t.required_equipment.clone())
            .unwrap_or_default()
    });
    let mut custom_equipment_input = use_signal(String::new);

    let mut form_post_care = use_signal(|| {
        editing_template
            .as_ref()
            .and_then(|t| t.post_care_instructions.clone())
            .unwrap_or_default()
    });
    let mut form_notes = use_signal(|| {
        editing_template
            .as_ref()
            .and_then(|t| t.clinical_notes.clone())
            .unwrap_or_default()
    });
    let mut is_submitting = use_signal(|| false);

    // Carregar dados de estoque
    let tok_stock = token.clone();
    let cid_stock = clinic_id.clone();
    let stock_resource = use_resource(move || {
        let t = tok_stock.clone();
        let c = cid_stock.clone();
        async move {
            if t.is_empty() || c.is_empty() {
                return None;
            }
            fetch_stock_data(&t, &c, None, None).await.ok()
        }
    });

    let (stock_materials, stock_equipments) = match &*stock_resource.read() {
        Some(Some(data)) => {
            let mats: Vec<InventoryItem> = data
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::Material || i.item_type == ItemType::Chemical)
                .cloned()
                .collect();
            let eqs: Vec<InventoryItem> = data
                .items
                .iter()
                .filter(|i| i.item_type == ItemType::Equipment)
                .cloned()
                .collect();
            (mats, eqs)
        }
        _ => (vec![], vec![]),
    };

    let tok_submit = token.clone();
    let cid_submit = clinic_id.clone();
    let tmpl_opt = editing_template.clone();

    let mut handle_submit = move |_| {
        let name = form_name().trim().to_string();
        if name.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome do procedimento.".into()));
            return;
        }

        let price_clean = form_price()
            .replace("R$", "")
            .replace(".", "")
            .replace(",", ".")
            .trim()
            .to_string();
        let price_cents = match price_clean.parse::<f64>() {
            Ok(v) => (v * 100.0).round() as i64,
            Err(_) => 0,
        };

        let duration = form_duration().trim().parse::<i32>().ok();
        let desc = if form_description().trim().is_empty() {
            None
        } else {
            Some(form_description().trim().to_string())
        };
        let post_care = if form_post_care().trim().is_empty() {
            None
        } else {
            Some(form_post_care().trim().to_string())
        };
        let notes = if form_notes().trim().is_empty() {
            None
        } else {
            Some(form_notes().trim().to_string())
        };

        // Formata os materiais como "2x Nome do Insumo (unidade)"
        let formatted_materials: Vec<String> = selected_materials()
            .into_iter()
            .map(|m| format!("{}x {} ({})", m.quantity, m.name, m.unit))
            .collect();

        let t = tok_submit.clone();
        let cid = cid_submit.clone();
        let item_edit = tmpl_opt.clone();
        let mut sub_sig = is_submitting;
        let mut open_sig = is_open;
        let mut rel_sig = reload_counter;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        sub_sig.set(true);
        spawn(async move {
            if let Some(ref edit_item) = item_edit {
                let req = UpdateTreatmentTemplateRequest {
                    clinic_id: cid.clone(),
                    name,
                    category: Some(form_category()),
                    description: desc,
                    default_price_cents: price_cents,
                    estimated_duration_minutes: duration,
                    dental_regions: selected_regions(),
                    target_teeth: selected_teeth(),
                    required_materials: formatted_materials,
                    required_equipment: selected_equipment(),
                    post_care_instructions: post_care,
                    clinical_notes: notes,
                    is_active: edit_item.is_active,
                };
                match update_treatment_template(&t, &cid, &edit_item.id, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        rel_sig.set(rel_sig() + 1);
                        toast.set(Some("Procedimento padrão atualizado com sucesso!".into()));
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao atualizar procedimento: {}", e)));
                    }
                }
            } else {
                let req = CreateTreatmentTemplateRequest {
                    clinic_id: cid.clone(),
                    name,
                    category: Some(form_category()),
                    description: desc,
                    default_price_cents: price_cents,
                    estimated_duration_minutes: duration,
                    dental_regions: selected_regions(),
                    target_teeth: selected_teeth(),
                    required_materials: formatted_materials,
                    required_equipment: selected_equipment(),
                    post_care_instructions: post_care,
                    clinical_notes: notes,
                };
                match create_treatment_template(&t, &cid, req).await {
                    Ok(_) => {
                        open_sig.set(false);
                        rel_sig.set(rel_sig() + 1);
                        toast.set(Some("Procedimento cadastrado no catálogo!".into()));
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao cadastrar procedimento: {}", e)));
                    }
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| is_open.set(false),
            div {
                class: "modal-card treatment-template-modal",
                onclick: move |e| e.stop_propagation(),

                // 1. Header do Modal
                div { class: "modal-header",
                    div { class: "modal-header-left",
                        div { class: "stock-header-icon-box",
                            IconTooth { size: 20, color: "#0284c7".to_string() }
                        }
                        div { class: "header-text-col",
                            h2 { class: "modal-title", "{modal_title}" }
                            p { class: "modal-subtitle", "Defina valores, dentes e vincule insumos com quantidades do estoque." }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "modal-close-btn",
                        onclick: move |_| is_open.set(false),
                        "✕"
                    }
                }

                // 2. Conteúdo com Rolagem Fluida
                div { class: "modal-body treatment-modal-scroll",
                    // 1. Especialidade / Categoria
                    div { class: "input-group-wrapper full-width",
                        label { "Especialidade Odontológica *" }
                        div { class: "treatment-category-selector-grid",
                            for cat in categories.iter() {
                                {
                                    let c = cat.to_string();
                                    let is_sel = form_category() == c;
                                    rsx! {
                                        button {
                                            key: "{cat}",
                                            r#type: "button",
                                            class: if is_sel { "treatment-category-card active" } else { "treatment-category-card" },
                                            onclick: move |_| form_category.set(c.clone()),
                                            span { "{cat}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 2. Nome do Procedimento
                    div { class: "input-group-wrapper full-width",
                        label { "Nome do Procedimento *" }
                        input {
                            class: "modern-input-field",
                            placeholder: "Ex: Extração Cirúrgica de Siso, Clareamento Dental a Laser, Restauração em Resina...",
                            value: "{form_name}",
                            oninput: move |e| form_name.set(e.value()),
                        }
                    }

                    // 3. Preço Base Sugerido e Duração
                    div { class: "form-row-grid-2",
                        div { class: "input-group-wrapper",
                            label { "Preço Base Sugerido (R$) *" }
                            input {
                                class: "modern-input-field font-mono",
                                placeholder: "0,00",
                                value: "{form_price}",
                                oninput: move |e| form_price.set(e.value()),
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Duração Estimada (minutos)" }
                            div { class: "input-with-icon-wrap",
                                IconClock { size: 16, color: "#64748b".to_string() }
                                input {
                                    r#type: "number",
                                    class: "modern-input-field font-mono",
                                    placeholder: "30",
                                    value: "{form_duration}",
                                    oninput: move |e| form_duration.set(e.value()),
                                }
                            }
                        }
                    }

                    // 4. Descrição do Procedimento
                    div { class: "input-group-wrapper full-width",
                        label { "Descrição / Detalhes do Procedimento" }
                        textarea {
                            class: "modern-input-field",
                            rows: "2",
                            placeholder: "Breve explicação sobre a indicação, técnica e objetivo clínico...",
                            value: "{form_description}",
                            oninput: move |e| form_description.set(e.value()),
                        }
                    }

                    // 5. SINCRONIZAÇÃO COM O ESTOQUE: Materiais & Insumos COM QUANTIDADE
                    div { class: "input-group-wrapper full-width stock-sync-section",
                        div { class: "sync-section-header",
                            div { class: "sync-title-wrap",
                                IconBox { size: 18, color: "#0284c7".to_string() }
                                strong { "Materiais & Insumos Necessários (Estoque da Clínica)" }
                            }
                            span { class: "sync-badge-count", "{selected_materials().len()} insumos definidos" }
                        }

                        // Seletor de Estoque com Quantidade
                        div { class: "stock-picker-grid-with-qty",
                            div { class: "picker-col-select",
                                select {
                                    class: "modern-input-field modern-select stock-picker-select",
                                    value: "{chosen_stock_mat_idx}",
                                    onchange: move |e: FormEvent| chosen_stock_mat_idx.set(e.value()),
                                    option { value: "", "🔍 Selecione um item do estoque..." }
                                    for (idx, mat) in stock_materials.iter().enumerate() {
                                        option {
                                            value: "{idx}",
                                            "{mat.name} ({mat.current_stock} {mat.unit_type} em estoque) - {mat.manufacturer.as_deref().unwrap_or(\"\")}"
                                        }
                                    }
                                }
                            }

                            div { class: "picker-col-qty",
                                span { class: "picker-qty-label", "Qtd:" }
                                input {
                                    r#type: "number",
                                    class: "modern-input-field font-mono qty-number-input",
                                    min: "1",
                                    value: "{chosen_mat_qty}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            chosen_mat_qty.set(v.max(1));
                                        }
                                    },
                                }
                            }

                            button {
                                r#type: "button",
                                class: "btn-primary btn-add-mat",
                                onclick: move |_| {
                                    let idx_str = chosen_stock_mat_idx();
                                    if let Ok(idx) = idx_str.parse::<usize>() {
                                        if let Some(item) = stock_materials.get(idx) {
                                            let mut list = selected_materials();
                                            let qty = chosen_mat_qty();
                                            // Se já existe, soma quantidade ou atualiza
                                            if let Some(existing) = list.iter_mut().find(|m| m.name == item.name) {
                                                existing.quantity += qty;
                                            } else {
                                                list.push(SelectedMaterialItem {
                                                    name: item.name.clone(),
                                                    quantity: qty,
                                                    unit: item.unit_type.clone(),
                                                });
                                            }
                                            selected_materials.set(list);
                                            chosen_stock_mat_idx.set(String::new());
                                            chosen_mat_qty.set(1);
                                        }
                                    }
                                },
                                IconPlus { size: 15, color: "#ffffff".to_string() }
                                span { "Adicionar" }
                            }
                        }

                        // Linha para adicionar material manual personalizado
                        div { class: "custom-material-manual-row",
                            input {
                                class: "modern-input-field flex-2",
                                placeholder: "Ou digite outro material/fármaco manual...",
                                value: "{custom_mat_name}",
                                oninput: move |e| custom_mat_name.set(e.value()),
                            }
                            select {
                                class: "modern-input-field modern-select flex-1",
                                value: "{custom_mat_unit}",
                                onchange: move |e: FormEvent| custom_mat_unit.set(e.value()),
                                option { value: "unidade", "unidade" }
                                option { value: "par", "par" }
                                option { value: "caixa", "caixa" }
                                option { value: "frasco", "frasco" }
                                option { value: "tubete", "tubete" }
                                option { value: "grama", "grama" }
                                option { value: "ml", "ml" }
                            }
                            button {
                                r#type: "button",
                                class: "btn-secondary btn-sm",
                                onclick: move |_| {
                                    let name = custom_mat_name().trim().to_string();
                                    if !name.is_empty() {
                                        let mut list = selected_materials();
                                        list.push(SelectedMaterialItem {
                                            name,
                                            quantity: chosen_mat_qty(),
                                            unit: custom_mat_unit(),
                                        });
                                        selected_materials.set(list);
                                        custom_mat_name.set(String::new());
                                    }
                                },
                                IconPlus { size: 14, color: "currentColor".to_string() }
                                span { "Incluir Manual" }
                            }
                        }

                        // Lista dos Materiais Selecionados com Quantidades
                        if !selected_materials().is_empty() {
                            div { class: "selected-materials-list-container",
                                for (idx, mat) in selected_materials().iter().enumerate() {
                                    {
                                        let m_clone = mat.clone();
                                        rsx! {
                                            div { key: "{idx}_{mat.name}", class: "selected-material-card-row",
                                                div { class: "mat-row-left",
                                                    span { class: "mat-qty-badge", "{m_clone.quantity}x" }
                                                    IconBox { size: 15, color: "#0284c7".to_string() }
                                                    span { class: "mat-name-text", "{m_clone.name}" }
                                                    span { class: "mat-unit-pill", "({m_clone.unit})" }
                                                }

                                                div { class: "mat-row-actions",
                                                    button {
                                                        r#type: "button",
                                                        class: "mat-qty-step-btn",
                                                        title: "Diminuir quantidade",
                                                        onclick: move |_| {
                                                            let mut list = selected_materials();
                                                            if let Some(item) = list.get_mut(idx) {
                                                                if item.quantity > 1 {
                                                                    item.quantity -= 1;
                                                                } else {
                                                                    list.remove(idx);
                                                                }
                                                                selected_materials.set(list);
                                                            }
                                                        },
                                                        "-"
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "mat-qty-step-btn",
                                                        title: "Aumentar quantidade",
                                                        onclick: move |_| {
                                                            let mut list = selected_materials();
                                                            if let Some(item) = list.get_mut(idx) {
                                                                item.quantity += 1;
                                                                selected_materials.set(list);
                                                            }
                                                        },
                                                        "+"
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "mat-remove-btn",
                                                        title: "Remover este insumo",
                                                        onclick: move |_| {
                                                            let mut list = selected_materials();
                                                            list.remove(idx);
                                                            selected_materials.set(list);
                                                        },
                                                        IconTrash { size: 13, color: "#dc2626".to_string() }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 6. SINCRONIZAÇÃO COM O ESTOQUE: Equipamentos & Patrimônio
                    div { class: "input-group-wrapper full-width stock-sync-section",
                        div { class: "sync-section-header",
                            div { class: "sync-title-wrap",
                                IconTool { size: 18, color: "#7e22ce".to_string() }
                                strong { "Equipamentos & Patrimônio Necessários (Estoque da Clínica)" }
                            }
                            span { class: "sync-badge-count", "{selected_equipment().len()} equipamentos" }
                        }

                        div { class: "stock-picker-row",
                            select {
                                class: "modern-input-field modern-select stock-picker-select",
                                onchange: move |e: FormEvent| {
                                    let val = e.value().trim().to_string();
                                    if !val.is_empty() {
                                        let mut list = selected_equipment();
                                        if !list.contains(&val) {
                                            list.push(val);
                                            selected_equipment.set(list);
                                        }
                                    }
                                },
                                option { value: "", "🛠️ Selecione um equipamento do patrimônio..." }
                                for eq in stock_equipments.iter() {
                                    option {
                                        value: "{eq.name}",
                                        "🛠️ {eq.name} (Série: {eq.serial_number.as_deref().unwrap_or(\"-\")})"
                                    }
                                }
                            }

                            div { class: "stock-custom-add-wrap",
                                input {
                                    class: "modern-input-field",
                                    placeholder: "Ou digite outro equipamento...",
                                    value: "{custom_equipment_input}",
                                    oninput: move |e| custom_equipment_input.set(e.value()),
                                }
                                button {
                                    r#type: "button",
                                    class: "btn-secondary btn-icon-only",
                                    title: "Adicionar equipamento manual",
                                    onclick: move |_| {
                                        let val = custom_equipment_input().trim().to_string();
                                        if !val.is_empty() {
                                            let mut list = selected_equipment();
                                            if !list.contains(&val) {
                                                list.push(val);
                                                selected_equipment.set(list);
                                            }
                                            custom_equipment_input.set(String::new());
                                        }
                                    },
                                    IconPlus { size: 14, color: "currentColor".to_string() }
                                }
                            }
                        }

                        if !selected_equipment().is_empty() {
                            div { class: "stock-chips-grid",
                                for (idx, eq) in selected_equipment().iter().enumerate() {
                                    {
                                        let eq_text = eq.clone();
                                        rsx! {
                                            div { key: "{idx}_{eq}", class: "stock-selected-chip chip-equipment",
                                                IconTool { size: 13, color: "#7e22ce".to_string() }
                                                span { class: "chip-text", "{eq_text}" }
                                                button {
                                                    r#type: "button",
                                                    class: "chip-remove-btn",
                                                    onclick: move |_| {
                                                        let mut list = selected_equipment();
                                                        list.retain(|item| item != &eq_text);
                                                        selected_equipment.set(list);
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

                    // 7. Dentes Alvo e Regiões Anatômicas
                    div { class: "input-group-wrapper full-width",
                        label { "Dentes Alvo Sugeridos (Atalhos Rápidos)" }
                        div { class: "tooth-preset-row",
                            button {
                                r#type: "button",
                                class: "tooth-preset-btn",
                                onclick: move |_| selected_teeth.set(vec!["Arcada Completa".into()]),
                                "Arcada Completa"
                            }
                            button {
                                r#type: "button",
                                class: "tooth-preset-btn",
                                onclick: move |_| selected_teeth.set(vec!["18".into(), "28".into(), "38".into(), "48".into()]),
                                "Sisos (18, 28, 38, 48)"
                            }
                            button {
                                r#type: "button",
                                class: "tooth-preset-btn",
                                onclick: move |_| selected_teeth.set(vec!["16".into(), "17".into(), "26".into(), "27".into(), "36".into(), "37".into(), "46".into(), "47".into()]),
                                "Molares"
                            }
                            button {
                                r#type: "button",
                                class: "tooth-preset-btn",
                                onclick: move |_| selected_teeth.set(vec!["11".into(), "12".into(), "21".into(), "22".into(), "31".into(), "32".into(), "41".into(), "42".into()]),
                                "Incisivos"
                            }
                        }

                        div { class: "stock-picker-row",
                            input {
                                class: "modern-input-field",
                                placeholder: "Adicionar número de dente específico (ex: 36)...",
                                value: "{custom_tooth_input}",
                                oninput: move |e| custom_tooth_input.set(e.value()),
                            }
                            button {
                                r#type: "button",
                                class: "btn-secondary btn-icon-only",
                                onclick: move |_| {
                                    let val = custom_tooth_input().trim().to_string();
                                    if !val.is_empty() {
                                        let mut list = selected_teeth();
                                        if !list.contains(&val) {
                                            list.push(val);
                                            selected_teeth.set(list);
                                        }
                                        custom_tooth_input.set(String::new());
                                    }
                                },
                                IconPlus { size: 14, color: "currentColor".to_string() }
                            }
                        }

                        if !selected_teeth().is_empty() {
                            div { class: "stock-chips-grid",
                                for (idx, tooth) in selected_teeth().iter().enumerate() {
                                    {
                                        let t_text = tooth.clone();
                                        rsx! {
                                            div { key: "{idx}_{tooth}", class: "stock-selected-chip chip-tooth",
                                                IconTooth { size: 13, color: "#0f766e".to_string() }
                                                span { class: "chip-text", "Dente {t_text}" }
                                                button {
                                                    r#type: "button",
                                                    class: "chip-remove-btn",
                                                    onclick: move |_| {
                                                        let mut list = selected_teeth();
                                                        list.retain(|item| item != &t_text);
                                                        selected_teeth.set(list);
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

                    // 8. Orientações Pós-Operatórias e Observações Clínicas
                    div { class: "form-row-grid-2",
                        div { class: "input-group-wrapper",
                            label { "Orientações Pós-Operatórias Padrão" }
                            textarea {
                                class: "modern-input-field",
                                rows: "2",
                                placeholder: "Ex: Repouso 24h, compressas de gelo, evitar sucção...",
                                value: "{form_post_care}",
                                oninput: move |e| form_post_care.set(e.value()),
                            }
                        }

                        div { class: "input-group-wrapper",
                            label { "Observações Clínicas / Protocolo" }
                            textarea {
                                class: "modern-input-field",
                                rows: "2",
                                placeholder: "Notas técnicas para os dentistas da clínica...",
                                value: "{form_notes}",
                                oninput: move |e| form_notes.set(e.value()),
                            }
                        }
                    }
                }

                // 3. Rodapé Fixo e Alinhado à Direita
                div { class: "modal-footer",
                    button {
                        r#type: "button",
                        class: "btn-secondary",
                        onclick: move |_| is_open.set(false),
                        "Cancelar"
                    }
                    button {
                        r#type: "button",
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: handle_submit,
                        if is_submitting() {
                            "Salvando..."
                        } else if is_editing {
                            "Salvar Alterações"
                        } else {
                            "Cadastrar Procedimento"
                        }
                    }
                }
            }
        }
    }
}
