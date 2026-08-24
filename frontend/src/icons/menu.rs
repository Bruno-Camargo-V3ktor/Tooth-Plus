use dioxus::prelude::*;

#[component]
pub fn IconMenu(#[props(default = 20)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "3", y1: "12", x2: "21", y2: "12" }
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "3", y1: "18", x2: "21", y2: "18" }
        }
    }
}
