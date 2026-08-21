//! # Seção de Assinatura Eletrônica e Confirmação de Sucesso (Frontend)
//!
//! Controla o quadro interativo de assinatura manuscrita (Canvas HTML5),
//! confirmação de termos legais, captura de metadados do dispositivo e submissão.

use crate::api::documents::submit_digital_signature;
use crate::components::icons::{IconCheckCircle, IconRefresh, IconSignature};
use dioxus::prelude::*;
use shared::documents::{SignAuthResponse, SubmitSignatureRequest};

/// Componente do quadro de assinatura manuscrita interativo e submissão da assinatura eletrônica.
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
    let mut signature_base64 = use_signal(String::new);
    let mut client_device_info = use_signal(String::new);

    let tok = token.clone();
    let signer_type_clone = auth_session.signer_type.clone();
    let is_doctor = auth_session.signer_type == "doctor";

    // Inicializa o canvas de desenho manuscrito e captura de dados do dispositivo via script
    use_effect(move || {
        spawn(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
            let js_code = r#"
                (function() {
                    const canvas = document.getElementById('signature-drawing-pad');
                    const hiddenInput = document.getElementById('signature-b64-input');
                    const devInput = document.getElementById('device-info-input');

                    if (devInput) {
                        const screenRes = window.screen ? `${window.screen.width}x${window.screen.height}` : 'unknown';
                        const tz = Intl && Intl.DateTimeFormat ? Intl.DateTimeFormat().resolvedOptions().timeZone : 'unknown';
                        const lang = navigator.language || 'pt-BR';
                        const plat = navigator.platform || 'unknown';
                        devInput.value = `Plataforma: ${plat} | Tela: ${screenRes} | Fuso: ${tz} | Idioma: ${lang}`;
                        devInput.dispatchEvent(new Event('input', { bubbles: true }));
                    }

                    if (!canvas) return;

                    const ctx = canvas.getContext('2d');
                    ctx.lineWidth = 2.8;
                    ctx.lineCap = 'round';
                    ctx.lineJoin = 'round';
                    ctx.strokeStyle = '#0f172a';

                    let isDrawing = false;
                    let hasStroke = false;

                    function getPos(e) {
                        const rect = canvas.getBoundingClientRect();
                        const clientX = e.touches && e.touches.length > 0 ? e.touches[0].clientX : e.clientX;
                        const clientY = e.touches && e.touches.length > 0 ? e.touches[0].clientY : e.clientY;
                        const scaleX = canvas.width / rect.width;
                        const scaleY = canvas.height / rect.height;
                        return {
                            x: (clientX - rect.left) * scaleX,
                            y: (clientY - rect.top) * scaleY
                        };
                    }

                    function start(e) {
                        e.preventDefault();
                        isDrawing = true;
                        hasStroke = true;
                        const pos = getPos(e);
                        ctx.beginPath();
                        ctx.moveTo(pos.x, pos.y);
                    }

                    function move(e) {
                        if (!isDrawing) return;
                        e.preventDefault();
                        const pos = getPos(e);
                        ctx.lineTo(pos.x, pos.y);
                        ctx.stroke();
                    }

                    function end(e) {
                        if (isDrawing) {
                            isDrawing = false;
                            if (hiddenInput && hasStroke) {
                                hiddenInput.value = canvas.toDataURL('image/png');
                                hiddenInput.dispatchEvent(new Event('input', { bubbles: true }));
                            }
                        }
                    }

                    // Limpa listeners anteriores para não duplicar
                    canvas.onmousedown = start;
                    canvas.onmousemove = move;
                    canvas.onmouseup = end;
                    canvas.onmouseleave = end;

                    canvas.ontouchstart = start;
                    canvas.ontouchmove = move;
                    canvas.ontouchend = end;

                    window.clearToothSignature = function() {
                        ctx.clearRect(0, 0, canvas.width, canvas.height);
                        hasStroke = false;
                        if (hiddenInput) {
                            hiddenInput.value = '';
                            hiddenInput.dispatchEvent(new Event('input', { bubbles: true }));
                        }
                    };
                })();
            "#;
            let _ = js_sys::eval(js_code);
        });
    });

    let handle_clear = move |_| {
        let _ = js_sys::eval("if (window.clearToothSignature) window.clearToothSignature();");
        signature_base64.set(String::new());
    };

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
            err.set(Some("Informe o código de segurança OTP de 6 dígitos completo.".into()));
            return;
        }

        let b64 = signature_base64().trim().to_string();
        if b64.is_empty() || !b64.starts_with("data:image/png;base64,") {
            let mut err = error_msg;
            err.set(Some("Por favor, desenhe sua assinatura no quadro antes de concluir.".into()));
            return;
        }

        let dev_meta = if client_device_info().trim().is_empty() {
            None
        } else {
            Some(client_device_info().trim().to_string())
        };

        let req = SubmitSignatureRequest {
            signer_type: signer_type_clone.clone(),
            signature_base64: b64,
            otp_code: Some(otp_code.trim().to_string()),
            device_info: dev_meta,
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
            // Hidden inputs para binding bidirecional com o JS do canvas
            input {
                r#type: "hidden",
                id: "signature-b64-input",
                value: "{signature_base64}",
                oninput: move |e| signature_base64.set(e.value())
            }
            input {
                r#type: "hidden",
                id: "device-info-input",
                value: "{client_device_info}",
                oninput: move |e| client_device_info.set(e.value())
            }

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

                // Quadro Interativo de Assinatura Manuscrita
                div { class: "portal-canvas-section",
                    div { class: "canvas-header", style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;",
                        label { class: "portal-label", style: "margin: 0; font-weight: 700;", "Desenhe sua Assinatura no Quadro *" }
                        button {
                            r#type: "button",
                            class: "btn-secondary btn-sm",
                            style: "font-size: 11px; padding: 3px 8px; display: inline-flex; align-items: center; gap: 4px; border-radius: 6px; background: #f1f5f9; color: #475569;",
                            onclick: handle_clear,
                            IconRefresh { size: 12, color: "currentColor".to_string() }
                            span { "Limpar Traço" }
                        }
                    }

                    div {
                        class: "signature-canvas-wrapper",
                        style: "width: 100%; height: 180px; background: #ffffff; border: 2px dashed #94a3b8; border-radius: 12px; position: relative; overflow: hidden; touch-action: none;",
                        canvas {
                            id: "signature-drawing-pad",
                            width: "600",
                            height: "200",
                            style: "width: 100%; height: 100%; display: block; cursor: crosshair; touch-action: none;"
                        }
                    }
                    p { class: "signature-hint", style: "margin-top: 6px; font-size: 11px; color: #64748b; text-align: center;",
                        "✍️ Desenhe usando o dedo (no celular/tablet) ou o mouse (no computador)."
                    }
                }

                div { class: "form-group mt-3",
                    label { class: "flex items-start gap-2 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            checked: agreed_terms(),
                            onchange: move |e| agreed_terms.set(e.checked())
                        }
                        span { class: "portal-helper-text", style: "font-size: 12px; color: #475569; line-height: 1.4;",
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
