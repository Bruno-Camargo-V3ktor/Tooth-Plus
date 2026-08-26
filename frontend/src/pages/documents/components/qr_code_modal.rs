use crate::components::modal::Modal;
use crate::icons::IconCopy;
use dioxus::prelude::*;

#[component]
pub fn QrCodeModal(
    is_open: bool,
    document_title: String,
    signing_token: String,
    on_close: EventHandler<()>,
    on_copied: EventHandler<()>,
) -> Element {
    if !is_open {
        return rsx! {};
    }

    let signing_url = format!("https://app.toothplus.com.br/sign/{}", signing_token);

    rsx! {
        Modal {
            title: "Assinatura Digital via QR Code".to_string(),
            is_open,
            on_close: move |_| on_close.call(()),
            footer: rsx! {
                button {
                    r#type: "button",
                    class: "btn-modal-primary",
                    style: "width: 100%;",
                    onclick: move |_| on_close.call(()),
                    "Concluir"
                }
            },

            div { style: "display: flex; flex-direction: column; align-items: center; text-align: center; gap: 16px;",
                p { style: "font-size: 13.5px; color: #94a3b8; margin: 0;",
                    "Aponte a câmera do celular ou tablet do paciente para assinar este documento digitalmente:"
                }

                // QR Code Container Simulado
                div { style: "background: #ffffff; padding: 16px; border-radius: 12px; display: inline-flex; align-items: center; justify-content: center; box-shadow: 0 4px 20px rgba(0,0,0,0.4);",
                    svg {
                        width: "180",
                        height: "180",
                        view_box: "0 0 100 100",
                        rect { x: "5", y: "5", width: "30", height: "30", fill: "#0c1222" }
                        rect { x: "10", y: "10", width: "20", height: "20", fill: "#ffffff" }
                        rect { x: "15", y: "15", width: "10", height: "10", fill: "#0c1222" }

                        rect { x: "65", y: "5", width: "30", height: "30", fill: "#0c1222" }
                        rect { x: "70", y: "10", width: "20", height: "20", fill: "#ffffff" }
                        rect { x: "75", y: "15", width: "10", height: "10", fill: "#0c1222" }

                        rect { x: "5", y: "65", width: "30", height: "30", fill: "#0c1222" }
                        rect { x: "10", y: "70", width: "20", height: "20", fill: "#ffffff" }
                        rect { x: "15", y: "75", width: "10", height: "10", fill: "#0c1222" }

                        rect { x: "45", y: "15", width: "10", height: "30", fill: "#0c1222" }
                        rect { x: "15", y: "45", width: "30", height: "10", fill: "#0c1222" }
                        rect { x: "45", y: "45", width: "15", height: "15", fill: "#00a0e4" }
                        rect { x: "65", y: "65", width: "20", height: "20", fill: "#0c1222" }
                    }
                }

                strong { style: "font-size: 14px; color: #f1f5f9;", "{document_title}" }

                div { style: "display: flex; align-items: center; gap: 8px; width: 100%; max-width: 440px;",
                    input {
                        class: "form-input",
                        style: "font-size: 11.5px; font-family: monospace; flex: 1;",
                        readonly: true,
                        value: "{signing_url}",
                    }
                    button {
                        r#type: "button",
                        class: "btn-secondary",
                        style: "font-weight: 700; font-size: 12px;",
                        onclick: move |_| on_copied.call(()),
                        IconCopy { size: 14, color: "#38bdf8".to_string() }
                        span { "Copiar" }
                    }
                }
            }
        }
    }
}
