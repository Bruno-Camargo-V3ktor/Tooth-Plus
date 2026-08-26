//! # Componente Base de Modal
//!
//! Fornece a estrutura de backdrop com blur, container flutuante, cabeçalho padronizado,
//! corpo com scroll interno e rodapé de ações.

use crate::icons::{IconClose, IconTooth};
use dioxus::prelude::*;

#[component]
pub fn Modal(
    title: String,
    subtitle: Option<String>,
    #[props(default = true)] is_open: bool,
    on_close: EventHandler<()>,
    children: Element,
    footer: Option<Element>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "modal-card",
                onclick: move |e| e.stop_propagation(),

                // Cabeçalho do Modal
                div { class: "modal-header",
                    div { class: "modal-header-left",
                        div { class: "modal-header-icon-box",
                            IconTooth { size: 20, color: "#0284c7".to_string() }
                        }
                        div { class: "modal-header-text-col",
                            h2 { class: "modal-title", "{title}" }
                            if let Some(sub) = subtitle {
                                p { class: "modal-subtitle", "{sub}" }
                            }
                        }
                    }

                    button {
                        r#type: "button",
                        class: "modal-close-btn",
                        title: "Fechar modal",
                        onclick: move |_| on_close.call(()),
                        IconClose { size: 18, color: "currentColor".to_string() }
                    }
                }

                // Corpo com Scroll Suave
                div { class: "modal-body",
                    {children}
                }

                // Rodapé de Ações (Opcional)
                if let Some(footer_elem) = footer {
                    div { class: "modal-footer",
                        {footer_elem}
                    }
                }
            }
        }
    }
}
