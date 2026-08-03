use dioxus::prelude::*;

#[component]
pub fn ClinicsView() -> Element {
    rsx! { div { h1 { class: "page-title", "Unidades (Filiais)" } div { class: "content-card", "Gestão de multi-tenants." } } }
}
