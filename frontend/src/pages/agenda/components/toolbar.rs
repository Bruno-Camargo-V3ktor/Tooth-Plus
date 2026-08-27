use crate::icons::{IconChevronLeft, IconChevronRight};
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
    let current_view = view_mode.read().clone();

    rsx! {
        div { class: "agenda-toolbar",
            div { class: "agenda-toolbar-left",
                select {
                    class: "agenda-select-prof",
                    value: "{dentist_filter}",
                    onchange: move |e| dentist_filter.set(e.value()),
                    option { value: "all", "Todos os profissionais" }
                    option { value: "usr:dr_lucas", "Dr. Lucas Mendes" }
                    option { value: "usr:dra_fernanda", "Dra. Fernanda Ramos" }
                    option { value: "usr:dra_luria", "Dra. Luria Silva" }
                }

                button {
                    r#type: "button",
                    class: "agenda-today-btn",
                    onclick: move |_| on_today.call(()),
                    "HOJE"
                }

                div { class: "agenda-nav-group",
                    button {
                        r#type: "button",
                        class: "agenda-nav-btn",
                        title: "Anterior",
                        onclick: move |_| on_prev.call(()),
                        IconChevronLeft { size: 16, color: "#94a3b8".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "agenda-nav-btn",
                        title: "Próximo",
                        onclick: move |_| on_next.call(()),
                        IconChevronRight { size: 16, color: "#94a3b8".to_string() }
                    }
                }

                span { class: "agenda-current-period", "{month_label}" }
            }

            div { class: "agenda-toolbar-right",
                input {
                    r#type: "date",
                    class: "agenda-date-input",
                    value: "{current_date_str}",
                    onchange: move |e| current_date_str.set(e.value()),
                }

                div { class: "agenda-view-toggle",
                    button {
                        r#type: "button",
                        class: if current_view == "week" { "agenda-view-btn active" } else { "agenda-view-btn" },
                        onclick: move |_| view_mode.set("week".to_string()),
                        "Semana"
                    }
                    button {
                        r#type: "button",
                        class: if current_view == "day" { "agenda-view-btn active" } else { "agenda-view-btn" },
                        onclick: move |_| view_mode.set("day".to_string()),
                        "Dia"
                    }
                }

                button {
                    r#type: "button",
                    class: "agenda-new-btn",
                    title: "Novo Agendamento",
                    onclick: move |_| on_open_new.call(()),
                    "+"
                }
            }
        }
    }
}
