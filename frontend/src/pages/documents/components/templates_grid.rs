use crate::icons::IconFileText;
use shared::documents::ContractTemplate;
use dioxus::prelude::*;

#[component]
pub fn TemplatesGrid(
    templates: Vec<ContractTemplate>,
    on_use_template: EventHandler<String>,
) -> Element {
    if templates.is_empty() {
        return rsx! {
            div { class: "empty-debits-box",
                h3 { class: "empty-debits-title", "Nenhum modelo cadastrado" }
                p { class: "empty-debits-desc", "Cadastre modelos padrão de contratos e atestados para agilizar emissões." }
            }
        };
    }

    rsx! {
        div { class: "treatments-grid",
            for tmpl in templates {
                {
                    let tid = tmpl.id.clone();
                    let desc = tmpl.description.clone().unwrap_or_else(|| "Modelo oficial da clínica.".to_string());

                    rsx! {
                        div { key: "{tmpl.id}", class: "treatment-card",
                            div { class: "treatment-card-top",
                                div {
                                    span { class: "treatment-card-cat", "{tmpl.category}" }
                                    h4 { class: "treatment-card-name", "{tmpl.title}" }
                                }
                                IconFileText { size: 20, color: "#38bdf8".to_string() }
                            }

                            p { class: "treatment-card-desc", "{desc}" }

                            div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.06); padding: 8px 10px; border-radius: 6px; font-size: 11.5px; color: #94a3b8; display: flex; flex-direction: column; gap: 3px;",
                                div { "✍️ Assinatura Paciente: " strong { if tmpl.requires_patient_signature { "Obrigatória" } else { "Opcional" } } }
                                div { "👨‍⚕️ Assinatura Dentista: " strong { if tmpl.requires_doctor_signature { "Obrigatória" } else { "Opcional" } } }
                            }

                            div { class: "treatment-card-footer",
                                button {
                                    r#type: "button",
                                    class: "btn-primary",
                                    style: "width: 100%; justify-content: center; font-size: 12.5px; font-weight: 700; padding: 6px 12px;",
                                    onclick: move |_| on_use_template.call(tid.clone()),
                                    "Emitir com este Modelo"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
