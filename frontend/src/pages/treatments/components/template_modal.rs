use crate::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn TemplateModal(
    is_open: bool,
    name: Signal<String>,
    category: Signal<String>,
    description: Signal<String>,
    price_str: Signal<String>,
    duration_str: Signal<String>,
    materials: Signal<String>,
    equipment: Signal<String>,
    post_care: Signal<String>,
    target_teeth: Signal<Vec<String>>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let perm_teeth = vec![
        "18", "17", "16", "15", "14", "13", "12", "11",
        "21", "22", "23", "24", "25", "26", "27", "28",
        "48", "47", "46", "45", "44", "43", "42", "41",
        "31", "32", "33", "34", "35", "36", "37", "38",
    ];

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

            div { class: "form-field",
                label { class: "form-label", "Nome do Procedimento *" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: "Ex: Restauração em Resina Composta, Tratamento Endodôntico...",
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
                label { class: "form-label", "Duração Estimada" }
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

            // Seleção de dentes alvo padrão
            div { class: "form-field",
                label { class: "form-label", "Dentes Alvo / Regiões Frequentes (Opcional)" }
                div { style: "display: flex; flex-wrap: wrap; gap: 4px; max-height: 90px; overflow-y: auto; background: #0b1120; padding: 8px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.08);",
                    for t in perm_teeth {
                        {
                            let t_str = t.to_string();
                            let is_sel = target_teeth.read().contains(&t_str);
                            let chip_cls = if is_sel { "badge badge-blue" } else { "badge badge-gray" };
                            let t_clone = t_str.clone();

                            rsx! {
                                button {
                                    key: "{t}",
                                    r#type: "button",
                                    class: "{chip_cls}",
                                    style: "cursor: pointer; font-size: 11px; padding: 3px 8px;",
                                    onclick: move |_| {
                                        let mut list = target_teeth.read().clone();
                                        if list.contains(&t_clone) {
                                            list.retain(|x| x != &t_clone);
                                        } else {
                                            list.push(t_clone.clone());
                                        }
                                        target_teeth.set(list);
                                    },
                                    "{t}"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "📦 Insumos / Materiais (Estoque)" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Resina Composta, Ácido Fosfórico, Luvas...",
                        value: "{materials}",
                        oninput: move |e| materials.set(e.value()),
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "🛠️ Equipamentos Necessários" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Fotopolimerizador, Contra-ângulo...",
                        value: "{equipment}",
                        oninput: move |e| equipment.set(e.value()),
                    }
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Orientações Pós-Atendimento" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: "Ex: Evitar alimentos duros por 2 horas, compressa fria...",
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
