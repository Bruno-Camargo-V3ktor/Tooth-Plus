use crate::components::icons::{IconPlus, IconSearch};
use dioxus::prelude::*;

#[component]
pub fn PageHeader(
    search_query: Signal<String>,
    show_new_btn: bool,
    btn_text: String,
    on_new: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "page-action-header compact-toolbar",
            div { class: "header-actions-group full-width-toolbar",
                div { class: "modern-search-bar",
                    div { class: "search-icon", IconSearch { size: 18, color: "currentColor".to_string() } }
                    input {
                        class: "search-input",
                        placeholder: "Buscar...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                }
                if show_new_btn {
                    button {
                        class: "btn-primary",
                        onclick: move |_| on_new.call(()),
                        IconPlus { size: 18, color: "currentColor".to_string() }
                        "{btn_text}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ActionModal(
    is_open: bool,
    title: String,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    if !is_open {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "action-modal modal-large",
                onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "{title}" }
                    }
                    button { class: "modal-close", onclick: move |_| on_close.call(()), "×" }
                }
                div { class: "modal-body scrollable",
                    {children}
                }
            }
        }
    }
}
