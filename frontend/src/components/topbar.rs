use dioxus::prelude::*;

#[component]
pub fn Topbar(user_name: String) -> Element {
    rsx! {
        div { class: "topbar",
            div { class: "topbar-user", "{user_name}" }
        }
    }
}
