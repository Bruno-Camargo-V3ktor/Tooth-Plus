use crate::icons::{IconChevronLeft, IconChevronRight, IconPlus};
use dioxus::prelude::*;

#[component]
pub fn AgendaToolbar(
    dentist_filter: Signal<String>,
    view_mode: Signal<String>,
    current_date_str: Signal<String>,
    month_label: String,
    on_prev: EventHandler<()>,
    on_today: EventHandler<()>,
    on_next: EventHandler<()>,
    on_open_new: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "agenda-toolbar",
            div { class: "agenda-toolbar-left",
                select {
                    class: "agenda-dentist-select",
                    value: "{dentist_filter}",
                    onchange: move |e| dentist_filter.set(e.value()),
                    option { value: "all", "Todos os profissionais" }
                    option { value: "usr-1", "Dr. Roberto Alencar" }
                    option { value: "usr-2", "Dr. Lucas Mendes" }
                }

                button {
                    r#type: "button",
                    class: "btn-today",
                    onclick: move |_| on_today.call(()),
                    "HOJE"
                }

                div { class: "agenda-nav-arrows",
                    button {
                        r#type: "button",
                        class: "btn-arrow",
                        onclick: move |_| on_prev.call(()),
                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "btn-arrow",
                        onclick: move |_| on_next.call(()),
                        IconChevronRight { size: 16, color: "currentColor".to_string() }
                    }
                }

                span { class: "agenda-current-month-label", "{month_label}" }
            }

            div { class: "agenda-toolbar-right",
                input {
                    r#type: "date",
                    class: "agenda-date-picker-input",
                    value: "{current_date_str}",
                    onchange: move |e| current_date_str.set(e.value()),
                }

                div { class: "agenda-view-mode-toggle",
                    button {
                        r#type: "button",
                        class: if *view_mode.read() == "week" { "view-toggle-btn active" } else { "view-toggle-btn" },
                        onclick: move |_| view_mode.set("week".to_string()),
                        "Semana"
                    }
                    button {
                        r#type: "button",
                        class: if *view_mode.read() == "day" { "view-toggle-btn active" } else { "view-toggle-btn" },
                        onclick: move |_| view_mode.set("day".to_string()),
                        "Dia"
                    }
                }

                button {
                    r#type: "button",
                    class: "btn-new-appointment",
                    onclick: move |_| on_open_new.call(()),
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                }
            }
        }
    }
}
