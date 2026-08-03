use dioxus::prelude::*;

#[component]
pub fn IconCalendar(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
            line { x1: "16", y1: "2", x2: "16", y2: "6" }
            line { x1: "8", y1: "2", x2: "8", y2: "6" }
            line { x1: "3", y1: "10", x2: "21", y2: "10" }
        }
    }
}

#[component]
pub fn IconUsers(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            path { d: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" }
            circle { cx: "9", cy: "7", r: "4" }
            path { d: "M23 21v-2a4 4 0 0 0-3-3.87" }
            path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
        }
    }
}

#[component]
pub fn IconFinance(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            line { x1: "12", y1: "1", x2: "12", y2: "23" }
            path { d: "M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" }
        }
    }
}

#[component]
pub fn IconBox(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            line { x1: "16.5", y1: "9.4", x2: "7.5", y2: "4.21" }
            path { d: "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" }
            polyline { points: "3.27 6.96 12 12.01 20.73 6.96" }
            line { x1: "12", y1: "22.08", x2: "12", y2: "12" }
        }
    }
}

#[component]
pub fn IconFile(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            line { x1: "16", y1: "13", x2: "8", y2: "13" }
            line { x1: "16", y1: "17", x2: "8", y2: "17" }
            polyline { points: "10 9 9 9 8 9" }
        }
    }
}

#[component]
pub fn IconBuilding(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            rect { x: "4", y: "2", width: "16", height: "20", rx: "2", ry: "2" }
            path { d: "M9 22v-4h6v4" }
            path { d: "M8 6h.01M16 6h.01M12 6h.01M12 10h.01M16 10h.01M8 10h.01M8 14h.01M12 14h.01M16 14h.01" }
        }
    }
}

#[component]
pub fn IconSettings(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" }
        }
    }
}

#[component]
pub fn IconLogout(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
            polyline { points: "16 17 21 12 16 7" }
            line { x1: "21", y1: "12", x2: "9", y2: "12" }
        }
    }
}

#[component]
pub fn IconMenu(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            line { x1: "3", y1: "12", x2: "21", y2: "12" }
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "3", y1: "18", x2: "21", y2: "18" }
        }
    }
}
