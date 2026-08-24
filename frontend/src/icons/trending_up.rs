use dioxus::prelude::*;

#[component]
pub fn IconTrendingUp(#[props(default = 20)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "1.8",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "23 6 13.5 15.5 8.5 10.5 1 18" }
            polyline { points: "17 6 23 6 23 12" }
        }
    }
}
