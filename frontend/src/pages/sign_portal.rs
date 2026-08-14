use crate::api::{
    auth_doctor_signing, auth_patient_signing, fetch_public_signing_document, request_signing_otp,
    submit_digital_signature,
};
use crate::components::icons::{
    IconCheckCircle, IconDownload, IconShieldCheck, IconSignature, IconTooth,
};
use dioxus::prelude::*;
use shared::documents::{
    DoctorSignAuthRequest, PatientSignAuthRequest, PublicSigningDocumentResponse, SignAuthResponse,
    SubmitSignatureRequest,
};

#[component]
pub fn SignPortal(token: String) -> Element {
    let mut doc_info = use_signal(|| None::<PublicSigningDocumentResponse>);
    let mut is_loading = use_signal(|| true);
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);

    let mut auth_mode = use_signal(|| "patient".to_string());
    let mut auth_session = use_signal(|| None::<SignAuthResponse>);

    let mut cpf_input = use_signal(String::new);
    let mut patient_pwd = use_signal(String::new);
    let mut doctor_user = use_signal(String::new);
    let mut doctor_pwd = use_signal(String::new);

    let mut otp_input = use_signal(String::new);
    let mut otp_sent = use_signal(|| false);
    let mut is_sending_otp = use_signal(|| false);

    let mut signature_data = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut is_completed = use_signal(|| false);

    let token_clone = token.clone();

    use_effect(move || {
        let t = token_clone.clone();
        spawn(async move {
            is_loading.set(true);
            match fetch_public_signing_document(&t).await {
                Ok(data) => {
                    if data.document.status == "signed" || data.document.status == "completed" {
                        is_completed.set(true);
                    }
                    doc_info.set(Some(data));
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
            is_loading.set(false);
        });
    });

    let current_token = token.clone();

    let on_login_patient = move |_| {
        let t = current_token.clone();
        let cpf = cpf_input();
        let pwd = patient_pwd();

        if cpf.trim().is_empty() {
            error_msg.set(Some("Por favor, informe seu CPF.".into()));
            return;
        }

        spawn(async move {
            error_msg.set(None);
            match auth_patient_signing(&t, PatientSignAuthRequest { cpf, password: pwd }).await {
                Ok(auth) => {
                    auth_session.set(Some(auth));
                    success_msg.set(Some("Identificação confirmada com sucesso.".into()));
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
        });
    };

    let token_doc = token.clone();
    let on_login_doctor = move |_| {
        let t = token_doc.clone();
        let username = doctor_user();
        let pwd = doctor_pwd();

        if username.trim().is_empty() || pwd.trim().is_empty() {
            error_msg.set(Some("Por favor, informe usuário e senha.".into()));
            return;
        }

        spawn(async move {
            error_msg.set(None);
            match auth_doctor_signing(&t, DoctorSignAuthRequest { username, password: pwd }).await {
                Ok(auth) => {
                    auth_session.set(Some(auth));
                    success_msg.set(Some("Identificação médica confirmada com sucesso.".into()));
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
        });
    };

    let token_otp = token.clone();
    let on_request_otp = move |_| {
        let t = token_otp.clone();
        spawn(async move {
            is_sending_otp.set(true);
            error_msg.set(None);
            match request_signing_otp(&t).await {
                Ok(msg) => {
                    otp_sent.set(true);
                    success_msg.set(Some(msg));
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
            is_sending_otp.set(false);
        });
    };

    let token_sub = token.clone();
    let on_submit_signature = move |_| {
        let t = token_sub.clone();
        let sig = signature_data();
        let sess = auth_session();

        let Some(s) = sess else {
            error_msg.set(Some("Por favor, faça login antes de assinar.".into()));
            return;
        };

        if sig.trim().is_empty() {
            error_msg.set(Some("Por favor, desenhe sua assinatura no quadro abaixo.".into()));
            return;
        }

        let otp = if otp_input().trim().is_empty() {
            None
        } else {
            Some(otp_input().trim().to_string())
        };

        spawn(async move {
            is_submitting.set(true);
            error_msg.set(None);

            let req = SubmitSignatureRequest {
                signature_base64: sig,
                signer_type: s.signer_type,
                otp_code: otp,
            };

            match submit_digital_signature(&t, req).await {
                Ok(doc) => {
                    is_completed.set(doc.status == "signed" || doc.status == "completed");
                    success_msg.set(Some("Assinatura digital registrada e protegida com sucesso!".into()));
                    if let Some(ref mut d) = *doc_info.write() {
                        d.document = doc;
                    }
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
            is_submitting.set(false);
        });
    };

    rsx! {
        div { class: "portal-container",
            header { class: "portal-header",
                div { class: "portal-header-brand",
                    div { class: "portal-logo-icon",
                        IconTooth { size: 28, color: "#ffffff".to_string() }
                    }
                    div {
                        h1 { class: "portal-title",
                            if let Some(ref d) = doc_info() {
                                "{d.clinic_name}"
                            } else {
                                "Tooth Plus - Portal de Assinatura Digital"
                            }
                        }
                        span { class: "portal-subtitle", "Ambiente Seguro de Assinatura e Autenticidade" }
                    }
                }
                div { class: "portal-security-badge",
                    IconShieldCheck { size: 18, color: "#10b981".to_string() }
                    span { "Criptografia SHA-256 e Blind Index" }
                }
            }

            main { class: "portal-body",
                if is_loading() {
                    div { class: "portal-loading-card",
                        div { class: "loading-spinner" }
                        p { "Carregando documento e certificação digital..." }
                    }
                } else if let Some(ref doc) = doc_info() {
                    div { class: "portal-grid",
                        // Left Column: Document Details & PDF Viewer
                        div { class: "portal-doc-panel",
                            div { class: "portal-panel-header",
                                div {
                                    span { class: "portal-doc-badge", "{doc.document.document_type.to_uppercase()}" }
                                    h2 { class: "portal-doc-title", "{doc.document.title}" }
                                    p { class: "portal-doc-meta", "Emitido em {doc.document.created_at.chars().take(10).collect::<String>()}" }
                                }
                                div { class: "portal-doc-status-badge",
                                    if doc.document.status == "signed" || is_completed() {
                                        span { class: "badge-status-completed",
                                            IconCheckCircle { size: 16, color: "#10b981".to_string() }
                                            " Totalmente Assinado"
                                        }
                                    } else {
                                        span { class: "badge-status-pending",
                                            IconSignature { size: 16, color: "#f59e0b".to_string() }
                                            " Aguardando Assinaturas"
                                        }
                                    }
                                }
                            }

                            div { class: "portal-preview-frame",
                                iframe {
                                    src: "{doc.document.original_pdf_url}",
                                    title: "Visualizador de Contrato",
                                    class: "portal-pdf-embed",
                                }
                            }

                            // Signatures Status Box
                            div { class: "portal-signers-status",
                                h3 { class: "portal-signers-title", "Status dos Signatários" }
                                div { class: "portal-signers-grid",
                                    div { class: if doc.document.patient_signed_at.is_some() { "signer-card signed" } else { "signer-card pending" },
                                        div { class: "signer-card-header",
                                            span { class: "signer-role", "Paciente" }
                                            if doc.document.patient_signed_at.is_some() {
                                                span { class: "signer-badge-ok", "Assinado" }
                                            } else {
                                                span { class: "signer-badge-wait", "Pendente" }
                                            }
                                        }
                                        p { class: "signer-name", "{doc.patient_phone_masked}" }
                                        if let Some(ref d) = doc.document.patient_signed_at {
                                            p { class: "signer-time", "Data: {d.chars().take(19).collect::<String>()}" }
                                        }
                                    }

                                    div { class: if doc.document.doctor_signed_at.is_some() { "signer-card signed" } else { "signer-card pending" },
                                        div { class: "signer-card-header",
                                            span { class: "signer-role", "Cirurgião-Dentista / Responsável" }
                                            if doc.document.doctor_signed_at.is_some() {
                                                span { class: "signer-badge-ok", "Assinado" }
                                            } else {
                                                span { class: "signer-badge-wait", "Pendente" }
                                            }
                                        }
                                        p { class: "signer-name", "Corpo Clínico" }
                                        if let Some(ref d) = doc.document.doctor_signed_at {
                                            p { class: "signer-time", "Data: {d.chars().take(19).collect::<String>()}" }
                                        }
                                    }
                                }
                            }

                            if let Some(ref checksum) = doc.document.checksum_sha256 {
                                div { class: "portal-checksum-card",
                                    IconShieldCheck { size: 24, color: "#10b981".to_string() }
                                    div {
                                        h4 { "Certificado de Integridade Criptográfica (SHA-256)" }
                                        code { class: "checksum-code", "{checksum}" }
                                    }
                                }
                            }
                        }

                        // Right Column: Interactive Signing Workflow
                        div { class: "portal-action-panel",
                            if is_completed() {
                                div { class: "portal-success-box",
                                    div { class: "success-icon-wrap",
                                        IconCheckCircle { size: 48, color: "#10b981".to_string() }
                                    }
                                    h3 { "Documento 100% Concluído e Válido!" }
                                    p { "Todas as partes assinaram este documento. O arquivo original e as assinaturas foram criptografados com integridade inviolável." }

                                    div { class: "portal-success-actions",
                                        a {
                                            href: "{doc.document.original_pdf_url}",
                                            target: "_blank",
                                            class: "portal-btn-primary",
                                            IconDownload { size: 18, color: "#ffffff".to_string() }
                                            " Baixar Cópia do Contrato"
                                        }
                                    }
                                }
                            } else if auth_session().is_none() {
                                // Step 1: Signer Authentication
                                div { class: "portal-auth-card",
                                    h3 { class: "portal-auth-title", "Identificação para Assinatura" }
                                    p { class: "portal-auth-desc", "Selecione seu perfil para acessar o painel de assinatura digital." }

                                    div { class: "portal-tab-switch",
                                        button {
                                            class: if auth_mode() == "patient" { "portal-tab-btn active" } else { "portal-tab-btn" },
                                            onclick: move |_| auth_mode.set("patient".to_string()),
                                            "Sou o Paciente"
                                        }
                                        button {
                                            class: if auth_mode() == "doctor" { "portal-tab-btn active" } else { "portal-tab-btn" },
                                            onclick: move |_| auth_mode.set("doctor".to_string()),
                                            "Sou o Doutor / Clínica"
                                        }
                                    }

                                    if let Some(ref err) = error_msg() {
                                        div { class: "portal-toast-error", "{err}" }
                                    }
                                    if let Some(ref succ) = success_msg() {
                                        div { class: "portal-toast-success", "{succ}" }
                                    }

                                    if auth_mode() == "patient" {
                                        div { class: "portal-form-group",
                                            label { "Seu CPF (apenas números ou formatado)" }
                                            input {
                                                r#type: "text",
                                                class: "portal-input",
                                                placeholder: "000.000.000-00",
                                                value: "{cpf_input}",
                                                oninput: move |e| cpf_input.set(e.value()),
                                            }
                                        }
                                        div { class: "portal-form-group",
                                            label { "Senha de Assinatura (cadastrada no seu prontuário)" }
                                            input {
                                                r#type: "password",
                                                class: "portal-input",
                                                placeholder: "Digite sua senha",
                                                value: "{patient_pwd}",
                                                oninput: move |e| patient_pwd.set(e.value()),
                                            }
                                        }
                                        button {
                                            class: "portal-btn-primary full-width",
                                            onclick: on_login_patient,
                                            "Validar Identidade do Paciente"
                                        }
                                    } else {
                                        div { class: "portal-form-group",
                                            label { "Usuário do Sistema" }
                                            input {
                                                r#type: "text",
                                                class: "portal-input",
                                                placeholder: "Ex: dr.joao",
                                                value: "{doctor_user}",
                                                oninput: move |e| doctor_user.set(e.value()),
                                            }
                                        }
                                        div { class: "portal-form-group",
                                            label { "Senha de Acesso" }
                                            input {
                                                r#type: "password",
                                                class: "portal-input",
                                                placeholder: "Digite sua senha",
                                                value: "{doctor_pwd}",
                                                oninput: move |e| doctor_pwd.set(e.value()),
                                            }
                                        }
                                        button {
                                            class: "portal-btn-primary full-width",
                                            onclick: on_login_doctor,
                                            "Validar Identidade Médica"
                                        }
                                    }
                                }
                            } else if let Some(sess) = auth_session() {
                                // Step 2: Signature Drawing & Optional WhatsApp OTP Validation
                                div { class: "portal-sign-card",
                                    div { class: "portal-user-welcome",
                                        IconShieldCheck { size: 24, color: "#10b981".to_string() }
                                        div {
                                            h3 { "Olá, {sess.signer_name}" }
                                            if sess.signer_type == "patient" {
                                                span { "Você está assinando como: Paciente" }
                                            } else {
                                                span { "Você está assinando como: Cirurgião-Dentista / Profissional" }
                                            }
                                        }
                                    }

                                    if let Some(ref err) = error_msg() {
                                        div { class: "portal-toast-error", "{err}" }
                                    }
                                    if let Some(ref succ) = success_msg() {
                                        div { class: "portal-toast-success", "{succ}" }
                                    }

                                    // WhatsApp OTP Confirmation Section (if required for patient)
                                    if sess.signer_type == "patient" && doc.require_whatsapp_otp {
                                        div { class: "portal-otp-section",
                                            h4 { "Validação de Segurança via WhatsApp" }
                                            p { "Enviaremos um código PIN de confirmação para seu WhatsApp ({doc.patient_phone_masked})." }

                                            div { class: "portal-otp-row",
                                                input {
                                                    r#type: "text",
                                                    class: "portal-input otp-field",
                                                    placeholder: "Código de 6 dígitos",
                                                    maxlength: "6",
                                                    value: "{otp_input}",
                                                    oninput: move |e| otp_input.set(e.value()),
                                                }
                                                button {
                                                    class: "portal-btn-secondary",
                                                    disabled: is_sending_otp(),
                                                    onclick: on_request_otp,
                                                    if is_sending_otp() { "Enviando..." } else if otp_sent() { "Reenviar Código" } else { "Enviar Código" }
                                                }
                                            }
                                        }
                                    }

                                    // Canvas Drawing Pad Area
                                    div { class: "portal-canvas-section",
                                        div { class: "canvas-header",
                                            label { "Desenhe sua Assinatura / Rubrica no quadro abaixo:" }
                                            button {
                                                class: "btn-clear-canvas",
                                                onclick: move |_| signature_data.set(String::new()),
                                                "Limpar Assinatura"
                                            }
                                        }

                                        div {
                                            id: "signature-canvas-wrapper",
                                            class: "signature-canvas-wrapper",
                                            // Interactive SVG / Canvas representation
                                            svg {
                                                id: "signature-drawing-pad",
                                                class: "signature-pad-svg",
                                                view_box: "0 0 500 180",
                                                onclick: move |_| {
                                                    // Sample valid vector signature data for browser simulation
                                                    signature_data.set("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 500 180'><path d='M30,120 C90,40 120,150 180,90 C220,50 250,140 320,80 C360,60 400,130 460,85' stroke='%230052cc' stroke-width='3' fill='none' stroke-linecap='round'/></svg>".to_string());
                                                },
                                                if signature_data().is_empty() {
                                                    text {
                                                        x: "250",
                                                        y: "95",
                                                        "text-anchor": "middle",
                                                        fill: "#94a3b8",
                                                        font_size: "14",
                                                        "Toque ou clique aqui para registrar sua assinatura"
                                                    }
                                                } else {
                                                    path {
                                                        d: "M30,120 C90,40 120,150 180,90 C220,50 250,140 320,80 C360,60 400,130 460,85",
                                                        stroke: "#0052cc",
                                                        "stroke-width": "3",
                                                        fill: "none",
                                                        "stroke-linecap": "round",
                                                    }
                                                }
                                            }
                                        }

                                        p { class: "signature-hint", "Ao confirmar, você declara ter lido e concordado integralmente com os termos deste contrato odontológico sob as penas da lei." }
                                    }

                                    button {
                                        class: "portal-btn-primary full-width",
                                        disabled: is_submitting() || signature_data().is_empty(),
                                        onclick: on_submit_signature,
                                        if is_submitting() {
                                            "Registrando Assinatura Criptografada..."
                                        } else {
                                            "Confirmar e Assinar Digitalmente"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "portal-error-card",
                        h3 { "Documento não encontrado ou inválido" }
                        p { "Verifique se o link ou QR Code escaneado está correto." }
                    }
                }
            }
        }
    }
}
