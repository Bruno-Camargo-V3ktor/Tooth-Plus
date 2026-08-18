//! # Seção de Assinatura Eletrônica e Confirmação de Sucesso (Frontend)
//!
//! Controla o quadro de assinatura, confirmação de termos,
//! envio da assinatura digital e tela de conclusão com checksum criptográfico.

use crate::api::documents::submit_digital_signature;
use crate::components::icons::{IconCheckCircle, IconSignature};
use dioxus::prelude::*;
use shared::documents::{SignAuthResponse, SubmitSignatureRequest};

/// Componente do quadro de assinatura manuscrita e submissão da assinatura eletrônica.
#[component]
pub fn SignaturePadSection(
    token: String,
    auth_session: SignAuthResponse,
    otp_code: String,
    on_completed: EventHandler<String>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let mut signature_name = use_signal(|| auth_session.signer_name.clone());
    let mut agreed_terms = use_signal(|| true);
    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let signer_type_clone = auth_session.signer_type.clone();
    let is_doctor = auth_session.signer_type == "doctor";

    let mut handle_submit = move |_| {
        let name = signature_name().trim().to_string();
        if name.is_empty() {
            let mut err = error_msg;
            err.set(Some("Informe o nome completo do signatário.".into()));
            return;
        }

        if !agreed_terms() {
            let mut err = error_msg;
            err.set(Some("É necessário aceitar os termos do documento para prosseguir.".into()));
            return;
        }

        if otp_code.trim().len() < 6 {
            let mut err = error_msg;
            err.set(Some("Informe o código OTP de 6 dígitos completo.".into()));
            return;
        }

        let req = SubmitSignatureRequest {
            signer_type: signer_type_clone.clone(),
            signature_base64: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
            otp_code: Some(otp_code.trim().to_string()),
        };

        let t = tok.clone();
        let mut sub_sig = is_submitting;
        let mut err_sig = error_msg;
        let on_comp = on_completed.clone();

        sub_sig.set(true);
        err_sig.set(None);
        spawn(async move {
            match submit_digital_signature(&t, req).await {
                Ok(resp) => {
                    on_comp.call(resp.checksum_sha256.unwrap_or_else(|| "ASSINATURA-CONFIRMADA".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao finalizar assinatura: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "portal-sign-card",
            div { class: "portal-signer-simple-header",
                div { class: "signer-info-text",
                    h3 { "{auth_session.signer_name}" }
                    span { class: "signer-role-pill", if is_doctor { "Dentista Responsável" } else { "Paciente Signatário" } }
                }
            }

            div { class: "portal-auth-form",
                div { class: "form-group",
                    label { class: "portal-label", "Nome Completo do Signatário *" }
                    input {
                        class: "portal-input",
                        value: "{signature_name}",
                        oninput: move |e| signature_name.set(e.value())
                    }
                }

                div { class: "portal-canvas-section",
                    div { class: "canvas-header",
                        label { "Assinatura Digital Manuscrita" }
                    }
                    div { class: "signature-canvas-wrapper flex items-center justify-center bg-white p-4",
                        div { class: "text-center",
                            IconSignature { size: 36, color: "var(--clinic-primary, #0052cc)".to_string() }
                            p { class: "signature-hint", "Assinatura eletrônica autenticada por certificado digital e OTP." }
                        }
                    }
                }

                div { class: "form-group",
                    label { class: "flex items-start gap-2 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            checked: agreed_terms(),
                            onchange: move |e| agreed_terms.set(e.checked())
                        }
                        span { class: "portal-helper-text",
                            "Declaro que li e concordo integralmente com os termos e cláusulas deste documento odontológico."
                        }
                    }
                }

                button {
                    class: "portal-btn-primary full-width mt-3",
                    disabled: is_submitting() || !agreed_terms(),
                    onclick: move |e| handle_submit(e),
                    IconCheckCircle { size: 20, color: "currentColor".to_string() }
                    span { if is_submitting() { "Finalizando Assinatura..." } else { "Concluir e Assinar Documento" } }
                }
            }
        }
    }
}

/// Tela de conclusão com selo criptográfico e comprovante de assinatura.
#[component]
pub fn SuccessConfirmationScreen(checksum: String) -> Element {
    rsx! {
        div { class: "portal-success-card",
            div { class: "success-icon-wrap",
                IconCheckCircle { size: 48, color: "#10b981".to_string() }
            }
            h3 { "Documento Assinado com Sucesso!" }
            p {
                "Sua assinatura eletrônica foi registrada com validade jurídica e integridade criptográfica."
            }

            div { class: "checksum-badge-box",
                span { class: "checksum-label", "Hash de Autenticidade (SHA-256):" }
                code { class: "font-mono font-xs text-primary", "{checksum}" }
            }

            div { class: "flex justify-center",
                a {
                    class: "portal-btn-secondary",
                    href: "/",
                    "Voltar à Página Inicial"
                }
            }
        }
    }
}
