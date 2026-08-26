use crate::icons::IconFileText;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct DocTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub icon: &'static str,
    pub category: &'static str,
}

#[component]
pub fn TemplateGrid(
    templates: Vec<DocTemplate>,
    on_select: EventHandler<&'static str>,
) -> Element {
    rsx! {
        div { class: "doc-templates-grid",
            for doc in templates {
                div {
                    key: "{doc.id}",
                    class: "doc-template-card",
                    div { class: "doc-card-header",
                        div { class: "doc-icon-badge", "{doc.icon}" }
                        div {
                            h3 { class: "doc-card-title", "{doc.title}" }
                            span { style: "font-size: 11.5px; color: #38bdf8; font-weight: 600;", "{doc.category}" }
                        }
                    }
                    p { class: "doc-card-desc", "{doc.desc}" }
                    div { class: "doc-card-footer",
                        button {
                            r#type: "button",
                            class: "btn-emit-doc",
                            onclick: move |_| on_select.call(doc.id),
                            IconFileText { size: 14, color: "#ffffff".to_string() }
                            span { "Preencher & Imprimir" }
                        }
                    }
                }
            }
        }
    }
}
