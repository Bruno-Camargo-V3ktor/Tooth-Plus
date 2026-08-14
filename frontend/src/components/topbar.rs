use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Topbar(user_name: String) -> Element {
    let current_route = use_route::<Route>();
    let page_title = current_route.title();

    rsx! {
        header { class: "topbar",
            div { class: "topbar-left",
                h1 { class: "topbar-page-title", "{page_title}" }
            }
            div { class: "topbar-right",
                div { class: "topbar-user", "{user_name}" }
            }
        }
    }
}
