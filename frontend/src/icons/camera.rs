use dioxus::prelude::*;

#[component]
pub fn IconCamera(#[props(default = 24)] size: u32, #[props(default = "currentColor".to_string())] color: String) -> Element {
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
            path { d: "M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z" }
            circle { cx: "12", cy: "13", r: "4" }
        }
    }
}
