use crate::components::modal::Modal;
use crate::icons::{IconEdit, IconPlus, IconTag, IconTrash};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct PlanItem {
    pub id: String,
    pub name: String,
    pub discount_percentage: f64,
}

#[component]
pub fn TabPlans() -> Element {
    let mut plans = use_signal(|| vec![
        PlanItem {
            id: "plan:particular".to_string(),
            name: "Particular".to_string(),
            discount_percentage: 0.0,
        },
        PlanItem {
            id: "plan:uniodonto".to_string(),
            name: "Uniodonto".to_string(),
            discount_percentage: 15.0,
        },
        PlanItem {
            id: "plan:amil".to_string(),
            name: "Amil Dental".to_string(),
            discount_percentage: 10.0,
        },
    ]);

    let mut show_modal = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut plan_name = use_signal(String::new);
    let mut plan_discount = use_signal(|| "0".to_string());

    let handle_open_new = move |_| {
        editing_id.set(None);
        plan_name.set(String::new());
        plan_discount.set("0".to_string());
        show_modal.set(true);
    };

    let handle_save = move |_| {
        let name = plan_name.read().trim().to_string();
        if name.is_empty() {
            return;
        }
        let disc = plan_discount.read().parse::<f64>().unwrap_or(0.0);
        let mut list = plans.read().clone();

        if let Some(ref edit_id) = *editing_id.read() {
            if let Some(item) = list.iter_mut().find(|p| p.id == *edit_id) {
                item.name = name;
                item.discount_percentage = disc;
            }
        } else {
            let new_item = PlanItem {
                id: format!("plan:{}", list.len() + 1),
                name,
                discount_percentage: disc,
            };
            list.push(new_item);
        }
        plans.set(list);
        show_modal.set(false);
    };

    rsx! {
        div {
            div { class: "settings-card", style: "padding: 16px 20px; margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center;",
                div {
                    h3 { style: "font-size: 15px; font-weight: 700; color: var(--text-main, #f8fafc); margin: 0;", "Tabelas de Preços e Convênios" }
                    p { style: "font-size: 12.5px; color: var(--text-muted, #94a3b8); margin: 2px 0 0 0;", "Cadastre planos particulares e convênios para aplicar descontos automáticos nos orçamentos." }
                }
                button {
                    r#type: "button",
                    class: "btn-primary-blue",
                    style: "height: 38px; font-size: 13px; font-weight: 700; display: inline-flex; align-items: center; gap: 8px; padding: 0 18px;",
                    onclick: handle_open_new,
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "Novo Plano" }
                }
            }

            div { class: "settings-table-card",
                table { class: "settings-table",
                    thead {
                        tr {
                            th { "Plano / Convênio" }
                            th { "Desconto Padrão" }
                            th { style: "text-align: right; width: 100px;", "Ações" }
                        }
                    }
                    tbody {
                        for p in plans() {
                            {
                                let pid = p.id.clone();
                                let pid_del = p.id.clone();
                                let pname = p.name.clone();
                                let pdisc = p.discount_percentage;

                                rsx! {
                                    tr { key: "{p.id}",
                                        td {
                                            div { style: "display: flex; align-items: center; gap: 10px;",
                                                IconTag { size: 16, color: "var(--primary, #00a0e4)".to_string() }
                                                span { style: "font-weight: 700; color: var(--text-main, #f8fafc); font-size: 14px;", "{p.name}" }
                                            }
                                        }
                                        td {
                                            if pdisc > 0.0 {
                                                span { class: "badge badge-blue", style: "font-size: 12px; font-weight: 700;", "{pdisc}% de desconto" }
                                            } else {
                                                span { class: "badge badge-gray", style: "font-size: 12px;", "Tabela Base (0%)" }
                                            }
                                        }
                                        td { style: "text-align: right;",
                                            div { style: "display: inline-flex; align-items: center; gap: 8px;",
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Editar plano",
                                                    onclick: move |_| {
                                                        editing_id.set(Some(pid.clone()));
                                                        plan_name.set(pname.clone());
                                                        plan_discount.set(pdisc.to_string());
                                                        show_modal.set(true);
                                                    },
                                                    IconEdit { size: 15, color: "var(--text-muted, #94a3b8)".to_string() }
                                                }
                                                if p.id != "plan:particular" {
                                                    button {
                                                        r#type: "button",
                                                        class: "action-btn-icon",
                                                        title: "Excluir plano",
                                                        onclick: move |_| {
                                                            let mut list = plans.read().clone();
                                                            list.retain(|item| item.id != pid_del);
                                                            plans.set(list);
                                                        },
                                                        IconTrash { size: 15, color: "#ef4444".to_string() }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if show_modal() {
                Modal {
                    title: if editing_id().is_some() { "Editar Plano / Convênio".to_string() } else { "Novo Plano / Convênio".to_string() },
                    is_open: show_modal(),
                    on_close: move |_| show_modal.set(false),
                    footer: rsx! {
                        div { style: "display: flex; justify-content: flex-end; gap: 12px; width: 100%;",
                            button {
                                r#type: "button",
                                class: "btn-modal-ghost",
                                onclick: move |_| show_modal.set(false),
                                "CANCELAR"
                            }
                            button {
                                r#type: "button",
                                class: "btn-primary-blue",
                                style: "font-weight: 700; padding: 0 24px; height: 38px;",
                                onclick: handle_save,
                                "SALVAR PLANO"
                            }
                        }
                    },
                    div { style: "display: flex; flex-direction: column; gap: 14px;",
                        div { class: "form-field",
                            label { class: "form-label", "Nome do Plano / Convênio *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Bradesco Dental, OdontoPrev, Particular...",
                                value: "{plan_name}",
                                oninput: move |e| plan_name.set(e.value()),
                            }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Desconto Aplicado na Tabela Base (%)" }
                            input {
                                class: "form-input",
                                r#type: "number",
                                min: "0",
                                max: "100",
                                value: "{plan_discount}",
                                oninput: move |e| plan_discount.set(e.value()),
                            }
                        }
                    }
                }
            }
        }
    }
}
