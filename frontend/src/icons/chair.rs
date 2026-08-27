use dioxus::prelude::*;

#[component]
pub fn IconChair(#[props(default = 24)] size: u32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M6 4h12v7H6z" }
            path { d: "M4 11h16v4H4z" }
            path { d: "M6 15v5" }
            path { d: "M18 15v5" }
            path { d: "M9 11V4" }
            path { d: "M15 11V4" }
        }
    }
}
