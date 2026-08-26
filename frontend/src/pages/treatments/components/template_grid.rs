use crate::icons::{IconClock, IconEdit, IconTrash};
use shared::treatments::TreatmentTemplate;
use dioxus::prelude::*;

#[component]
pub fn TemplateGrid(
    templates: Vec<TreatmentTemplate>,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    if templates.is_empty() {
        return rsx! {
            div { class: "empty-debits-box",
                h3 { class: "empty-debits-title", "Nenhum procedimento encontrado" }
                p { class: "empty-debits-desc", "Cadastre procedimentos padrão no catálogo para agilizar orçamentos e atendimentos." }
            }
        };
    }

    rsx! {
        div { class: "treatments-grid",
            for tmpl in templates {
                {
                    let tid = tmpl.id.clone();
                    let tid_del = tmpl.id.clone();
                    let price_fmt = format!("R$ {:.2}", tmpl.default_price_cents as f64 / 100.0);
                    let duration = tmpl.estimated_duration_minutes.unwrap_or(30);
                    let cat = tmpl.category.clone().unwrap_or_else(|| "Geral".to_string());
                    let desc = tmpl.description.clone().unwrap_or_else(|| "Procedimento odontológico padrão.".to_string());
                    let has_materials = !tmpl.required_materials.is_empty();
                    let has_equipment = !tmpl.required_equipment.is_empty();
                    let has_teeth = !tmpl.target_teeth.is_empty();

                    rsx! {
                        div { key: "{tmpl.id}", class: "treatment-card",
                            div { class: "treatment-card-top",
                                div {
                                    span { class: "treatment-card-cat", "{cat}" }
                                    h4 { class: "treatment-card-name", "{tmpl.name}" }
                                }
                                div { style: "display: flex; align-items: center; gap: 6px;",
                                    button {
                                        r#type: "button",
                                        class: "action-btn-icon",
                                        title: "Editar Procedimento",
                                        onclick: move |_| on_edit.call(tid.clone()),
                                        IconEdit { size: 14, color: "#94a3b8".to_string() }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "action-btn-icon",
                                        title: "Excluir Procedimento",
                                        onclick: move |_| on_delete.call(tid_del.clone()),
                                        IconTrash { size: 14, color: "#ef4444".to_string() }
                                    }
                                }
                            }

                            p { class: "treatment-card-desc", "{desc}" }

                            if has_teeth {
                                div { style: "display: flex; flex-wrap: wrap; gap: 4px;",
                                    span { style: "font-size: 11.5px; color: #64748b;", "Dentes:" }
                                    for t in tmpl.target_teeth.iter().take(8) {
                                        span { key: "{t}", class: "badge badge-blue", style: "font-size: 10.5px; padding: 1px 6px;", "{t}" }
                                    }
                                    if tmpl.target_teeth.len() > 8 {
                                        span { class: "badge badge-gray", style: "font-size: 10.5px; padding: 1px 6px;", "+{tmpl.target_teeth.len() - 8}" }
                                    }
                                }
                            }

                            if has_materials || has_equipment {
                                div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.06); padding: 8px 10px; border-radius: 6px; font-size: 11.5px; color: #94a3b8; display: flex; flex-direction: column; gap: 3px;",
                                    if has_materials {
                                        div { "📦 Insumos: {tmpl.required_materials.join(\", \")}" }
                                    }
                                    if has_equipment {
                                        div { "🛠️ Equipamentos: {tmpl.required_equipment.join(\", \")}" }
                                    }
                                }
                            }

                            div { class: "treatment-card-footer",
                                div { style: "display: flex; align-items: center; gap: 5px; color: #94a3b8; font-size: 12.5px;",
                                    IconClock { size: 14, color: "#64748b".to_string() }
                                    span { "{duration} min" }
                                }
                                span { class: "treatment-price-val", "{price_fmt}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
