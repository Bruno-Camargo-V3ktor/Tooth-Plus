use crate::components::modal::Modal;
use dioxus::prelude::*;

#[component]
pub fn ModalItem(
    is_open: bool,
    is_editing: bool,
    name: Signal<String>,
    item_type: Signal<String>,
    unit_type: Signal<String>,
    current_stock: Signal<String>,
    min_stock: Signal<String>,
    cost_price: Signal<String>,
    manufacturer: Signal<String>,
    expiration_date: Signal<String>,
    batch_number: Signal<String>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let title = if is_editing { "Editar Item do Estoque" } else { "Cadastrar Novo Item no Estoque" };

    rsx! {
        Modal {
            title: title.to_string(),
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
                    "Salvar Item"
                }
            },

            div { style: "display: flex; flex-direction: column; gap: 14px;",
                div { class: "form-field",
                    label { class: "form-label", "Nome do Item / Material *" }
                    input {
                        class: "form-input",
                        r#type: "text",
                        placeholder: "Ex: Resina Filtek Z350 XT, Anestésico Lidocaína 2%...",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Tipo de Item *" }
                        select {
                            class: "form-select",
                            value: "{item_type}",
                            onchange: move |e| item_type.set(e.value()),
                            option { value: "material", "Material / Insumo" }
                            option { value: "chemical", "Químico / Medicamento" }
                            option { value: "equipment", "Equipamento Odontológico" }
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Unidade de Medida" }
                        select {
                            class: "form-select",
                            value: "{unit_type}",
                            onchange: move |e| unit_type.set(e.value()),
                            option { value: "un", "Unidade (un)" }
                            option { value: "cx", "Caixa (cx)" }
                            option { value: "pct", "Pacote (pct)" }
                            option { value: "fr", "Frasco (fr)" }
                            option { value: "tubete", "Tubete" }
                            option { value: "kit", "Kit" }
                        }
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Quantidade Inicial *" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            min: "0",
                            value: "{current_stock}",
                            oninput: move |e| current_stock.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Estoque Mínimo de Alerta *" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            min: "0",
                            value: "{min_stock}",
                            oninput: move |e| min_stock.set(e.value()),
                        }
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Preço de Custo (R$)" }
                        input {
                            class: "form-input",
                            r#type: "number",
                            step: "0.01",
                            placeholder: "0.00",
                            value: "{cost_price}",
                            oninput: move |e| cost_price.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Fabricante / Fornecedor" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "Ex: 3M ESPE, Dentsply, FGM, SDI...",
                            value: "{manufacturer}",
                            oninput: move |e| manufacturer.set(e.value()),
                        }
                    }
                }

                div { class: "form-row-2 form-row",
                    div { class: "form-field",
                        label { class: "form-label", "Data de Validade (Opcional)" }
                        input {
                            class: "form-input",
                            r#type: "date",
                            value: "{expiration_date}",
                            oninput: move |e| expiration_date.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", "Lote / Número de Série" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            placeholder: "Ex: LOT-2026-A89",
                            value: "{batch_number}",
                            oninput: move |e| batch_number.set(e.value()),
                        }
                    }
                }
            }
        }
    }
}
