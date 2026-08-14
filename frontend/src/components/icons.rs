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

#[component]
pub fn IconSearch(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" }
        }
    }
}

#[component]
pub fn IconPlus(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
        }
    }
}

#[component]
pub fn IconLock(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.5", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0V10.5m-2.25 0h13.5c.621 0 1.125.504 1.125 1.125v7.497c0 .621-.504 1.125-1.125 1.125H3.75c-.621 0-1.125-.504-1.125-1.125v-7.497c0-.621.504-1.125 1.125-1.125Z" }
        }
    }
}

#[component]
pub fn IconEdit(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.8", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" }
        }
    }
}

#[component]
pub fn IconTrash(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.8", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" }
        }
    }
}

#[component]
pub fn IconPower(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.8", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9" }
        }
    }
}

#[component]
pub fn IconChevronDown(size: i32, color: String, class: Option<String>) -> Element {
    let extra_class = class.unwrap_or_default();
    rsx! {
        svg { class: "{extra_class}", style: "width: {size}px; height: {size}px; flex-shrink: 0; transition: transform 0.2s;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19.5 8.25l-7.5 7.5-7.5-7.5" }
        }
    }
}

#[component]
pub fn IconChevronLeft(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 19.5L8.25 12l7.5-7.5" }
        }
    }
}

#[component]
pub fn IconChevronRight(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M8.25 4.5l7.5 7.5-7.5 7.5" }
        }
    }
}

#[component]
pub fn IconClock(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        }
    }
}

#[component]
pub fn IconCheck(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

#[component]
pub fn IconX(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        }
    }
}

#[component]
pub fn IconFilter(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
        }
    }
}
