use dioxus::prelude::*;

#[component]
pub fn IconBuilding(#[props(default = 24)] size: u32, #[props(default = "currentColor".to_string())] color: String) -> Element {
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
            rect { x: "4", y: "2", width: "16", height: "20", rx: "2", ry: "2" }
            line { x1: "9", y1: "22", x2: "9", y2: "22.01" }
            line { x1: "15", y1: "22", x2: "15", y2: "22.01" }
            line { x1: "9", y1: "6", x2: "9", y2: "6.01" }
            line { x1: "15", y1: "6", x2: "15", y2: "6.01" }
            line { x1: "9", y1: "10", x2: "9", y2: "10.01" }
            line { x1: "15", y1: "10", x2: "15", y2: "10.01" }
            line { x1: "9", y1: "14", x2: "9", y2: "14.01" }
            line { x1: "15", y1: "14", x2: "15", y2: "14.01" }
            line { x1: "9", y1: "18", x2: "9", y2: "18.01" }
            line { x1: "15", y1: "18", x2: "15", y2: "18.01" }
        }
    }
}
