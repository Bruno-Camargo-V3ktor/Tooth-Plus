use dioxus::prelude::*;

#[component]
pub fn IconTrash(#[props(default = 18)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "3 6 5 6 21 6" }
            path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
        }
    }
}
