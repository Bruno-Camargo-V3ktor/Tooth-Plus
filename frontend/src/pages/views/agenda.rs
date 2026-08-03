pub use dioxus::prelude::*;

#[component]
pub fn AgendaView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Agenda" }
            div { class: "content-card",
                "Calendar component will be rendered here."
            }
        }
    }
}
