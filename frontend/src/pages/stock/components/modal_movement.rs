use crate::components::modal::Modal;
use shared::stock::InventoryItem;
use dioxus::prelude::*;

#[component]
pub fn ModalMovement(
    is_open: bool,
    items: Vec<InventoryItem>,
    selected_item_id: Signal<String>,
    movement_type: Signal<String>,
    quantity_str: Signal<String>,
    reason: Signal<String>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    rsx! {
        Modal {
            title: "Registrar Movimentação de Estoque".to_string(),
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
                    "Confirmar Movimentação"
                }
            },

            div { class: "form-field",
                label { class: "form-label", "Item / Insumo *" }
                select {
                    class: "form-select",
                    value: "{selected_item_id}",
                    onchange: move |e| selected_item_id.set(e.value()),
                    for it in items {
                        option { key: "{it.id}", value: "{it.id}", "{it.name} (Atual: {it.current_stock} {it.unit_type})" }
                    }
                }
            }

            div { class: "form-row-2 form-row",
                div { class: "form-field",
                    label { class: "form-label", "Tipo de Movimentação *" }
                    select {
                        class: "form-select",
                        value: "{movement_type}",
                        onchange: move |e| movement_type.set(e.value()),
                        option { value: "ENTRY", "⬆️ Entrada (Compra / Reposição)" }
                        option { value: "EXIT", "⬇️ Saída (Uso Clínico / Descarte)" }
                    }
                }
                div { class: "form-field",
                    label { class: "form-label", "Quantidade *" }
                    input {
                        class: "form-input",
                        r#type: "number",
                        value: "{quantity_str}",
                        oninput: move |e| quantity_str.set(e.value()),
                    }
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Motivo / Observações" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: "Ex: Uso no procedimento do paciente X, reposição NF 482...",
                    value: "{reason}",
                    oninput: move |e| reason.set(e.value()),
                }
            }
        }
    }
}
