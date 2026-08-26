use dioxus::prelude::*;

#[component]
pub fn IconArrowUpDown(
    #[props(default = 16)] size: u32,
    #[props(default = "currentColor".to_string())] color: String,
) -> Element {
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
            path { d: "M7 20V4m0 0l-4 4m4-4l4 4M17 4v16m0 0l4-4m-4 4l-4-4" }
        }
    }
}
