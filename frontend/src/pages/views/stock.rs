pub use dioxus::prelude::*;

#[component]
pub fn StockView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Estoque" }
            div { class: "content-card",
                "Inventory data will be rendered here."
            }
        }
    }
}
