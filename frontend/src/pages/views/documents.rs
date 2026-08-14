use dioxus::prelude::*;

#[component]
pub fn DocumentsView() -> Element {
    rsx! {
        div {
            div { class: "content-card", "Editor e emissão de contratos, termos e documentos." }
        }
    }
}
