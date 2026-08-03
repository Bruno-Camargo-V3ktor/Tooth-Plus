pub use dioxus::prelude::*;

#[component]
pub fn FinanceView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Fluxo de Caixa" }
            div { class: "content-card",
                "Financial data will be rendered here."
            }
        }
    }
}
