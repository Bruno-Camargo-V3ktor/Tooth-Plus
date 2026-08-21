//! # Módulo do Portal Público de Assinatura Digital (Frontend)
//!
//! Portal responsivo para pacientes e dentistas autenticarem, receberem código OTP
//! e assinarem termos de consentimento e contratos com validade jurídica.

pub mod auth_card;
pub mod otp_section;
pub mod pdf_viewer;
pub mod signature_section;

pub use auth_card::*;
pub use otp_section::*;
pub use pdf_viewer::*;
pub use signature_section::*;

use crate::api::documents::fetch_public_signing_document;
use crate::components::icons::{IconLock, IconShieldCheck, IconTooth};
use dioxus::prelude::*;
use shared::documents::SignAuthResponse;

/// Componente principal da página do Portal Público de Assinatura Eletrônica.
#[component]
pub fn SignPortal(token: String) -> Element {
    let signing_token = token.clone();

    let doc_res = use_resource(move || {
        let t = signing_token.clone();
        async move { fetch_public_signing_document(&t).await }
    });

    let auth_session = use_signal(|| None::<SignAuthResponse>);
    let otp_verified = use_signal(|| false);
    let otp_code = use_signal(String::new);
    let mut is_completed = use_signal(|| false);
    let mut completed_checksum = use_signal(String::new);

    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);

    rsx! {
        div { class: "sign-portal-wrapper",
            div { class: "portal-container",
                match &*doc_res.read() {
                    Some(Ok(doc)) => rsx! {
                        div { class: "portal-header",
                            div { class: "portal-brand",
                                div { class: "portal-clinic-icon-box bg-primary",
                                    IconTooth { size: 24, color: "#ffffff".to_string() }
                                }
                                div { class: "portal-clinic-info",
                                    h1 { class: "portal-clinic-name", "{doc.clinic_name}" }
                                    span { class: "portal-clinic-sub", "Portal Seguro de Assinatura Eletrônica" }
                                }
                            }
                            div { class: "portal-security-badge",
                                IconShieldCheck { size: 16, color: "currentColor".to_string() }
                                span { "Ambiente Seguro LGPD & Criptografia SHA-256" }
                            }
                        }

                        if let Some(ref err) = *error_msg.read() {
                            div { class: "portal-toast-error",
                                span { "{err}" }
                            }
                        }

                        if let Some(ref succ) = *success_msg.read() {
                            div { class: "portal-toast-success",
                                span { "{succ}" }
                            }
                        }

                        if is_completed() {
                            SuccessConfirmationScreen { checksum: completed_checksum() }
                        } else {
                            div { class: "portal-content-grid",
                                div { class: "portal-pdf-column",
                                    DocumentViewerCard { doc: doc.clone() }
                                }

                                div { class: "portal-action-column",
                                    if let Some(ref session) = *auth_session.read() {
                                        if !otp_verified() {
                                            OtpVerificationSection {
                                                token: token.clone(),
                                                doc_info: doc.clone(),
                                                auth_session: session.clone(),
                                                otp_verified,
                                                otp_code,
                                                error_msg,
                                                success_msg,
                                            }
                                        } else {
                                            SignaturePadSection {
                                                token: token.clone(),
                                                auth_session: session.clone(),
                                                otp_code: otp_code(),
                                                on_completed: move |checksum: String| {
                                                    completed_checksum.set(checksum);
                                                    is_completed.set(true);
                                                },
                                                error_msg,
                                                success_msg,
                                            }
                                        }
                                    } else {
                                        SignerAuthCard {
                                            token: token.clone(),
                                            doc_info: Some(doc.clone()),
                                            auth_session,
                                            error_msg,
                                            success_msg,
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "portal-already-signed-card text-center p-8",
                            IconLock { size: 48, color: "#ef4444".to_string() }
                            h3 { class: "mt-3 text-danger", "Link de Assinatura Inválido ou Expirado" }
                            p { class: "text-muted font-xs mt-2", "{e}" }
                        }
                    },
                    None => rsx! {
                        div { class: "portal-already-signed-card text-center p-8",
                            p { class: "loading-text", "Carregando documento e chaves criptográficas..." }
                        }
                    },
                }
            }
        }
    }
}
