use dioxus::prelude::*;

#[component]
pub fn IconFilter(#[props(default = 18)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
        }
    }
}
