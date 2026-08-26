pub mod auth_card;
pub mod otp_section;
pub mod pdf_viewer;
pub mod signature_section;

use auth_card::AuthCard;
use otp_section::OtpSection;
use pdf_viewer::DocumentViewerCard;
use signature_section::SignaturePadSection;

use crate::api::documents::DocumentsApi;
use crate::icons::{IconCheck, IconTooth};
use shared::documents::{PublicSigningDocumentResponse, SignAuthResponse};
use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/sign_portal/style.css");

#[component]
pub fn SignPortal(token: String) -> Element {
    let signing_token = token.clone();

    let mut doc_data = use_signal(|| None::<PublicSigningDocumentResponse>);
    let mut auth_session = use_signal(|| None::<SignAuthResponse>);
    let mut otp_verified = use_signal(|| false);
    let mut completed_checksum = use_signal(|| None::<String>);
    let mut load_error = use_signal(|| None::<String>);

    let tok_eff = signing_token.clone();
    use_effect(move || {
        let tok = tok_eff.clone();
        spawn(async move {
            match DocumentsApi::get_public_document(&tok).await {
                Ok(data) => doc_data.set(Some(data)),
                Err(err) => load_error.set(Some(err)),
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "sign-portal-page",
            div { class: "portal-container",
                // Top Header Oficial
                div { class: "portal-header",
                    div { class: "portal-brand",
                        div { style: "width: 40px; height: 40px; border-radius: 10px; background: #00a0e4; display: flex; align-items: center; justify-content: center;",
                            IconTooth { size: 24, color: "#ffffff".to_string() }
                        }
                        div {
                            h1 { class: "portal-clinic-name",
                                if let Some(ref d) = *doc_data.read() {
                                    "{d.clinic_name}"
                                } else {
                                    "Tooth Plus — Portal de Assinatura"
                                }
                            }
                            p { class: "portal-clinic-sub", "Assinatura Digital de Documentos e Termos Odontológicos" }
                        }
                    }

                    div { class: "portal-security-badge",
                        span { "🔒 Ambiente Seguro LGPD & SHA-256" }
                    }
                }

                if let Some(ref checksum) = *completed_checksum.read() {
                    // TELA DE SUCESSO
                    div { class: "portal-success-card",
                        div { class: "portal-success-icon", "✓" }
                        h2 { style: "font-size: 20px; font-weight: 800; color: #22c55e; margin: 0;", "Documento Assinado com Sucesso!" }
                        p { style: "font-size: 13.5px; color: #94a3b8; max-width: 480px; margin: 0;",
                            "Sua assinatura digital foi gravada e vinculada ao prontuário eletrônico com validade jurídica."
                        }

                        div { style: "background: #0b1120; border: 1px solid rgba(255,255,255,0.08); padding: 12px 16px; border-radius: 8px; font-family: monospace; font-size: 11.5px; color: #38bdf8; word-break: break-all; margin-top: 8px;",
                            "HASH SHA-256: {checksum}"
                        }
                    }
                } else if let Some(ref doc_resp) = *doc_data.read() {
                    div { class: "portal-content-grid",
                        // Coluna da Esquerda: Pré-visualização do Documento
                        DocumentViewerCard {
                            doc: doc_resp.document.clone(),
                            clinic_name: doc_resp.clinic_name.clone(),
                        }

                        // Coluna da Direita: Fluxo de Assinatura
                        div { style: "display: flex; flex-direction: column; gap: 14px;",
                            if auth_session.read().is_none() {
                                AuthCard {
                                    token: signing_token.clone(),
                                    on_authenticated: move |auth| auth_session.set(Some(auth)),
                                }
                            } else if !otp_verified() {
                                OtpSection {
                                    token: signing_token.clone(),
                                    patient_phone: doc_resp.patient_phone_masked.clone(),
                                    on_verified: move |_| otp_verified.set(true),
                                }
                            } else {
                                SignaturePadSection {
                                    token: signing_token.clone(),
                                    signer_name: auth_session.read().as_ref().map(|s| s.signer_name.clone()).unwrap_or_else(|| "Paciente".to_string()),
                                    on_completed: move |chk| completed_checksum.set(Some(chk)),
                                }
                            }
                        }
                    }
                } else if let Some(ref err) = *load_error.read() {
                    div { class: "empty-debits-box",
                        h3 { class: "empty-debits-title", style: "color: #f87171;", "Documento Indisponível" }
                        p { class: "empty-debits-desc", "{err}" }
                    }
                } else {
                    div { class: "empty-debits-box",
                        p { class: "empty-debits-desc", "Carregando documento seguro..." }
                    }
                }
            }
        }
    }
}
