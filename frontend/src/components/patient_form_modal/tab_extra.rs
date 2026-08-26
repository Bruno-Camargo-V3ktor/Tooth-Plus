use dioxus::prelude::*;

#[component]
pub fn TabExtra(
    guardian_name: Signal<String>,
    guardian_phone: Signal<String>,
    notes: Signal<String>,
) -> Element {
    rsx! {
        div { class: "form-row-2 form-row",
            div { class: "form-field",
                label { class: "form-label", "Nome do responsável" }
                input { class: "form-input", r#type: "text", placeholder: "Nome completo", value: "{guardian_name}",
                    oninput: move |e| guardian_name.set(e.value()) }
            }
            div { class: "form-field",
                label { class: "form-label", "Celular do responsável" }
                input { class: "form-input", r#type: "tel", placeholder: "(00) 00000-0000", value: "{guardian_phone}",
                    oninput: move |e| guardian_phone.set(e.value()) }
            }
        }
        div { class: "form-field",
            label { class: "form-label", "Observação" }
            textarea { class: "form-textarea", placeholder: "Observações gerais sobre o paciente...",
                rows: "3", value: "{notes}",
                oninput: move |e| notes.set(e.value()) }
        }
    }
}
