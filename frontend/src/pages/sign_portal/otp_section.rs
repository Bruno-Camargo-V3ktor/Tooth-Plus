//! # Seção de Validação de Código OTP (Frontend)
//!
//! Controla o disparo de código de uso único (OTP de 6 dígitos) para o telefone/email
//! do signatário autenticado (paciente ou dentista) e validação em 2 etapas.

use crate::api::documents::request_signing_otp;
use crate::components::icons::{IconCheckCircle, IconMail, IconPhone, IconRefresh, IconShieldCheck};
use dioxus::prelude::*;
use shared::documents::{PublicSigningDocumentResponse, SignAuthResponse};

/// Componente de solicitação e validação de OTP para assinatura eletrônica.
#[component]
pub fn OtpVerificationSection(
    token: String,
    doc_info: PublicSigningDocumentResponse,
    auth_session: SignAuthResponse,
    otp_verified: Signal<bool>,
    otp_code: Signal<String>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let mut otp_channel = use_signal(|| "whatsapp".to_string());
    let mut is_sending = use_signal(|| false);
    let mut otp_sent = use_signal(|| false);

    let tok = token.clone();
    let is_doctor = auth_session.signer_type == "doctor";
    let signer_type_clone = auth_session.signer_type.clone();

    let mut handle_request_otp = move |_| {
        let t = tok.clone();
        let s_type = signer_type_clone.clone();
        let mut send_sig = is_sending;
        let mut sent_sig = otp_sent;
        let mut err_sig = error_msg;
        let mut succ_sig = success_msg;

        send_sig.set(true);
        err_sig.set(None);
        spawn(async move {
            match request_signing_otp(&t, &otp_channel(), Some(&s_type)).await {
                Ok(msg) => {
                    sent_sig.set(true);
                    succ_sig.set(Some(msg));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao enviar código OTP: {}", e)));
                }
            }
            send_sig.set(false);
        });
    };

    let phone_opt = if is_doctor {
        doc_info.doctor_phone_masked.as_ref()
    } else {
        Some(&doc_info.patient_phone_masked)
    };

    let email_opt = if is_doctor {
        doc_info.doctor_email_masked.as_ref()
    } else {
        doc_info.patient_email_masked.as_ref()
    };

    rsx! {
        div { class: "portal-sign-card",
            div { class: "portal-signer-simple-header",
                div { class: "signer-info-text",
                    h3 { "{auth_session.signer_name}" }
                    span { class: "signer-role-pill", if is_doctor { "Dentista Responsável" } else { "Paciente Signatário" } }
                }
            }

            div { class: "portal-otp-section",
                div { class: "portal-otp-header",
                    IconShieldCheck { size: 20, color: "var(--clinic-primary, #0052cc)".to_string() }
                    h4 { "Verificação em Duas Etapas (OTP)" }
                }
                p { class: "portal-otp-sub", "Para garantir a autenticidade e validade jurídica da sua assinatura, enviaremos um código de uso único." }

                div { class: "portal-otp-channel-picker",
                    if let Some(phone) = phone_opt {
                        button {
                            r#type: "button",
                            class: if otp_channel() == "whatsapp" { "otp-channel-btn active" } else { "otp-channel-btn" },
                            onclick: move |_| otp_channel.set("whatsapp".to_string()),
                            title: "Enviar via WhatsApp / SMS: {phone}",
                            span { class: "otp-channel-icon",
                                IconPhone { size: 18, color: "currentColor".to_string() }
                            }
                            div { class: "otp-channel-text",
                                span { class: "otp-channel-name", "WhatsApp / SMS" }
                                span { class: "otp-channel-val", "{phone}" }
                            }
                        }
                    }
                    if let Some(email) = email_opt {
                        button {
                            r#type: "button",
                            class: if otp_channel() == "email" { "otp-channel-btn active" } else { "otp-channel-btn" },
                            onclick: move |_| otp_channel.set("email".to_string()),
                            title: "Enviar via E-mail: {email}",
                            span { class: "otp-channel-icon",
                                IconMail { size: 18, color: "currentColor".to_string() }
                            }
                            div { class: "otp-channel-text",
                                span { class: "otp-channel-name", "E-mail Seguro" }
                                span { class: "otp-channel-val", "{email}" }
                            }
                        }
                    }
                }

                div { class: "portal-otp-row",
                    input {
                        class: "otp-field",
                        placeholder: "000000",
                        maxlength: "6",
                        value: "{otp_code}",
                        oninput: move |e| otp_code.set(e.value())
                    }
                    button {
                        class: "portal-btn-otp-action",
                        disabled: is_sending(),
                        onclick: move |e| handle_request_otp(e),
                        if is_sending() {
                            IconRefresh { size: 14, color: "currentColor".to_string() }
                            span { "Enviando..." }
                        } else if otp_sent() {
                            IconRefresh { size: 14, color: "currentColor".to_string() }
                            span { "Reenviar" }
                        } else {
                            span { "Receber Código" }
                        }
                    }
                }
            }

            button {
                class: "portal-btn-primary full-width",
                disabled: otp_code().trim().len() < 6,
                onclick: move |_| {
                    let code = otp_code().trim().to_string();
                    if code.len() == 6 {
                        let mut v = otp_verified;
                        v.set(true);
                    }
                },
                IconCheckCircle { size: 18, color: "currentColor".to_string() }
                span { "Validar Código e Prosseguir" }
            }
        }
    }
}
