use dioxus::prelude::*;

#[component]
pub fn TabAddress(
    address_zip: Signal<String>,
    address_street: Signal<String>,
    address_number: Signal<String>,
    address_neighborhood: Signal<String>,
    address_city: Signal<String>,
    address_state: Signal<String>,
) -> Element {
    rsx! {
        div { class: "form-row-2 form-row",
            div { class: "form-field",
                label { class: "form-label", "CEP" }
                input { class: "form-input", r#type: "text", placeholder: "00000-000", value: "{address_zip}",
                    oninput: move |e| address_zip.set(e.value()) }
            }
            div { class: "form-field",
                label { class: "form-label", "Logradouro" }
                input { class: "form-input", r#type: "text", placeholder: "Rua, Av.", value: "{address_street}",
                    oninput: move |e| address_street.set(e.value()) }
            }
        }
        div { class: "form-row-3 form-row",
            div { class: "form-field",
                label { class: "form-label", "Número" }
                input { class: "form-input", r#type: "text", placeholder: "Nº", value: "{address_number}",
                    oninput: move |e| address_number.set(e.value()) }
            }
            div { class: "form-field",
                label { class: "form-label", "Bairro" }
                input { class: "form-input", r#type: "text", placeholder: "Bairro", value: "{address_neighborhood}",
                    oninput: move |e| address_neighborhood.set(e.value()) }
            }
            div { class: "form-field",
                label { class: "form-label", "Cidade / UF" }
                div { style: "display: flex; gap: 8px;",
                    input { class: "form-input", r#type: "text", placeholder: "Cidade", value: "{address_city}",
                        oninput: move |e| address_city.set(e.value()) }
                    input { class: "form-input", r#type: "text", placeholder: "UF", style: "max-width: 60px;", value: "{address_state}",
                        oninput: move |e| address_state.set(e.value()) }
                }
            }
        }
    }
}
