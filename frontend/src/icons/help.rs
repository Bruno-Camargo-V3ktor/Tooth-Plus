use dioxus::prelude::*;

#[component]
pub fn IconHelp(#[props(default = 20)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}
