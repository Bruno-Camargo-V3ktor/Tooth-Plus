use dioxus::prelude::*;

#[component]
pub fn IconCopy(
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
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}
