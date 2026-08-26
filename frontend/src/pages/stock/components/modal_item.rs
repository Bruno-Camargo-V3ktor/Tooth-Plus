use crate::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn ModalNewItem(
    is_open: bool,
    name: Signal<String>,
    category: Signal<String>,
    unit_type: Signal<String>,
    current_stock_str: Signal<String>,
    min_stock_str: Signal<String>,
    manufacturer: Signal<String>,
    cost_price_str: Signal<String>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    rsx! {
        Modal {
            title: "Cadastrar Novo Produto / Insumo".to_string(),
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
                    "Cadastrar Produto"
                }
            },

            div { class: "form-field",
                label { class: "form-label", "Nome do Produto / Descrição *" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: "Ex: Resina Composta A2, Luvas de Látex P...",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Tipo do Item" }
                    select {
                        class: "form-select",
                        value: "{category}",
                        onchange: move |e| category.set(e.value()),
                        option { value: "material", "Material Clínico" }
                        option { value: "chemical", "Químico / Medicamento" }
                        option { value: "equipment", "Equipamento / Instrumental" }
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Fabricante / Marca" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: 3M, Dentsply, SDI...",
                        value: "{manufacturer}",
                        oninput: move |e| manufacturer.set(e.value()),
                    }
                }
            }

            div { class: "form-row-3 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Unidade de Medida" }
                    select {
                        class: "form-select",
                        value: "{unit_type}",
                        onchange: move |e| unit_type.set(e.value()),
                        option { value: "un", "Unidade (un)" }
                        option { value: "cx", "Caixa (cx)" }
                        option { value: "pct", "Pacote (pct)" }
                        option { value: "ml", "Mililitros (ml)" }
                        option { value: "g", "Gramas (g)" }
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Estoque Inicial *" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        value: "{current_stock_str}",
                        oninput: move |e| current_stock_str.set(e.value()),
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Estoque Mínimo *" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        value: "{min_stock_str}",
                        oninput: move |e| min_stock_str.set(e.value()),
                    }
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Preço de Custo Unitário (R$)" }
                input {
                    class: "form-input",
                    r#type: "number",
                    step: "0.01",
                    placeholder: "0.00",
                    value: "{cost_price_str}",
                    oninput: move |e| cost_price_str.set(e.value()),
                }
            }
        }
    }
}
