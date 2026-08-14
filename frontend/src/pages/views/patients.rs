pub use dioxus::prelude::*;

#[component]
pub fn PatientsView() -> Element {
    rsx! {
        div {
            div { class: "content-card",
                "Gestão de pacientes e prontuários clínicos."
            }
        }
    }
}
