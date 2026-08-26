use dioxus::prelude::*;

#[component]
pub fn IconInfo(
    #[props(default = 24)] size: i32,
    #[props(default = "currentColor".to_string())] color: String,
) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        }
    }
}
