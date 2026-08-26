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
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    rsx! {
        Modal {
            title: "Procedimento do Catálogo".to_string(),
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
                    placeholder: "Ex: Restauração em Resina Composta, Profilaxia...",
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
                label { class: "form-label", "Duração Estimada (minutos)" }
                select {
                    class: "form-select",
                    value: "{duration_str}",
                    onchange: move |e| duration_str.set(e.value()),
                    option { value: "15", "15 minutos" }
                    option { value: "30", "30 minutos" }
                    option { value: "45", "45 minutos" }
                    option { value: "60", "1 hora" }
                    option { value: "90", "1h30" }
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Descrição / Orientações Clínicas" }
                textarea {
                    class: "form-textarea",
                    placeholder: "Detalhes do procedimento e orientações ao paciente...",
                    rows: "3",
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                }
            }
        }
    }
}
