use dioxus::prelude::*;

#[component]
pub fn DocumentsView() -> Element {
    rsx! { div { h1 { class: "page-title", "Templates de Documentos" } div { class: "content-card", "Editor de contratos e termos." } } }
}
