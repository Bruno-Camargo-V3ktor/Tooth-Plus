pub use dioxus::prelude::*;

#[component]
pub fn PatientsView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Pacientes" }
            div { class: "content-card",
                "Patient list and registration form will be rendered here."
            }
        }
    }
}
