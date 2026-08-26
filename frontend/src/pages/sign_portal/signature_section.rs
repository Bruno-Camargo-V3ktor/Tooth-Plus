use crate::api::documents::DocumentsApi;
use crate::icons::IconCheck;
use shared::documents::SubmitSignatureRequest;
use dioxus::prelude::*;

#[component]
pub fn SignaturePadSection(
    token: String,
    signer_name: String,
    on_completed: EventHandler<String>,
) -> Element {
    let mut has_drawn = use_signal(|| false);
    let mut agreed_terms = use_signal(|| true);
    let mut is_submitting = use_signal(|| false);

    let handle_submit = {
        let tok = token.clone();
        let s_name = signer_name.clone();
        let mut load_sig = is_submitting;
        let mut on_c = on_completed;
        let agree_sig = agreed_terms;

        move |_| {
            if !*agree_sig.read() {
                return;
            }

            load_sig.set(true);
            let tok_clone = tok.clone();
            let mut on_c_clone = on_c;
            let mut load_c = load_sig;

            spawn(async move {
                let req = SubmitSignatureRequest {
                    signature_base64: "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik0xMCAxMCBMMTAwIDEwMCIgc3Ryb2tlPSIjMDAwIi8+PC9zdmc+".to_string(),
                    signer_type: "patient".to_string(),
                    otp_code: Some("123456".to_string()),
                    device_info: Some("Web Browser Digital Pad".to_string()),
                };
                match DocumentsApi::submit_signature(&tok_clone, req).await {
                    Ok(checksum) => {
                        on_c_clone.call(checksum);
                    }
                    Err(_) => {
                        load_c.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "portal-card",
            div { style: "display: flex; align-items: center; justify-content: space-between;",
                div {
                    h3 { style: "font-size: 16px; font-weight: 800; color: #ffffff; margin: 0;", "Desenhe sua Assinatura" }
                    p { style: "font-size: 12px; color: #94a3b8; margin: 0;", "Signatário: {signer_name}" }
                }
                button {
                    r#type: "button",
                    style: "background: none; border: none; color: #f87171; font-size: 12px; font-weight: 600; cursor: pointer;",
                    onclick: move |_| has_drawn.set(false),
                    "Limpar"
                }
            }

            div {
                class: "signature-canvas-wrap",
                onclick: move |_| has_drawn.set(true),
                ontouchstart: move |_| has_drawn.set(true),

                if !has_drawn() {
                    span { class: "signature-hint", "✍️ Assine ou rubrique com o dedo ou mouse aqui" }
                } else {
                    svg {
                        width: "280",
                        height: "120",
                        view_box: "0 0 280 120",
                        path {
                            d: "M 20 80 Q 60 20, 100 60 T 180 50 T 260 70",
                            fill: "none",
                            stroke: "#0c1222",
                            stroke_width: "3",
                            stroke_linecap: "round",
                        }
                    }
                }
            }

            label { style: "display: flex; align-items: flex-start; gap: 8px; font-size: 12.5px; color: #cbd5e1; cursor: pointer;",
                input {
                    r#type: "checkbox",
                    checked: "{agreed_terms}",
                    onchange: move |e| agreed_terms.set(e.checked()),
                }
                span { "Declaro que li e concordo integralmente com os termos deste documento e autorizo o registro desta assinatura eletrônica." }
            }

            button {
                r#type: "button",
                class: "btn-new-patient-green",
                style: "width: 100%; height: 44px; font-size: 14.5px; font-weight: 700;",
                disabled: is_submitting() || !*agreed_terms.read(),
                onclick: handle_submit,
                IconCheck { size: 18, color: "#ffffff".to_string() }
                span {
                    if is_submitting() { "Gravando Assinatura..." } else { "Concluir e Assinar Documento" }
                }
            }
        }
    }
}
