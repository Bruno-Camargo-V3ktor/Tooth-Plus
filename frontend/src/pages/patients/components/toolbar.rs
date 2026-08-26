use crate::icons::{IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn PatientToolbar(
    search_query: Signal<String>,
    on_search: EventHandler<()>,
    on_open_modal: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "patients-toolbar",
            div { class: "patients-search-box",
                IconSearch { size: 16, color: "#64748b".to_string() }
                input {
                    r#type: "text",
                    class: "patients-search-input",
                    placeholder: "Buscar paciente por nome, CPF ou telefone...",
                    value: "{search_query}",
                    oninput: move |e| {
                        search_query.set(e.value());
                        on_search.call(());
                    },
                }
            }

            div { class: "patients-actions",
                button {
                    r#type: "button",
                    class: "btn-new-patient",
                    onclick: move |_| on_open_modal.call(()),
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "Novo Paciente" }
                }
            }
        }
    }
}
