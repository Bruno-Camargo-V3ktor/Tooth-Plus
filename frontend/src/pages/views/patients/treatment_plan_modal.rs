//! # Modal de Elaboração de Orçamento / Plano de Tratamento
//!
//! Permite montar planos de tratamento completos vinculados ao paciente,
//! selecionando procedimentos padrão do catálogo da clínica ou criando itens do zero,
//! com totalizador automático em destaque e opção de aprovação com emissão de cobrança financeira.

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
        async move { fetch_treatment_templates(&t, &cid).await.unwrap_or_default() }
    });

    let templates_list: Vec<TreatmentTemplate> = templates_res.read().clone().unwrap_or_default();

    // Helper: Adicionar item a partir de um template
    let mut add_template_item = move |tmpl: TreatmentTemplate| {
        let reals = tmpl.default_price_cents / 100;
        let centavos = tmpl.default_price_cents % 100;
        let new_item = PlanFormItem {
            template_id: Some(tmpl.id),
            procedure_name: tmpl.name,
            category: tmpl.category.unwrap_or_else(|| "Geral".into()),
            tooth_number: String::new(),
            dental_region: tmpl.dental_regions.first().cloned().unwrap_or_default(),
            surfaces: vec![],
            price_reals_str: format!("{}.{:02}", reals, centavos),
            clinical_notes: tmpl.clinical_notes.unwrap_or_default(),
        };
        let mut list = items();
        list.push(new_item);
        items.set(list);
    };

    // Helper: Adicionar item personalizado do zero
    let mut add_custom_item = move || {
        let new_item = PlanFormItem {
            template_id: None,
            procedure_name: String::new(),
            category: "Dentística".to_string(),
            tooth_number: String::new(),
            dental_region: String::new(),
            surfaces: vec![],
            price_reals_str: "0.00".to_string(),
            clinical_notes: String::new(),
        };
        let mut list = items();
        list.push(new_item);
        items.set(list);
    };

    // Helper: Remover item da lista
    let mut remove_item = move |index: usize| {
        let mut list = items();
        if index < list.len() {
            list.remove(index);
            items.set(list);
        }
    };

    // Cálculo dinâmico do total em centavos
    let current_items = items();
    let total_budget_cents: i64 = current_items
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

    // Handler de Salvamento como Rascunho
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

    // Handler de Aprovação Direta e Geração de Cobrança
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

    let items_count = current_items.len();
    let proc_label = if items_count == 1 { "procedimento incluído" } else { "procedimentos incluídos" };

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal-card treatment-plan-modal-card",
                style: "max-width: 960px; width: 95vw; max-height: 92vh; display: flex; flex-direction: column; border-radius: 16px; background: #ffffff; box-shadow: 0 25px 60px -15px rgba(15, 23, 42, 0.3);",
                onclick: move |e| e.stop_propagation(),

                // 1. Header do Modal (Espaçoso e Elegante)
                div { class: "modal-header", style: "padding: 22px 28px; border-bottom: 1px solid #e2e8f0; display: flex; align-items: center; justify-content: space-between;",
                    div { class: "modal-header-left flex items-center gap-3",
                        div { class: "stock-header-icon-box", style: "width: 44px; height: 44px; border-radius: 10px; background: #eff6ff; border: 1px solid #bfdbfe; display: flex; align-items: center; justify-content: center; color: #0284c7;",
                            IconTooth { size: 22, color: "#0284c7".to_string() }
                        }
                        div { class: "header-text-col",
                            h2 { class: "modal-title", style: "font-size: 1.25rem; font-weight: 700; color: #0f172a; margin: 0;",
                                if editing_plan.is_some() {
                                    "Editar Orçamento / Plano de Tratamento"
                                } else {
                                    "Novo Orçamento / Plano de Tratamento"
                                }
                            }
                            p { class: "modal-subtitle", style: "font-size: 0.85rem; color: #64748b; margin-top: 4px; margin-bottom: 0;",
                                "Monte a proposta com procedimentos tabelados ou personalizados."
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "modal-close-btn",
                        style: "background: transparent; border: none; font-size: 26px; color: #94a3b8; cursor: pointer; padding: 4px 8px; line-height: 1;",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                // 2. Corpo do Modal (Espaçoso e com Respiro Visual)
                div { class: "modal-body treatment-modal-scroll", style: "padding: 26px 28px; overflow-y: auto; flex: 1; display: flex; flex-direction: column; gap: 22px;",
                    // Informações Gerais do Orçamento
                    div { style: "display: grid; grid-template-columns: 2fr 1fr; gap: 20px;",
                        div { class: "input-group-wrapper",
                            label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Título do Orçamento *" }
                            input {
                                r#type: "text",
                                class: "modern-input-field",
                                style: "width: 100%; height: 44px; padding: 0 16px; border: 1px solid #cbd5e1; border-radius: 8px; font-size: 0.95rem;",
                                placeholder: "Ex: Reabilitação Oral, Restauração e Canal...",
                                value: "{plan_title}",
                                oninput: move |e| plan_title.set(e.value()),
                            }
                        }
                        div { class: "input-group-wrapper",
                            label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Previsão de Início" }
                            input {
                                r#type: "date",
                                class: "modern-input-field font-mono",
                                style: "width: 100%; height: 44px; padding: 0 16px; border: 1px solid #cbd5e1; border-radius: 8px; font-size: 0.95rem;",
                                value: "{start_date}",
                                oninput: move |e| start_date.set(e.value()),
                            }
                        }
                    }

                    // Seção Construtor de Procedimentos
                    div { style: "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 12px; padding: 22px 24px;",
                        div { class: "flex items-center justify-between", style: "margin-bottom: 16px;",
                            div { class: "flex items-center gap-2",
                                IconTooth { size: 18, color: "#0284c7".to_string() }
                                strong { style: "color: #0f172a; font-size: 1rem;", "Procedimentos do Orçamento" }
                            }
                            span { class: "badge-status-neutral", style: "background: #e2e8f0; color: #475569; font-size: 0.8rem; padding: 4px 12px; border-radius: 9999px; font-weight: 600;",
                                "{items_count} procedimentos"
                            }
                        }

                        // Barra Seletora de Catálogo & Item Personalizado
                        div { style: "display: grid; grid-template-columns: 1fr auto; gap: 14px; margin-bottom: 18px;",
                            select {
                                class: "modern-input-field modern-select",
                                style: "height: 44px; padding: 0 16px; border: 1px solid #cbd5e1; border-radius: 8px; background: #ffffff; font-size: 0.92rem;",
                                value: "{selected_template_id}",
                                onchange: move |e| {
                                    let val = e.value();
                                    if let Some(tmpl) = templates_list.iter().find(|t| t.id == val) {
                                        add_template_item(tmpl.clone());
                                    }
                                    selected_template_id.set(String::new());
                                },
                                option { value: "", "+ Selecionar Procedimento Padrão do Catálogo..." }
                                for tmpl in templates_list.iter() {
                                    {
                                        let cat = tmpl.category.as_deref().unwrap_or("Geral");
                                        rsx! {
                                            option { value: "{tmpl.id}",
                                                "{tmpl.name} ({format_currency(tmpl.default_price_cents)}) - {cat}"
                                            }
                                        }
                                    }
                                }
                            }

                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "height: 44px; padding: 0 20px; display: flex; align-items: center; gap: 8px; border: 1px solid #cbd5e1; background: #ffffff; border-radius: 8px; font-size: 0.92rem; font-weight: 600;",
                                onclick: move |_| add_custom_item(),
                                IconPlus { size: 16, color: "currentColor".to_string() }
                                span { "Item Personalizado" }
                            }
                        }

                        // Lista de Itens do Orçamento
                        if current_items.is_empty() {
                            div { style: "text-align: center; padding: 36px 20px; background: #ffffff; border: 1px dashed #cbd5e1; border-radius: 10px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px;",
                                IconTooth { size: 36, color: "#94a3b8".to_string() }
                                p { style: "color: #475569; font-size: 0.95rem; font-weight: 600; margin: 0;", "Nenhum procedimento adicionado ao orçamento." }
                                p { style: "color: #94a3b8; font-size: 0.85rem; margin: 0;", "Selecione um procedimento do catálogo ou adicione um item personalizado acima." }
                            }
                        } else {
                            div { style: "display: flex; flex-direction: column; gap: 14px;",
                                for (idx, item) in current_items.iter().enumerate() {
                                    {
                                        let item_idx = idx;
                                        rsx! {
                                            div {
                                                key: "{idx}",
                                                style: "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 10px; padding: 18px 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.03);",
                                                div { style: "display: grid; grid-template-columns: auto 2.5fr 1.2fr 1fr auto; gap: 14px; align-items: flex-end;",
                                                    div {
                                                        label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 6px;", "Item" }
                                                        div { style: "height: 42px; display: flex; align-items: center; justify-content: center; background: #f1f5f9; border: 1px solid #e2e8f0; padding: 0 14px; border-radius: 8px; font-size: 0.9rem; font-weight: 700; font-family: monospace; color: #475569;",
                                                            "#{idx + 1}"
                                                        }
                                                    }

                                                    div {
                                                        label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 6px;", "Procedimento *" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field font-semibold text-slate-800",
                                                            style: "width: 100%; height: 42px; padding: 0 14px; border: 1px solid #cbd5e1; border-radius: 8px; font-size: 0.92rem;",
                                                            placeholder: "Ex: Restauração Resina 2 Faces",
                                                            value: "{item.procedure_name}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if item_idx < list.len() {
                                                                    list[item_idx].procedure_name = e.value();
                                                                    items.set(list);
                                                                }
                                                            },
                                                        }
                                                    }

                                                    div {
                                                        label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 6px;", "Dente / Região" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field",
                                                            style: "width: 100%; height: 42px; padding: 0 14px; border: 1px solid #cbd5e1; border-radius: 8px; font-size: 0.92rem;",
                                                            placeholder: "Ex: 16, 21, Superior",
                                                            value: "{item.tooth_number}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if item_idx < list.len() {
                                                                    list[item_idx].tooth_number = e.value();
                                                                    items.set(list);
                                                                }
                                                            },
                                                        }
                                                    }

                                                    div {
                                                        label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 6px;", "Valor (R$)" }
                                                        input {
                                                            r#type: "text",
                                                            class: "modern-input-field font-mono text-right font-semibold",
                                                            style: "width: 100%; height: 42px; padding: 0 14px; border: 1px solid #cbd5e1; border-radius: 8px; font-size: 0.92rem;",
                                                            placeholder: "0.00",
                                                            value: "{item.price_reals_str}",
                                                            oninput: move |e| {
                                                                let mut list = items();
                                                                if item_idx < list.len() {
                                                                    list[item_idx].price_reals_str = e.value();
                                                                    items.set(list);
                                                                }
                                                            },
                                                        }
                                                    }

                                                    div {
                                                        label { style: "font-size: 0.75rem; color: transparent; font-weight: 600; display: block; margin-bottom: 6px;", "Ação" }
                                                        button {
                                                            r#type: "button",
                                                            class: "btn-action-icon text-danger",
                                                            style: "width: 42px; height: 42px; display: flex; align-items: center; justify-content: center; border: 1px solid #fecaca; border-radius: 8px; background: #fff1f2; color: #ef4444; cursor: pointer;",
                                                            title: "Remover este procedimento",
                                                            onclick: move |_| remove_item(item_idx),
                                                            IconTrash { size: 18, color: "#ef4444".to_string() }
                                                        }
                                                    }
                                                }

                                                // Linha de Observações Clínicas / Técnicas
                                                div { style: "margin-top: 14px;",
                                                    label { style: "font-size: 0.75rem; color: #64748b; font-weight: 600; display: block; margin-bottom: 6px;", "Notas Clínicas / Técnica" }
                                                    input {
                                                        r#type: "text",
                                                        class: "modern-input-field",
                                                        style: "width: 100%; height: 38px; padding: 0 14px; border: 1px solid #e2e8f0; background: #f8fafc; border-radius: 6px; font-size: 0.88rem; color: #475569;",
                                                        placeholder: "Observações específicas deste procedimento...",
                                                        value: "{item.clinical_notes}",
                                                        oninput: move |e| {
                                                            let mut list = items();
                                                            if item_idx < list.len() {
                                                                list[item_idx].clinical_notes = e.value();
                                                                items.set(list);
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Observações Gerais e Condições Comerciais
                    div { class: "form-group",
                        label { style: "font-size: 0.85rem; font-weight: 600; color: #334155; margin-bottom: 8px; display: block;", "Observações Gerais e Condições do Orçamento" }
                        textarea {
                            class: "modern-input-field",
                            style: "width: 100%; padding: 12px 16px; border: 1px solid #cbd5e1; border-radius: 8px; font-size: 0.9rem; min-height: 80px; resize: vertical;",
                            rows: "3",
                            placeholder: "Ex: Valores válidos por 30 dias. Forma de pagamento: Entrada + 3x no cartão.",
                            value: "{plan_notes}",
                            oninput: move |e| plan_notes.set(e.value()),
                        }
                    }
                }

                // 3. Rodapé do Modal (Design Premium em Barra Unificada e Alinhada)
                div {
                    class: "modal-footer-custom",
                    style: "display: flex; align-items: center; justify-content: space-between; padding: 20px 28px; background: #f8fafc; border-top: 1px solid #e2e8f0; border-bottom-left-radius: 16px; border-bottom-right-radius: 16px; gap: 20px; flex-wrap: nowrap;",
                    
                    // Lado Esquerdo: Bloco de Totalizador com Destaque
                    div { class: "budget-modal-total-box", style: "display: flex; flex-direction: column; gap: 2px;",
                        span { style: "font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase; letter-spacing: 0.5px;",
                            "Valor Total da Proposta"
                        }
                        div { class: "flex items-baseline gap-2",
                            span { style: "font-size: 26px; font-weight: 800; color: #0284c7; font-family: monospace; line-height: 1.1; letter-spacing: -0.02em;",
                                "{format_currency(total_budget_cents)}"
                            }
                            span { style: "font-size: 13px; color: #64748b; font-weight: 500;",
                                "({items_count} {proc_label})"
                            }
                        }
                    }

                    // Lado Direito: Ações Alinhadas em Linha Única
                    div { class: "budget-modal-actions-box", style: "display: flex; align-items: center; gap: 12px; flex-shrink: 0;",
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            style: "height: 44px; padding: 0 20px; border: 1px solid #cbd5e1; background: #ffffff; border-radius: 8px; font-size: 13.5px; font-weight: 600; color: #475569; cursor: pointer; transition: all 0.2s ease;",
                            disabled: is_submitting(),
                            onclick: move |_| on_close.call(()),
                            "Cancelar"
                        }
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            style: "height: 44px; padding: 0 20px; border: 1px solid #cbd5e1; background: #ffffff; border-radius: 8px; font-size: 13.5px; font-weight: 600; color: #1e293b; cursor: pointer; transition: all 0.2s ease;",
                            disabled: is_submitting(),
                            onclick: handle_save_draft,
                            if is_submitting() { "Salvando..." } else { "Salvar Rascunho" }
                        }
                        button {
                            r#type: "button",
                            class: "btn-primary",
                            style: "height: 44px; padding: 0 24px; display: flex; align-items: center; gap: 8px; background: #0284c7; border: none; border-radius: 8px; color: #ffffff; font-size: 14px; font-weight: 700; cursor: pointer; box-shadow: 0 2px 6px rgba(2, 132, 199, 0.35); transition: all 0.2s ease;",
                            disabled: is_submitting(),
                            onclick: handle_save_and_approve,
                            IconCheck { size: 18, color: "#ffffff".to_string() }
                            span { if is_submitting() { "Processando..." } else { "Aprovar e Gerar Cobrança" } }
                        }
                    }
                }
            }
        }
    }
}
