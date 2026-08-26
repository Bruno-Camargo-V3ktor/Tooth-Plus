use dioxus::prelude::*;

#[component]
pub fn IconPrinter(
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
            polyline { points: "6 9 6 2 18 2 18 9" }
            path { d: "M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" }
            rect { x: "6", y: "14", width: "12", height: "8" }
        }
    }
}
