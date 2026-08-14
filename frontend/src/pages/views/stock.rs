pub use dioxus::prelude::*;

#[component]
pub fn StockView() -> Element {
    rsx! {
        div {
            div { class: "content-card",
                "Controle de insumos, materiais e estoque da clínica."
            }
        }
    }
}
