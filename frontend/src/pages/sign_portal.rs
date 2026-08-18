use crate::api::documents::{
    auth_doctor_signing, auth_patient_signing, check_patient_signing,
    fetch_public_signing_document, register_patient_password, request_signing_otp,
    submit_digital_signature,
};
use crate::components::icons::{
    IconCheck, IconCheckCircle, IconChevronLeft, IconClock, IconExternalLink,
    IconLock, IconRefresh, IconShieldCheck, IconSignature, IconTooth, IconUsers,
};
use dioxus::prelude::*;
use shared::documents::{
    DoctorSignAuthRequest, PatientCheckResponse, PatientSignAuthRequest,
    SignAuthResponse, SubmitSignatureRequest,
};

#[component]
pub fn SignPortal(token: String) -> Element {
    let signing_token = token.clone();

    // Document Resource
    let doc_res = use_resource(move || {
        let t = signing_token.clone();
        async move { fetch_public_signing_document(&t).await }
    });

    // Auth & Identity State
    let mut active_tab = use_signal(|| "patient".to_string()); // "patient" or "doctor"
    let mut auth_session = use_signal(|| None::<SignAuthResponse>);
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);

    // Patient Form State
    let mut patient_cpf = use_signal(String::new);
    let mut patient_check_info = use_signal(|| None::<PatientCheckResponse>);
    let mut is_checking_patient = use_signal(|| false);
    let mut patient_password_input = use_signal(String::new);
    let mut patient_confirm_password = use_signal(String::new);

    // Doctor Form State
    let mut doctor_username = use_signal(String::new);
    let mut doctor_password = use_signal(String::new);

    // Signature Canvas State
    let mut signature_data = use_signal(String::new);

    // OTP State
    let mut otp_channel = use_signal(|| "whatsapp".to_string()); // "whatsapp" or "email"
    let mut otp_input = use_signal(String::new);
    let mut is_sending_otp = use_signal(|| false);
    let mut otp_sent = use_signal(|| false);

    // Submission State
    let mut is_logging_in = use_signal(|| false);
    let mut is_submitting = use_signal(|| false);
    let mut is_completed = use_signal(|| false);
    let mut completed_checksum = use_signal(String::new);

    // Action: Check Patient CPF
    let on_check_patient = {
        let t = token.clone();
        move |_| {
            let cpf_val = patient_cpf();
            let clean_cpf: String = cpf_val.chars().filter(|c| c.is_alphanumeric()).collect();

            if clean_cpf.len() < 11 {
                error_msg.set(Some("Por favor, digite um CPF válido com 11 dígitos.".into()));
                return;
            }

            let t_clone = t.clone();
            spawn(async move {
                is_checking_patient.set(true);
                error_msg.set(None);

                match check_patient_signing(&t_clone, &cpf_val).await {
                    Ok(info) => {
                        is_checking_patient.set(false);
                        patient_check_info.set(Some(info));
                    }
                    Err(e) => {
                        is_checking_patient.set(false);
                        error_msg.set(Some(e));
                    }
                }
            });
        }
    };

    // Action: Register Patient Password
    let on_register_patient_password = {
        let t = token.clone();
        move |_| {
            let p1 = patient_password_input();
            let p2 = patient_confirm_password();
            if p1.trim().len() < 6 {
                error_msg.set(Some("A senha deve ter no mínimo 6 dígitos.".into()));
                return;
            }
            if p1 != p2 {
                error_msg.set(Some("As senhas digitadas não coincidem.".into()));
                return;
            }

            let t_clone = t.clone();
            let cpf_val = patient_cpf();
            spawn(async move {
                is_logging_in.set(true);
                error_msg.set(None);

                match register_patient_password(&t_clone, &cpf_val, &p1).await {
                    Ok(resp) => {
                        is_logging_in.set(false);
                        auth_session.set(Some(resp));
                        success_msg.set(Some("Senha cadastrada com sucesso! Agora desenhe sua assinatura no quadro abaixo.".into()));
                    }
                    Err(e) => {
                        is_logging_in.set(false);
                        error_msg.set(Some(e));
                    }
                }
            });
        }
    };

    // Action: Login Patient
    let on_login_patient = {
        let t = token.clone();
        move |_| {
            let t_clone = t.clone();
            let req = PatientSignAuthRequest {
                cpf: patient_cpf(),
                password: patient_password_input(),
            };

            spawn(async move {
                is_logging_in.set(true);
                error_msg.set(None);

                match auth_patient_signing(&t_clone, req).await {
                    Ok(resp) => {
                        is_logging_in.set(false);
                        auth_session.set(Some(resp));
                    }
                    Err(e) => {
                        is_logging_in.set(false);
                        error_msg.set(Some(e));
                    }
                }
            });
        }
    };

    // Action: Login Doctor
    let on_login_doctor = {
        let t = token.clone();
        move |_| {
            let t_clone = t.clone();
            let req = DoctorSignAuthRequest {
                username: doctor_username(),
                password: doctor_password(),
            };

            spawn(async move {
                is_logging_in.set(true);
                error_msg.set(None);

                match auth_doctor_signing(&t_clone, req).await {
                    Ok(resp) => {
                        is_logging_in.set(false);
                        auth_session.set(Some(resp));
                    }
                    Err(e) => {
                        is_logging_in.set(false);
                        error_msg.set(Some(e));
                    }
                }
            });
        }
    };

    // Action: Request OTP via WhatsApp or E-mail
    let on_request_otp = {
        let t = token.clone();
        move |_| {
            let t_clone = t.clone();
            let channel = otp_channel();
            spawn(async move {
                is_sending_otp.set(true);
                error_msg.set(None);

                match request_signing_otp(&t_clone, &channel).await {
                    Ok(msg) => {
                        is_sending_otp.set(false);
                        otp_sent.set(true);
                        success_msg.set(Some(msg));
                    }
                    Err(e) => {
                        is_sending_otp.set(false);
                        error_msg.set(Some(e));
                    }
                }
            });
        }
    };

    // Action: Submit Signature
    let on_submit_signature = {
        let t = token.clone();
        move |_| {
            let Some(ref sess) = auth_session() else { return; };
            let t_clone = t.clone();
            let sig_val = signature_data();

            if sig_val.is_empty() {
                error_msg.set(Some("Por favor, desenhe sua assinatura no quadro antes de confirmar.".into()));
                return;
            }

            let req = SubmitSignatureRequest {
                signature_base64: sig_val,
                signer_type: sess.signer_type.clone(),
                otp_code: if otp_input().is_empty() { None } else { Some(otp_input()) },
            };

            spawn(async move {
                is_submitting.set(true);
                error_msg.set(None);

                match submit_digital_signature(&t_clone, req).await {
                    Ok(doc) => {
                        is_submitting.set(false);
                        is_completed.set(true);
                        if let Some(cs) = doc.checksum_sha256 {
                            completed_checksum.set(cs);
                        }
                    }
                    Err(e) => {
                        is_submitting.set(false);
                        error_msg.set(Some(e));
                    }
                }
            });
        }
    };

    // Setup interactive pixel-perfect HTML5 Canvas with PointerEvents
    use_effect(move || {
        if auth_session().is_some() {
            let eval_script = r###"
                setTimeout(() => {
                    const canvas = document.getElementById("signature-canvas");
                    if (!canvas) return;
                    const ctx = canvas.getContext("2d");
                    ctx.strokeStyle = "#0052cc";
                    ctx.lineWidth = 2.8;
                    ctx.lineCap = "round";
                    ctx.lineJoin = "round";

                    function drawGuide() {
                        ctx.save();
                        ctx.strokeStyle = "#cbd5e1";
                        ctx.lineWidth = 1;
                        ctx.setLineDash([5, 5]);
                        ctx.beginPath();
                        ctx.moveTo(24, 130);
                        ctx.lineTo(476, 130);
                        ctx.stroke();
                        ctx.restore();
                    }
                    drawGuide();

                    let isDrawing = false;
                    let lastPos = { x: 0, y: 0 };

                    function getPos(e) {
                        const rect = canvas.getBoundingClientRect();
                        const scaleX = canvas.width / rect.width;
                        const scaleY = canvas.height / rect.height;
                        return {
                            x: (e.clientX - rect.left) * scaleX,
                            y: (e.clientY - rect.top) * scaleY
                        };
                    }

                    canvas.onpointerdown = (e) => {
                        canvas.setPointerCapture(e.pointerId);
                        isDrawing = true;
                        lastPos = getPos(e);
                        ctx.beginPath();
                        ctx.moveTo(lastPos.x, lastPos.y);
                    };

                    canvas.onpointermove = (e) => {
                        if (!isDrawing) return;
                        const pos = getPos(e);
                        ctx.lineTo(pos.x, pos.y);
                        ctx.stroke();
                        lastPos = pos;
                    };

                    const stopDrawing = (e) => {
                        if (!isDrawing) return;
                        isDrawing = false;
                        try { canvas.releasePointerCapture(e.pointerId); } catch(_) {}
                        const dataUrl = canvas.toDataURL("image/png");
                        const hidden = document.getElementById("signature-data-carrier");
                        if (hidden) {
                            hidden.value = dataUrl;
                            hidden.dispatchEvent(new Event("input", { bubbles: true }));
                        }
                    };

                    canvas.onpointerup = stopDrawing;
                    canvas.onpointercancel = stopDrawing;

                    window.__clearSignatureCanvas = () => {
                        ctx.clearRect(0, 0, canvas.width, canvas.height);
                        drawGuide();
                        const hidden = document.getElementById("signature-data-carrier");
                        if (hidden) {
                            hidden.value = "";
                            hidden.dispatchEvent(new Event("input", { bubbles: true }));
                        }
                    };
                }, 100);
            "###;
            let _ = document::eval(eval_script);
        }
    });

    rsx! {
        div { class: "sign-portal-wrapper",
            div { class: "portal-container",
                if let Some(Ok(doc)) = &*doc_res.read() {
                    // Header with Clinic Theming & Branding
                    header {
                        class: "portal-header",
                        style: "--theme-color: {doc.clinic_theme_color};",
                        div { class: "portal-brand",
                            if let Some(ref logo) = doc.clinic_logo_url {
                                if !logo.is_empty() && !logo.contains("placehold.co") {
                                    img {
                                        src: "{logo}",
                                        alt: "{doc.clinic_name}",
                                        class: "portal-clinic-logo-img",
                                    }
                                } else {
                                    div {
                                        class: "portal-clinic-icon-box",
                                        style: "background-color: {doc.clinic_theme_color};",
                                        IconTooth { size: 22, color: "white".to_string() }
                                    }
                                }
                            } else {
                                div {
                                    class: "portal-clinic-icon-box",
                                    style: "background-color: {doc.clinic_theme_color};",
                                    IconTooth { size: 22, color: "white".to_string() }
                                }
                            }
                            div { class: "portal-clinic-info",
                                h1 { class: "portal-clinic-name", "{doc.clinic_name}" }
                                span { class: "portal-clinic-sub", "Portal de Validação e Assinatura Digital" }
                            }
                        }
                        div { class: "portal-security-badge",
                            IconShieldCheck { size: 18, color: "#10b981".to_string() }
                            span { "Assinatura Digital (Lei 14.063/2020)" }
                        }
                    }

                    // Main Layout Grid
                    div { class: "portal-content-grid",
                        // Left Column: PDF Contract Viewer
                        div { class: "portal-pdf-column",
                            div { class: "portal-doc-header",
                                div { class: "portal-doc-info-left",
                                    span { class: "portal-doc-badge", "{doc.document.document_type.to_uppercase()}" }
                                    h2 { class: "portal-doc-title", "{doc.document.title}" }
                                    p { class: "portal-doc-meta", "Emitido em {crate::utils::format_date_br(&doc.document.created_at)}" }
                                }
                                div { class: "portal-doc-status-badge",
                                    if doc.document.status == "signed" || is_completed() {
                                        span { class: "badge-status-completed",
                                            IconCheckCircle { size: 16, color: "#10b981".to_string() }
                                            " Totalmente Assinado"
                                        }
                                    } else {
                                        span { class: "badge-status-pending",
                                            IconClock { size: 16, color: "#b45309".to_string() }
                                            " Aguardando Assinaturas"
                                        }
                                    }
                                }
                            }

                            // PDF Preview Container (Native Inline Embed)
                            div { class: "portal-preview-frame",
                                div { class: "pdf-frame-actions-bar",
                                    span { class: "pdf-frame-label", "Visualizador Oficial de Documento Clínico" }
                                    a {
                                        class: "pdf-frame-btn-open",
                                        href: "{doc.document.original_pdf_url}",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        IconExternalLink { size: 14, color: "{doc.clinic_theme_color}".to_string() }
                                        " Abrir em Nova Aba"
                                    }
                                }
                                object {
                                    data: "{doc.document.original_pdf_url}#toolbar=1&view=FitH",
                                    r#type: "application/pdf",
                                    class: "portal-pdf-embed",
                                    iframe {
                                        src: "{doc.document.original_pdf_url}",
                                        title: "Visualizador de Contrato",
                                        class: "portal-pdf-embed",
                                    }
                                }
                            }

                            // Signatures Status Box
                            div { class: "portal-signers-status",
                                h3 { class: "portal-signers-title", "Status dos Signatários" }
                                div { class: "portal-signers-grid",
                                    div { class: if doc.document.patient_signed_at.is_some() || (is_completed() && auth_session().as_ref().map(|s| s.signer_type.as_str()) == Some("patient")) { "signer-card signed" } else { "signer-card pending" },
                                        div { class: "signer-card-header",
                                            span { class: "signer-role", "Paciente" }
                                            if doc.document.patient_signed_at.is_some() || (is_completed() && auth_session().as_ref().map(|s| s.signer_type.as_str()) == Some("patient")) {
                                                span { class: "signer-badge-ok",
                                                    IconCheck { size: 12, color: "#15803d".to_string() }
                                                    " Assinado"
                                                }
                                            } else {
                                                span { class: "signer-badge-wait",
                                                    IconClock { size: 12, color: "#b45309".to_string() }
                                                    " Pendente"
                                                }
                                            }
                                        }
                                        p { class: "signer-name", "{doc.patient_phone_masked}" }
                                        if let Some(ref d) = doc.document.patient_signed_at {
                                            p { class: "signer-time", "Data: {d.chars().take(19).collect::<String>()}" }
                                        }
                                    }

                                    div { class: if doc.document.doctor_signed_at.is_some() || (is_completed() && auth_session().as_ref().map(|s| s.signer_type.as_str()) == Some("doctor")) { "signer-card signed" } else { "signer-card pending" },
                                        div { class: "signer-card-header",
                                            span { class: "signer-role", "Cirurgião-Dentista / Responsável" }
                                            if doc.document.doctor_signed_at.is_some() || (is_completed() && auth_session().as_ref().map(|s| s.signer_type.as_str()) == Some("doctor")) {
                                                span { class: "signer-badge-ok",
                                                    IconCheck { size: 12, color: "#15803d".to_string() }
                                                    " Assinado"
                                                }
                                            } else {
                                                span { class: "signer-badge-wait",
                                                    IconClock { size: 12, color: "#b45309".to_string() }
                                                    " Pendente"
                                                }
                                            }
                                        }
                                        p { class: "signer-name", "Corpo Clínico da Unidade" }
                                        if let Some(ref d) = doc.document.doctor_signed_at {
                                            p { class: "signer-time", "Data: {d.chars().take(19).collect::<String>()}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Right Column: Signing Actions & Interactive Pad
                        div { class: "portal-action-column",
                            if is_completed() {
                                // Success Screen after submission
                                div { class: "portal-success-card",
                                    div { class: "success-icon-wrap",
                                        IconCheckCircle { size: 48, color: "#10b981".to_string() }
                                    }
                                    h3 { "Assinatura Registrada com Sucesso!" }
                                    p { "Sua assinatura eletrônica e os metadados de integridade foram autenticados e salvos com segurança." }

                                    if !completed_checksum().is_empty() {
                                        div { class: "checksum-badge-box",
                                            span { class: "checksum-label", "Checksum SHA-256 de Autenticidade:" }
                                            code { "{completed_checksum()}" }
                                        }
                                    }

                                    p { class: "portal-footer-notice", "Uma via deste documento assinado está arquivada no prontuário eletrônico." }
                                }
                            } else if active_tab() == "patient" && doc.document.patient_signed_at.is_some() {
                                // Patient already signed this document
                                div { class: "portal-auth-card",
                                    div { class: "portal-tabs",
                                        button {
                                            class: "portal-tab active",
                                            style: "color: {doc.clinic_theme_color};",
                                            onclick: move |_| { active_tab.set("patient".into()); },
                                            IconUsers { size: 16, color: doc.clinic_theme_color.clone() }
                                            "Sou o Paciente"
                                        }
                                        button {
                                            class: "portal-tab",
                                            onclick: move |_| { active_tab.set("doctor".into()); },
                                            IconTooth { size: 16, color: "#64748b".to_string() }
                                            "Sou o Dentista"
                                        }
                                    }

                                    div { class: "portal-already-signed-card",
                                        div { class: "already-signed-icon",
                                            IconCheckCircle { size: 36, color: "#10b981".to_string() }
                                        }
                                        h3 { "Assinatura do Paciente Registrada" }
                                        p { "O paciente já realizou a assinatura digital deste documento clínico com validade jurídica." }
                                        div { class: "already-signed-details",
                                            span { "Data e Hora:" }
                                            strong { "{doc.document.patient_signed_at.as_deref().unwrap_or(\"\").chars().take(19).collect::<String>()} UTC" }
                                        }
                                        a {
                                            class: "portal-btn-secondary full-width",
                                            href: "{doc.document.original_pdf_url}",
                                            target: "_blank",
                                            rel: "noopener noreferrer",
                                            IconExternalLink { size: 16, color: "#334155".to_string() }
                                            "Visualizar Documento Assinado"
                                        }
                                    }
                                }
                            } else if active_tab() == "doctor" && doc.document.doctor_signed_at.is_some() {
                                // Doctor already signed this document
                                div { class: "portal-auth-card",
                                    div { class: "portal-tabs",
                                        button {
                                            class: "portal-tab",
                                            onclick: move |_| { active_tab.set("patient".into()); },
                                            IconUsers { size: 16, color: "#64748b".to_string() }
                                            "Sou o Paciente"
                                        }
                                        button {
                                            class: "portal-tab active",
                                            style: "color: {doc.clinic_theme_color};",
                                            onclick: move |_| { active_tab.set("doctor".into()); },
                                            IconTooth { size: 16, color: doc.clinic_theme_color.clone() }
                                            "Sou o Dentista"
                                        }
                                    }

                                    div { class: "portal-already-signed-card",
                                        div { class: "already-signed-icon",
                                            IconCheckCircle { size: 36, color: "#10b981".to_string() }
                                        }
                                        h3 { "Assinatura Médica Registrada" }
                                        p { "O cirurgião-dentista responsável já assinou e autenticou este documento." }
                                        div { class: "already-signed-details",
                                            span { "Data e Hora:" }
                                            strong { "{doc.document.doctor_signed_at.as_deref().unwrap_or(\"\").chars().take(19).collect::<String>()} UTC" }
                                        }
                                        a {
                                            class: "portal-btn-secondary full-width",
                                            href: "{doc.document.original_pdf_url}",
                                            target: "_blank",
                                            rel: "noopener noreferrer",
                                            IconExternalLink { size: 16, color: "#334155".to_string() }
                                            "Visualizar Documento Assinado"
                                        }
                                    }
                                }
                            } else if auth_session().is_none() {
                                // Step 1: Authentication Tabs
                                div { class: "portal-auth-card",
                                    div { class: "portal-tabs",
                                        button {
                                            class: if active_tab() == "patient" { "portal-tab active" } else { "portal-tab" },
                                            style: if active_tab() == "patient" { "color: {doc.clinic_theme_color};" } else { "" },
                                            onclick: move |_| { active_tab.set("patient".into()); error_msg.set(None); },
                                            IconUsers { size: 16, color: if active_tab() == "patient" { doc.clinic_theme_color.clone() } else { "#64748b".to_string() } }
                                            "Sou o Paciente"
                                        }
                                        button {
                                            class: if active_tab() == "doctor" { "portal-tab active" } else { "portal-tab" },
                                            style: if active_tab() == "doctor" { "color: {doc.clinic_theme_color};" } else { "" },
                                            onclick: move |_| { active_tab.set("doctor".into()); error_msg.set(None); },
                                            IconTooth { size: 16, color: if active_tab() == "doctor" { doc.clinic_theme_color.clone() } else { "#64748b".to_string() } }
                                            "Sou o Dentista"
                                        }
                                    }

                                    if let Some(ref err) = error_msg() {
                                        div { class: "portal-toast-error", "{err}" }
                                    }

                                    if active_tab() == "patient" {
                                        if let Some(info) = patient_check_info() {
                                            if info.has_password {
                                                // Patient has existing password
                                                div { class: "portal-auth-form",
                                                    div { class: "patient-identified-badge",
                                                        IconShieldCheck { size: 18, color: "#166534".to_string() }
                                                        div {
                                                            strong { "{info.patient_name}" }
                                                            span { class: "patient-id-sub", "Identidade localizada" }
                                                        }
                                                    }
                                                    label { class: "portal-label", "Digite sua Senha de Assinatura:" }
                                                    input {
                                                        r#type: "password",
                                                        class: "portal-input",
                                                        placeholder: "Sua senha de 6 dígitos",
                                                        value: "{patient_password_input}",
                                                        oninput: move |e| patient_password_input.set(e.value()),
                                                    }
                                                    button {
                                                        class: "portal-btn-primary full-width",
                                                        style: "background-color: {doc.clinic_theme_color};",
                                                        disabled: is_logging_in() || patient_password_input().is_empty(),
                                                        onclick: on_login_patient,
                                                        IconLock { size: 16, color: "white".to_string() }
                                                        if is_logging_in() { "Validando Acesso..." } else { "Acessar e Assinar" }
                                                    }
                                                    button {
                                                        class: "portal-btn-ghost full-width",
                                                        onclick: move |_| { patient_check_info.set(None); error_msg.set(None); },
                                                        IconChevronLeft { size: 14, color: "#64748b".to_string() }
                                                        "Informar outro CPF"
                                                    }
                                                }
                                            } else {
                                                // Patient needs to create a password
                                                div { class: "portal-auth-form",
                                                    div { class: "patient-identified-badge",
                                                        IconShieldCheck { size: 18, color: "#166534".to_string() }
                                                        div {
                                                            strong { "Olá, {info.patient_name}" }
                                                            span { class: "patient-id-sub", "Cadastre sua senha de assinatura" }
                                                        }
                                                    }
                                                    p { class: "portal-helper-text", "Crie uma senha de no mínimo 6 dígitos para proteger e assinar seus termos:" }
                                                    label { class: "portal-label", "Crie sua Senha de Assinatura:" }
                                                    input {
                                                        r#type: "password",
                                                        class: "portal-input",
                                                        placeholder: "Mínimo 6 dígitos",
                                                        value: "{patient_password_input}",
                                                        oninput: move |e| patient_password_input.set(e.value()),
                                                    }
                                                    label { class: "portal-label", "Confirme a Senha:" }
                                                    input {
                                                        r#type: "password",
                                                        class: "portal-input",
                                                        placeholder: "Repita a mesma senha",
                                                        value: "{patient_confirm_password}",
                                                        oninput: move |e| patient_confirm_password.set(e.value()),
                                                    }
                                                    button {
                                                        class: "portal-btn-primary full-width",
                                                        style: "background-color: {doc.clinic_theme_color};",
                                                        disabled: is_logging_in() || patient_password_input().is_empty(),
                                                        onclick: on_register_patient_password,
                                                        IconCheck { size: 16, color: "white".to_string() }
                                                        if is_logging_in() { "Cadastrando Senha..." } else { "Cadastrar Senha e Prosseguir" }
                                                    }
                                                    button {
                                                        class: "portal-btn-ghost full-width",
                                                        onclick: move |_| { patient_check_info.set(None); error_msg.set(None); },
                                                        IconChevronLeft { size: 14, color: "#64748b".to_string() }
                                                        "Informar outro CPF"
                                                    }
                                                }
                                            }
                                        } else {
                                            // Step 1: Input CPF
                                            div { class: "portal-auth-form",
                                                label { class: "portal-label", "Informe seu CPF cadastrado na clínica:" }
                                                input {
                                                    r#type: "text",
                                                    class: "portal-input",
                                                    placeholder: "000.000.000-00",
                                                    value: "{patient_cpf}",
                                                    oninput: move |e| patient_cpf.set(e.value()),
                                                }
                                                p { class: "portal-helper-text", "Seus dados estão protegidos por criptografia de ponta a ponta." }
                                                button {
                                                    class: "portal-btn-primary full-width",
                                                    style: "background-color: {doc.clinic_theme_color};",
                                                    disabled: is_checking_patient() || patient_cpf().is_empty(),
                                                    onclick: on_check_patient,
                                                    IconCheckCircle { size: 16, color: "white".to_string() }
                                                    if is_checking_patient() { "Verificando CPF..." } else { "Continuar" }
                                                }
                                            }
                                        }
                                    } else {
                                        // Doctor Auth Form
                                        div { class: "portal-auth-form",
                                            label { class: "portal-label", "Usuário do Sistema:" }
                                            input {
                                                r#type: "text",
                                                class: "portal-input",
                                                placeholder: "Ex: dr.andre",
                                                value: "{doctor_username}",
                                                oninput: move |e| doctor_username.set(e.value()),
                                            }
                                            label { class: "portal-label", "Senha Profissional:" }
                                            input {
                                                r#type: "password",
                                                class: "portal-input",
                                                placeholder: "Sua senha de dentista",
                                                value: "{doctor_password}",
                                                oninput: move |e| doctor_password.set(e.value()),
                                            }
                                            button {
                                                class: "portal-btn-primary full-width",
                                                style: "background-color: {doc.clinic_theme_color};",
                                                disabled: is_logging_in() || doctor_username().is_empty(),
                                                onclick: on_login_doctor,
                                                IconShieldCheck { size: 16, color: "white".to_string() }
                                                if is_logging_in() { "Validando Acesso..." } else { "Validar Identidade Médica" }
                                            }
                                        }
                                    }
                                }
                            } else if let Some(sess) = auth_session() {
                                // Step 2: Signature Drawing & Optional WhatsApp/E-mail OTP Validation
                                div { class: "portal-sign-card",
                                    // Clean Minimalist Signer Info (NO bulky green box)
                                    div { class: "portal-signer-simple-header",
                                        div { class: "signer-info-text",
                                            h3 { class: "signer-main-name", "Olá, {sess.signer_name}" }
                                            span { class: "signer-role-pill",
                                                if sess.signer_type == "patient" { "Signatário: Paciente" } else { "Signatário: Cirurgião-Dentista / Responsável" }
                                            }
                                        }
                                    }

                                    if let Some(ref err) = error_msg() {
                                        div { class: "portal-toast-error", "{err}" }
                                    }
                                    if let Some(ref succ) = success_msg() {
                                        div { class: "portal-toast-success", "{succ}" }
                                    }

                                    // OTP Verification Section with Channel Selector (WhatsApp or E-mail)
                                    if sess.signer_type == "patient" && doc.require_whatsapp_otp {
                                        div { class: "portal-otp-section",
                                            h4 { "Validação de Segurança de Assinatura" }
                                            p { "Escolha por onde deseja receber seu código de verificação PIN:" }

                                            // Channel Selector Tabs
                                            div { class: "portal-otp-channel-picker",
                                                button {
                                                    class: if otp_channel() == "whatsapp" { "otp-channel-btn active" } else { "otp-channel-btn" },
                                                    onclick: move |_| { otp_channel.set("whatsapp".into()); },
                                                    "📱 WhatsApp ({doc.patient_phone_masked})"
                                                }
                                                if let Some(ref email_m) = doc.patient_email_masked {
                                                    button {
                                                        class: if otp_channel() == "email" { "otp-channel-btn active" } else { "otp-channel-btn" },
                                                        onclick: move |_| { otp_channel.set("email".into()); },
                                                        "✉️ E-mail ({email_m})"
                                                    }
                                                }
                                            }

                                            div { class: "portal-otp-row",
                                                input {
                                                    r#type: "text",
                                                    class: "portal-input otp-field",
                                                    placeholder: "PIN 6 dígitos",
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

                                    // HTML5 Canvas Drawing Pad Area (with ONLY "Limpar" button)
                                    div { class: "portal-canvas-section",
                                        div { class: "canvas-header",
                                            label { "Desenhe sua assinatura no quadro abaixo:" }
                                            div { class: "canvas-actions-right",
                                                button {
                                                    class: "btn-canvas-action",
                                                    onclick: move |_| {
                                                        signature_data.set(String::new());
                                                        let _ = document::eval("window.__clearSignatureCanvas && window.__clearSignatureCanvas();");
                                                    },
                                                    IconRefresh { size: 13, color: "#475569".to_string() }
                                                    "Limpar"
                                                }
                                            }
                                        }

                                        div {
                                            id: "signature-canvas-wrapper",
                                            class: "signature-canvas-wrapper",
                                            // Real HTML5 Canvas with PointerEvents
                                            canvas {
                                                id: "signature-canvas",
                                                class: "signature-html-canvas",
                                                width: "500",
                                                height: "180",
                                            }
                                            // Hidden Carrier Input to bridge canvas dataURL to Dioxus reactive signal
                                            input {
                                                id: "signature-data-carrier",
                                                r#type: "hidden",
                                                value: "{signature_data}",
                                                oninput: move |e| signature_data.set(e.value()),
                                            }
                                        }

                                        p { class: "signature-hint", "Ao confirmar, você declara ter lido e concordado integralmente com os termos deste documento clínico sob as penas da lei." }
                                    }

                                    button {
                                        class: "portal-btn-primary full-width portal-btn-glow",
                                        style: "background-color: {doc.clinic_theme_color};",
                                        disabled: is_submitting() || signature_data().is_empty(),
                                        onclick: on_submit_signature,
                                        IconSignature { size: 18, color: "white".to_string() }
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
