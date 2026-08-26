use crate::api::documents::DocumentsApi;
use crate::icons::IconWhatsapp;
use shared::documents::RequestOtpRequest;
use dioxus::prelude::*;

#[component]
pub fn OtpSection(
    token: String,
    patient_phone: String,
    on_verified: EventHandler<()>,
) -> Element {
    let mut otp_code = use_signal(String::new);
    let mut is_sending = use_signal(|| false);
    let mut msg_sent = use_signal(|| true);

    let handle_verify = {
        let mut on_v = on_verified;
        let code_sig = otp_code;
        move |_| {
            if code_sig.read().trim().len() >= 4 {
                on_v.call(());
            }
        }
    };

    rsx! {
        div { class: "portal-card",
            div { style: "display: flex; align-items: center; gap: 10px;",
                IconWhatsapp { size: 24, color: "#22c55e".to_string() }
                div {
                    h3 { style: "font-size: 16px; font-weight: 800; color: #ffffff; margin: 0;", "Código de Segurança (OTP)" }
                    p { style: "font-size: 12px; color: #94a3b8; margin: 0;",
                        "Enviamos um código de 6 dígitos via WhatsApp para {patient_phone}."
                    }
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Digite o código recebido *" }
                input {
                    class: "form-input",
                    style: "font-size: 20px; text-align: center; letter-spacing: 8px; font-weight: 800; height: 48px;",
                    r#type: "text",
                    maxlength: "6",
                    placeholder: "• • • • • •",
                    value: "{otp_code}",
                    oninput: move |e| otp_code.set(e.value()),
                }
            }

            div { style: "display: flex; align-items: center; justify-content: space-between;",
                span { style: "font-size: 12px; color: #64748b;", "Não recebeu o código?" }
                button {
                    r#type: "button",
                    style: "background: none; border: none; color: #38bdf8; font-size: 12px; font-weight: 600; cursor: pointer;",
                    onclick: move |_| {
                        let tok = token.clone();
                        spawn(async move {
                            let _ = DocumentsApi::request_otp(&tok, RequestOtpRequest::default()).await;
                        });
                    },
                    "Reenviar via WhatsApp"
                }
            }

            button {
                r#type: "button",
                class: "btn-new-patient-green",
                style: "width: 100%; height: 42px; font-size: 14px; font-weight: 700;",
                onclick: handle_verify,
                "Validar Código e Assinar →"
            }
        }
    }
}
