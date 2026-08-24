use dioxus::prelude::*;

#[component]
pub fn IconAlertTriangle(#[props(default = 18)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
            line { x1: "12", y1: "9", x2: "12", y2: "13" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}
