use crate::icons::{IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn TreatmentToolbar(
    search_query: Signal<String>,
    category_filter: Signal<String>,
    on_new_template: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "treatments-toolbar",
            div { style: "display: flex; align-items: center; gap: 12px; flex: 1;",
                div { class: "treatments-search-box",
                    IconSearch { size: 16, color: "#64748b".to_string() }
                    input {
                        r#type: "text",
                        placeholder: "Buscar procedimento por nome ou código...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                select {
                    class: "form-select",
                    style: "max-width: 220px; height: 38px;",
                    value: "{category_filter}",
                    onchange: move |e| category_filter.set(e.value()),
                    option { value: "ALL", "Todas as Categorias" }
                    option { value: "Dentística", "Dentística & Estética" }
                    option { value: "Endodontia", "Endodontia (Canal)" }
                    option { value: "Cirurgia", "Cirurgia & Exodontia" }
                    option { value: "Periodontia", "Periodontia & Profilaxia" }
                    option { value: "Ortodontia", "Ortodontia" }
                    option { value: "Prótese", "Prótese & Implante" }
                }
            }

            button {
                r#type: "button",
                class: "btn-new-patient-green",
                onclick: move |_| on_new_template.call(()),
                IconPlus { size: 16, color: "#ffffff".to_string() }
                span { "NOVO PROCEDIMENTO" }
            }
        }
    }
}
