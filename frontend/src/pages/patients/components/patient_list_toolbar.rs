use crate::icons::{IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn PatientListToolbar(
    search_query: Signal<String>,
    on_new_patient: EventHandler<()>,
    on_export: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "patients-header-row",
            h1 { class: "patients-main-title", "Pacientes" }

            div { class: "patients-search-and-actions",
                div { class: "patients-search-input-box",
                    IconSearch { size: 16, color: "#64748b".to_string() }
                    input {
                        r#type: "text",
                        placeholder: "Digite o nome do paciente, CPF, celular do paciente...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                button {
                    r#type: "button",
                    class: "btn-export",
                    onclick: move |_| on_export.call(()),
                    "⬇ EXPORTAR"
                }

                button {
                    r#type: "button",
                    class: "btn-new-patient-green",
                    onclick: move |_| on_new_patient.call(()),
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                    span { "NOVO PACIENTE" }
                }
            }
        }
    }
}
