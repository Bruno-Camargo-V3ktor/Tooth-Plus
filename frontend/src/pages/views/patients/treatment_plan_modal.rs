//! # Modal de Elaboração de Orçamento / Plano de Tratamento
//!
//! Permite montar planos de tratamento completos vinculados ao paciente,
//! selecionando procedimentos padrão do catálogo da clínica ou criando itens do zero,
//! com totalizador automático e opção de aprovação com emissão de cobrança financeira.

use crate::api::{create_treatment_plan, fetch_treatment_templates, update_treatment_plan};
use crate::components::icons::{IconCheck, IconPlus, IconTooth, IconTrash};
use dioxus::prelude::*;
use shared::treatments::{
    CreateTreatmentPlanItemRequest, CreateTreatmentPlanRequest, PatientTreatmentPlan,
    TreatmentTemplate, UpdateTreatmentPlanRequest,
};

fn format_currency(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let reals = abs_cents / 100;
    let centavos = abs_cents % 100;

    let reals_str = reals.to_string();
    let mut formatted_reals = String::new();
    let len = reals_str.len();
    for (i, ch) in reals_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_reals.push('.');
        }
        formatted_reals.push(ch);
    }

    if is_negative {
        format!("- R$ {},{:02}", formatted_reals, centavos)
    } else {
        format!("R$ {},{:02}", formatted_reals, centavos)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlanFormItem {
    pub template_id: Option<String>,
    pub procedure_name: String,
    pub category: String,
    pub tooth_number: String,
    pub dental_region: String,
    pub surfaces: Vec<String>,
    pub price_reals_str: String,
    pub clinical_notes: String,
}

#[component]
pub fn TreatmentPlanModal(
    patient_id: String,
    clinic_id: String,
    token: String,
    editing_plan: Option<PatientTreatmentPlan>,
    on_close: EventHandler<()>,
    on_saved: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut plan_title = use_signal(|| {
        if let Some(ref p) = editing_plan {
            p.title.clone()
        } else {
            "Plano de Tratamento Odontológico".to_string()
        }
    });

    let mut plan_notes = use_signal(|| {
        editing_plan
            .as_ref()
            .and_then(|p| p.notes.clone())
            .unwrap_or_default()
    });

    let mut start_date = use_signal(|| {
        editing_plan
            .as_ref()
            .and_then(|p| p.planned_start_date.clone())
            .unwrap_or_default()
    });

    let mut end_date = use_signal(|| {
        editing_plan
            .as_ref()
            .and_then(|p| p.planned_end_date.clone())
            .unwrap_or_default()
    });

    let mut items = use_signal(|| {
        if let Some(ref p) = editing_plan {
            p.items
                .iter()
                .map(|item| {
                    let reals = item.price_cents / 100;
                    let centavos = item.price_cents % 100;
                    PlanFormItem {
                        template_id: item.template_id.clone(),
                        procedure_name: item.procedure_name.clone(),
                        category: item.category.clone().unwrap_or_else(|| "Geral".into()),
                        tooth_number: item.tooth_number.clone().unwrap_or_default(),
                        dental_region: item.dental_region.clone().unwrap_or_default(),
                        surfaces: item.surfaces.clone(),
                        price_reals_str: format!("{}.{:02}", reals, centavos),
                        clinical_notes: item.clinical_notes.clone().unwrap_or_default(),
                    }
                })
                .collect::<Vec<PlanFormItem>>()
        } else {
            vec![]
        }
    });

    let mut is_submitting = use_signal(|| false);
    let mut selected_template_id = use_signal(|| "".to_string());

    // Fetch treatment templates for dropdown
    let tok_tpl = token.clone();
    let cid_tpl = clinic_id.clone();
    let templates_res = use_resource(move || {
        let t = tok_tpl.clone();
        let cid = cid_tpl.clone();
        async move {
            if t.is_empty() || cid.is_empty() {
                return Ok(vec![]);
            }
            fetch_treatment_templates(&t, &cid).await
        }
    });

    let templates_list: Vec<TreatmentTemplate> = match &*templates_res.read() {
        Some(Ok(tpls)) => tpls.clone(),
        _ => vec![],
    };

    // Calculate running total
    let current_items = items();
    let total_cents: i64 = current_items
        .iter()
        .map(|item| {
            let clean = item
                .price_reals_str
                .trim()
                .replace(',', ".")
                .replace("R$", "")
                .replace(' ', "");
            if let Ok(val) = clean.parse::<f64>() {
                (val * 100.0).round() as i64
            } else {
                0
            }
        })
        .sum();

    let add_custom_item = move |_| {
        let mut list = items();
        list.push(PlanFormItem {
            template_id: None,
            procedure_name: String::new(),
            category: "Dentística".to_string(),
            tooth_number: String::new(),
            dental_region: String::new(),
            surfaces: vec![],
            price_reals_str: "0.00".to_string(),
            clinical_notes: String::new(),
        });
        items.set(list);
    };

    let mut add_template_item = move |tmpl: TreatmentTemplate| {
        let reals = tmpl.default_price_cents / 100;
        let cents = tmpl.default_price_cents % 100;
        let mut list = items();
        list.push(PlanFormItem {
            template_id: Some(tmpl.id.clone()),
            procedure_name: tmpl.name.clone(),
            category: tmpl.category.clone().unwrap_or_else(|| "Geral".into()),
            tooth_number: tmpl.target_teeth.join(", "),
            dental_region: tmpl.dental_regions.join(", "),
            surfaces: vec![],
            price_reals_str: format!("{}.{:02}", reals, cents),
            clinical_notes: tmpl.clinical_notes.clone().unwrap_or_default(),
        });
        items.set(list);
    };

    let tok_draft = token.clone();
    let pat_draft = patient_id.clone();
    let cid_draft = clinic_id.clone();
    let edit_plan_draft = editing_plan.clone();

    let handle_save_draft = move |_| {
        let title = plan_title().trim().to_string();
        if title.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe um título para o orçamento.".into()));
            return;
        }

        let current_list = items();
        if current_list.is_empty() {
            let mut err = error_toast;
            err.set(Some("Adicione ao menos um procedimento ao orçamento.".into()));
            return;
        }

        for (idx, itm) in current_list.iter().enumerate() {
            if itm.procedure_name.trim().is_empty() {
                let mut err = error_toast;
                err.set(Some(format!(
                    "O item #{} precisa de um nome de procedimento.",
                    idx + 1
                )));
                return;
            }
        }

        let plan_items_req: Vec<CreateTreatmentPlanItemRequest> = current_list
            .iter()
            .enumerate()
            .map(|(idx, itm)| {
                let clean = itm
                    .price_reals_str
                    .trim()
                    .replace(',', ".")
                    .replace("R$", "")
                    .replace(' ', "");
                let price_cents = if let Ok(val) = clean.parse::<f64>() {
                    (val * 100.0).round() as i64
                } else {
                    0
                };

                CreateTreatmentPlanItemRequest {
                    template_id: itm.template_id.clone(),
                    procedure_name: itm.procedure_name.trim().to_string(),
                    category: Some(itm.category.clone()),
                    tooth_number: if itm.tooth_number.trim().is_empty() {
                        None
                    } else {
                        Some(itm.tooth_number.trim().to_string())
                    },
                    dental_region: if itm.dental_region.trim().is_empty() {
                        None
                    } else {
                        Some(itm.dental_region.trim().to_string())
                    },
                    surfaces: itm.surfaces.clone(),
                    price_cents,
                    clinical_notes: if itm.clinical_notes.trim().is_empty() {
                        None
                    } else {
                        Some(itm.clinical_notes.trim().to_string())
                    },
                    sort_order: Some(idx as i32),
                }
            })
            .collect();

        let s_date = if start_date().trim().is_empty() { None } else { Some(start_date().trim().to_string()) };
        let e_date = if end_date().trim().is_empty() { None } else { Some(end_date().trim().to_string()) };
        let notes_opt = if plan_notes().trim().is_empty() { None } else { Some(plan_notes().trim().to_string()) };

        let t = tok_draft.clone();
        let pid = pat_draft.clone();
        let cid = cid_draft.clone();
        let existing = edit_plan_draft.clone();

        let mut sub_sig = is_submitting;
        sub_sig.set(true);
        let on_s = on_saved.clone();
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        spawn(async move {
            let res = if let Some(ref pl) = existing {
                let req = UpdateTreatmentPlanRequest {
                    clinic_id: cid.clone(),
                    dentist_user_id: None,
                    title,
                    items: plan_items_req,
                    notes: notes_opt,
                    planned_start_date: s_date,
                    planned_end_date: e_date,
                };
                update_treatment_plan(&t, &pid, &pl.id, req).await.map(|_| ())
            } else {
                let req = CreateTreatmentPlanRequest {
                    clinic_id: cid.clone(),
                    dentist_user_id: None,
                    title,
                    items: plan_items_req,
                    notes: notes_opt,
                    planned_start_date: s_date,
                    planned_end_date: e_date,
                    approve_immediately: false,
                };
                create_treatment_plan(&t, &pid, req).await.map(|_| ())
            };

            sub_sig.set(false);
            match res {
                Ok(_) => {
                    toast.set(Some("Orçamento salvo como rascunho com sucesso!".into()));
                    on_s.call(());
                }
                Err(e) => {
                    err_sig.set(Some(e));
                }
            }
        });
    };

    let tok_approve = token.clone();
    let pat_approve = patient_id.clone();
    let cid_approve = clinic_id.clone();
    let edit_plan_approve = editing_plan.clone();

    let handle_save_and_approve = move |_| {
        let title = plan_title().trim().to_string();
        if title.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe um título para o orçamento.".into()));
            return;
        }

        let current_list = items();
        if current_list.is_empty() {
            let mut err = error_toast;
            err.set(Some("Adicione ao menos um procedimento ao orçamento.".into()));
            return;
        }

        for (idx, itm) in current_list.iter().enumerate() {
            if itm.procedure_name.trim().is_empty() {
                let mut err = error_toast;
                err.set(Some(format!(
                    "O item #{} precisa de um nome de procedimento.",
                    idx + 1
                )));
                return;
            }
        }

        let plan_items_req: Vec<CreateTreatmentPlanItemRequest> = current_list
            .iter()
            .enumerate()
            .map(|(idx, itm)| {
                let clean = itm
                    .price_reals_str
                    .trim()
                    .replace(',', ".")
                    .replace("R$", "")
                    .replace(' ', "");
                let price_cents = if let Ok(val) = clean.parse::<f64>() {
                    (val * 100.0).round() as i64
                } else {
                    0
                };

                CreateTreatmentPlanItemRequest {
                    template_id: itm.template_id.clone(),
                    procedure_name: itm.procedure_name.trim().to_string(),
                    category: Some(itm.category.clone()),
                    tooth_number: if itm.tooth_number.trim().is_empty() {
                        None
                    } else {
                        Some(itm.tooth_number.trim().to_string())
                    },
                    dental_region: if itm.dental_region.trim().is_empty() {
                        None
                    } else {
                        Some(itm.dental_region.trim().to_string())
                    },
                    surfaces: itm.surfaces.clone(),
                    price_cents,
                    clinical_notes: if itm.clinical_notes.trim().is_empty() {
                        None
                    } else {
                        Some(itm.clinical_notes.trim().to_string())
                    },
                    sort_order: Some(idx as i32),
                }
            })
            .collect();

        let s_date = if start_date().trim().is_empty() { None } else { Some(start_date().trim().to_string()) };
        let e_date = if end_date().trim().is_empty() { None } else { Some(end_date().trim().to_string()) };
        let notes_opt = if plan_notes().trim().is_empty() { None } else { Some(plan_notes().trim().to_string()) };

        let t = tok_approve.clone();
        let pid = pat_approve.clone();
        let cid = cid_approve.clone();
        let existing = edit_plan_approve.clone();

        let mut sub_sig = is_submitting;
        sub_sig.set(true);
        let on_s = on_saved.clone();
        let mut toast = toast_msg;
        let mut err_sig = error_toast;

        spawn(async move {
            let res = if let Some(ref pl) = existing {
                let req = UpdateTreatmentPlanRequest {
                    clinic_id: cid.clone(),
                    dentist_user_id: None,
                    title,
                    items: plan_items_req,
                    notes: notes_opt,
                    planned_start_date: s_date,
                    planned_end_date: e_date,
                };
                update_treatment_plan(&t, &pid, &pl.id, req).await.map(|_| ())
            } else {
                let req = CreateTreatmentPlanRequest {
                    clinic_id: cid.clone(),
                    dentist_user_id: None,
                    title,
                    items: plan_items_req,
                    notes: notes_opt,
                    planned_start_date: s_date,
                    planned_end_date: e_date,
                    approve_immediately: true,
                };
                create_treatment_plan(&t, &pid, req).await.map(|_| ())
            };

            sub_sig.set(false);
            match res {
                Ok(_) => {
                    toast.set(Some("Orçamento aprovado e cobrança financeira gerada com sucesso!".into()));
                    on_s.call(());
                }
                Err(e) => {
                    err_sig.set(Some(e));
                }
            }
        });
    };

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal-card treatment-plan-modal-card",
                onclick: move |e| e.stop_propagation(),

                // 1. Header do Modal
                div { class: "modal-header",
                    div { class: "modal-header-left",
                        div { class: "stock-header-icon-box",
                            IconTooth { size: 20, color: "#0284c7".to_string() }
                        }
                        div { class: "header-text-col",
                            h2 { class: "modal-title",
                                if editing_plan.is_some() {
                                    "Editar Orçamento / Plano de Tratamento"
                                } else {
                                    "Novo Orçamento / Plano de Tratamento"
                                }
                            }
                            p { class: "modal-subtitle",
                                "Monte o plano com procedimentos tabelados ou personalizados."
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "modal-close-btn",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // 2. Corpo do Modal
                div { class: "modal-body treatment-modal-scroll",
                    // Informações Gerais
                    div { class: "form-row-grid-2",
                        div { class: "input-group-wrapper",
                            label { "Título do Orçamento *" }
                            input {
                                r#type: "text",
                                class: "modern-input-field",
                                placeholder: "Ex: Reabilitação Oral, Restauração e Canal...",
                                value: "{plan_title}",
                                oninput: move |e| plan_title.set(e.value()),
                            }
                        }
                        div { class: "input-group-wrapper",
                            label { "Previsão de Início" }
                            input {
                                r#type: "date",
                                class: "modern-input-field font-mono",
                                value: "{start_date}",
                                oninput: move |e| start_date.set(e.value()),
                            }
                        }
                    }

                    // Seção Construtor de Procedimentos
                    div { class: "input-group-wrapper full-width stock-sync-section",
                        div { class: "sync-section-header",
                            div { class: "sync-title-wrap",
                                IconTooth { size: 18, color: "#0284c7".to_string() }
                                strong { "Procedimentos do Orçamento" }
                            }
                            span { class: "sync-badge-count", "{current_items.len()} procedimentos" }
                        }

                        // Barra seletora de itens
                        div { class: "stock-picker-row",
                            select {
                                class: "modern-input-field modern-select stock-picker-select",
                                value: "{selected_template_id}",
                                onchange: move |e| {
                                    let val = e.value();
                                    if let Some(tmpl) = templates_list.iter().find(|t| t.id == val) {
                                        add_template_item(tmpl.clone());
                                    }
                                    selected_template_id.set(String::new());
                                },
                                option { value: "", "➕ Selecionar Procedimento Padrão do Catálogo..." }
                                for tmpl in templates_list.iter() {
                                    option { value: "{tmpl.id}",
                                        "{tmpl.name} ({format_currency(tmpl.default_price_cents)}) - {tmpl.category.as_deref().unwrap_or(\"\")}"
                                    }
                                }
                            }

                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "white-space: nowrap;",
                                onclick: add_custom_item,
                                IconPlus { size: 14, color: "currentColor".to_string() }
                                span { "Item Personalizado" }
                            }
                        }

                        // Lista dos Itens do Orçamento
                        if current_items.is_empty() {
                            div { class: "plan-empty-items-box",
                                IconTooth { size: 36, color: "#94a3b8".to_string() }
                                p { "Nenhum procedimento adicionado ao orçamento ainda." }
                                span { "Selecione um tratamento padrão acima ou clique em 'Item Personalizado'." }
                            }
                        } else {
                            div { class: "plan-items-table-wrapper",
                                for (idx, itm) in current_items.iter().enumerate() {
                                    {
                                        let item_idx = idx;
                                        let mut item_name = itm.procedure_name.clone();
                                        let mut item_tooth = itm.tooth_number.clone();
                                        let mut item_region = itm.dental_region.clone();
                                        let mut item_price = itm.price_reals_str.clone();
                                        let mut item_notes = itm.clinical_notes.clone();

                                        rsx! {
                                            div { key: "{idx}", class: "plan-item-row-card",
                                                div { class: "item-row-number", "#{idx + 1}" }

                                                div { class: "item-row-fields-grid",
                                                    div { class: "input-group-wrapper item-proc-name",
                                                        label { "Procedimento *" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field",
                                                            placeholder: "Nome do procedimento",
                                                            value: "{item_name}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if let Some(it) = list.get_mut(item_idx) {
                                                                    it.procedure_name = e.value();
                                                                }
                                                                items.set(list);
                                                            },
                                                        }
                                                    }

                                                    div { class: "input-group-wrapper item-tooth",
                                                        label { "Dente / Região" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field",
                                                            placeholder: "Ex: 21, 38 ou Arcada",
                                                            value: "{item_tooth}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if let Some(it) = list.get_mut(item_idx) {
                                                                    it.tooth_number = e.value();
                                                                }
                                                                items.set(list);
                                                            },
                                                        }
                                                    }

                                                    div { class: "input-group-wrapper item-price",
                                                        label { "Valor (R$)" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field font-mono",
                                                            placeholder: "0.00",
                                                            value: "{item_price}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if let Some(it) = list.get_mut(item_idx) {
                                                                    it.price_reals_str = e.value();
                                                                }
                                                                items.set(list);
                                                            },
                                                        }
                                                    }

                                                    div { class: "input-group-wrapper item-notes full-width-row",
                                                        label { "Notas Clínicas / Técnica" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field",
                                                            placeholder: "Observações específicas deste procedimento...",
                                                            value: "{item_notes}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if let Some(it) = list.get_mut(item_idx) {
                                                                    it.clinical_notes = e.value();
                                                                }
                                                                items.set(list);
                                                            },
                                                        }
                                                    }
                                                }

                                                button {
                                                    r#type: "button",
                                                    class: "mat-remove-btn",
                                                    title: "Remover este procedimento",
                                                    onclick: move |_| {
                                                        let mut list = items();
                                                        list.remove(item_idx);
                                                        items.set(list);
                                                    },
                                                    IconTrash { size: 14, color: "#dc2626".to_string() }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Observações Gerais e Condições do Orçamento
                    div { class: "input-group-wrapper full-width",
                        label { "Observações Gerais e Condições do Orçamento" }
                        textarea {
                            class: "modern-input-field",
                            rows: "2",
                            placeholder: "Ex: Valores válidos por 30 dias. Forma de pagamento: Entrada + 3x no cartão.",
                            value: "{plan_notes}",
                            oninput: move |e| plan_notes.set(e.value()),
                        }
                    }
                }

                // 3. Rodapé Fixo com Total e Ações
                div { class: "modal-footer",
                    div { class: "modal-footer-summary",
                        span { class: "summary-lbl", "Total do Orçamento:" }
                        strong { class: "summary-val font-mono", "{format_currency(total_cents)}" }
                        span { class: "summary-count", "({current_items.len()} procedimentos)" }
                    }

                    div { class: "modal-footer-buttons",
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            onclick: move |_| on_close.call(()),
                            "Cancelar"
                        }
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            disabled: is_submitting(),
                            onclick: handle_save_draft,
                            "Salvar Rascunho"
                        }
                        button {
                            r#type: "button",
                            class: "btn-primary",
                            disabled: is_submitting(),
                            onclick: handle_save_and_approve,
                            IconCheck { size: 16, color: "#ffffff".to_string() }
                            span { " Aprovar e Gerar Cobrança" }
                        }
                    }
                }
            }
        }
    }
}
