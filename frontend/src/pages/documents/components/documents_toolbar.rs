use crate::icons::{IconPlus, IconSearch};
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Copy)]
pub enum DocumentTab {
    Issued,
    Templates,
}

#[component]
pub fn DocumentsToolbar(
    active_tab: Signal<DocumentTab>,
    search_query: Signal<String>,
    status_filter: Signal<String>,
    on_issue_document: EventHandler<()>,
    on_new_template: EventHandler<()>,
) -> Element {
    let current_tab = *active_tab.read();
    let current_status = status_filter.read().clone();

    rsx! {
        div { class: "stock-toolbar-row",
            div { class: "tab-underline-bar", style: "margin: 0; padding: 0;",
                button {
                    class: if current_tab == DocumentTab::Issued { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                    onclick: move |_| active_tab.set(DocumentTab::Issued),
                    "Documentos Emitidos"
                }
                button {
                    class: if current_tab == DocumentTab::Templates { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                    onclick: move |_| active_tab.set(DocumentTab::Templates),
                    "Modelos & Templates"
                }
            }

            div { class: "stock-search-box",
                IconSearch { size: 16, color: "#64748b".to_string() }
                input {
                    r#type: "text",
                    placeholder: if current_tab == DocumentTab::Issued { "Buscar por paciente, documento ou dentista..." } else { "Buscar modelos cadastrados..." },
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value()),
                }
            }

            if current_tab == DocumentTab::Issued {
                div { class: "stock-category-filters",
                    button {
                        class: if current_status == "ALL" { "btn-filter-pill active" } else { "btn-filter-pill" },
                        onclick: move |_| status_filter.set("ALL".to_string()),
                        "Todos"
                    }
                    button {
                        class: if current_status == "pending" { "btn-filter-pill active" } else { "btn-filter-pill" },
                        onclick: move |_| status_filter.set("pending".to_string()),
                        "Pendentes"
                    }
                    button {
                        class: if current_status == "signed" { "btn-filter-pill active" } else { "btn-filter-pill" },
                        onclick: move |_| status_filter.set("signed".to_string()),
                        "Assinados"
                    }
                }
            }

            div { style: "display: flex; align-items: center; gap: 8px;",
                if current_tab == DocumentTab::Templates {
                    button {
                        r#type: "button",
                        class: "btn-secondary",
                        style: "font-weight: 700;",
                        onclick: move |_| on_new_template.call(()),
                        IconPlus { size: 15, color: "#ffffff".to_string() }
                        span { "NOVO MODELO" }
                    }
                }

                button {
                    r#type: "button",
                    class: "btn-new-patient-green",
                    onclick: move |_| on_issue_document.call(()),
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "EMITIR DOCUMENTO" }
                }
            }
        }
    }
}
