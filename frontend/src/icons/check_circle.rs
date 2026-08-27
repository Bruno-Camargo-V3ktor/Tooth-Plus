use dioxus::prelude::*;

#[component]
pub fn IconCheckCircle(#[props(default = 24)] size: u32, #[props(default = "currentColor".to_string())] color: String) -> Element {
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
            path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
            polyline { points: "22 4 12 14.01 9 11.01" }
        }
    }
}
