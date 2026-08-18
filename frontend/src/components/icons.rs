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

#[component]
pub fn IconAlertTriangle(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 18.75h.007v.008H12v-.008Z" }
        }
    }
}

#[component]
pub fn IconTool(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M11.42 15.17 17.25 21A2.652 2.652 0 0 0 21 17.25l-5.877-5.877M11.42 15.17l2.496-3.03c.317-.384.74-.626 1.208-.766M11.42 15.17l-4.655 5.653a2.548 2.548 0 1 1-3.586-3.586l6.837-5.63m5.108-.233c.55-.164 1.163-.188 1.743-.14a4.5 4.5 0 0 0 4.486-6.32l-3.276 3.277a3.004 3.004 0 0 1-2.25-2.25l3.276-3.276a4.5 4.5 0 0 0-6.32 4.486c.09.84.42 1.63 1.02 2.24Z" }
        }
    }
}

#[component]
pub fn IconFlask(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9.75 3.104v5.714a2.25 2.25 0 0 1-.659 1.591L5 14.5M9.75 3.104c-.251.023-.501.05-.75.082m.75-.082a24.301 24.301 0 0 1 4.5 0m0 0v5.714c0 .597.237 1.17.659 1.591L19.8 15.3M14.25 3.104c.251.023.501.05.75.082M19.8 15.3l-1.57.942A6 6 0 0 1 15 17H9a6 6 0 0 1-3.23-.942L4.2 15.1" }
        }
    }
}

#[component]
pub fn IconRefresh(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99" }
        }
    }
}

#[component]
pub fn IconArrowUp(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2.2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M4.5 10.5 12 3m0 0 7.5 7.5M12 3v18" }
        }
    }
}

#[component]
pub fn IconArrowDown(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2.2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19.5 13.5 12 21m0 0-7.5-7.5M12 21V3" }
        }
    }
}

#[component]
pub fn IconPaperclip(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "m18.375 12.739-7.693 7.693a4.5 4.5 0 0 1-6.364-6.364l10.94-10.94A3 3 0 1 1 19.5 7.373L8.559 18.32a1.5 1.5 0 1 1-2.122-2.122l8.25-8.25" }
        }
    }
}

#[component]
pub fn IconUpload(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5m-13.5-9L12 3m0 0 4.5 4.5M12 3v13.5" }
        }
    }
}

#[component]
pub fn IconExternalLink(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M13.5 6H5.25A2.25 2.25 0 0 0 3 8.25v10.5A2.25 2.25 0 0 0 5.25 21h10.5A2.25 2.25 0 0 0 18 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25" }
        }
    }
}

#[component]
pub fn IconTooth(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 2C8 2 5 4.5 5 8c0 3 1.5 6.5 3 11 1 3 2.5 3 4 1 1.5 2 3 2 4-1 1.5-4.5 3-8 3-11 0-3.5-3-6-7-6z" }
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 8c0 1.5 1.5 2.5 3 2.5s3-1 3-2.5" }
        }
    }
}

#[component]
pub fn IconSignature(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" }
        }
    }
}

#[component]
pub fn IconQrCode(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M14 14h3v3h-3zM14 20h3M20 14v3M20 20h.01" }
        }
    }
}

#[component]
pub fn IconHeartPulse(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19.5 12.572l-7.5 7.428-7.5-7.428A5 5 0 1 1 12 6.006a5 5 0 1 1 7.5 6.572" }
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M3.5 12h3l2-3 3 6 2-3h5" }
        }
    }
}

#[component]
pub fn IconShieldCheck(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z" }
        }
    }
}

#[component]
pub fn IconEye(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178Z" }
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" }
        }
    }
}

#[component]
pub fn IconDownload(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3" }
        }
    }
}

#[component]
pub fn IconCopy(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

#[component]
pub fn IconCheckCircle(size: i32, color: String) -> Element {
    rsx! {
        svg { style: "width: {size}px; height: {size}px; flex-shrink: 0;", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "{color}",
            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" }
        }
    }
}

#[component]
pub fn IconPhone(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            path { d: "M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" }
        }
    }
}

#[component]
pub fn IconFolder(size: i32, color: String) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", "viewBox": "0 0 24 24", fill: "none", stroke: "{color}", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
        }
    }
}

#[component]
pub fn IconKey(size: i32, color: String) -> Element {
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
            path { d: "m15.5 7.5 2.3 2.3a1 1 0 0 0 1.4 0l2.1-2.1a1 1 0 0 0 0-1.4L19 4" }
            path { d: "m21 2-9.6 9.6" }
            circle { cx: "7.5", cy: "15.5", r: "5.5" }
        }
    }
}

#[component]
pub fn IconMail(size: i32, color: String) -> Element {
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
            rect { width: "20", height: "16", x: "2", y: "4", rx: "2" }
            path { d: "m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" }
        }
    }
}
