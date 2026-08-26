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
    let current_view = view_mode.read().clone();

    rsx! {
        div { class: "agenda-toolbar",
            div { class: "agenda-toolbar-left",
                select {
                    class: "agenda-dentist-select",
                    value: "{dentist_filter}",
                    onchange: move |e| dentist_filter.set(e.value()),
                    option { value: "all", "Todos os profissionais" }
                    option { value: "usr:dr_lucas", "Dr. Lucas Mendes" }
                    option { value: "usr:dra_fernanda", "Dra. Fernanda Ramos" }
                    option { value: "usr:dra_luria", "Dra. Luria Silva" }
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
                        title: "Anterior",
                        onclick: move |_| on_prev.call(()),
                        IconChevronLeft { size: 16, color: "#94a3b8".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "btn-arrow",
                        title: "Próximo",
                        onclick: move |_| on_next.call(()),
                        IconChevronRight { size: 16, color: "#94a3b8".to_string() }
                    }
                }

                span { class: "agenda-current-month-label", "{month_label}" }
            }

            div { class: "agenda-toolbar-right",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    input {
                        r#type: "date",
                        class: "agenda-date-picker-input",
                        value: "{current_date_str}",
                        onchange: move |e| current_date_str.set(e.value()),
                    }

                    select {
                        class: "agenda-dentist-select",
                        style: "min-width: 100px; height: 34px;",
                        value: "{view_mode}",
                        onchange: move |e| view_mode.set(e.value()),
                        option { value: "week", "Semana" }
                        option { value: "day", "Dia" }
                    }
                }

                button {
                    r#type: "button",
                    class: "btn-new-appointment",
                    title: "Novo Agendamento",
                    onclick: move |_| on_open_new.call(()),
                    IconPlus { size: 16, color: "#ffffff".to_string() }
                }
            }
        }
    }
}
