use dioxus::prelude::*;

#[component]
pub fn IconTooth(#[props(default = 20)] size: i32, #[props(default = "currentColor".to_string())] color: String) -> Element {
    rsx! {
        svg {
            style: "width: {size}px; height: {size}px; flex-shrink: 0;",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 2C8 2 5 4.5 5 8c0 3 1.5 6.5 3 11 1 3 2.5 3 4 1 1.5 2 3 2 4-1 1.5-4.5 3-8 3-11 0-3.5-3-6-7-6z" }
            path { d: "M9 8c0 1.5 1.5 2.5 3 2.5s3-1 3-2.5" }
        }
    }
}
