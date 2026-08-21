//! # Aba de Contratos e Documentos Digitais do Paciente (Frontend)
//!
//! Controla a listagem de termos e contratos emitidos para o paciente,
//! emissão de novos documentos com QR Code e pré-visualização de PDF.

use crate::api::create_patient_document;
use crate::components::icons::{
    IconCheckCircle, IconCopy, IconExternalLink, IconEye, IconFile, IconQrCode, IconSignature,
    IconUpload, IconWhatsApp,
};
use crate::utils::{build_signing_url, resolve_file_url};
use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;
use shared::documents::{ContractTemplate, CreatePatientDocumentRequest, PatientDocument};

/// Gera o SVG do QR Code a partir de uma URL.
fn generate_qr_svg(url: &str) -> String {
    if let Ok(code) = QrCode::new(url.as_bytes()) {
        code.render::<svg::Color>()
            .min_dimensions(180, 180)
            .dark_color(svg::Color("#0052cc"))
            .light_color(svg::Color("#ffffff"))
            .build()
    } else {
        String::new()
    }
}

/// Codifica texto simples para URL query param sem dependência externa.
fn simple_url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for ch in input.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => encoded.push(ch),
            ' ' => encoded.push_str("%20"),
            _ => {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                for b in s.bytes() {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    encoded
}

/// Componente da aba de Contratos e Documentos com suporte a QR Code, emissão e visualização de PDF.
#[component]
pub fn PatientDocumentsTab(
    patient_id: String,
    patient_name: String,
    patient_cpf: Option<String>,
    patient_phone: Option<String>,
    patient_insurance: Option<String>,
    clinic_id: String,
    token: String,
    documents: Vec<PatientDocument>,
    templates: Vec<ContractTemplate>,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
    is_emit_modal_open: Signal<bool>,
) -> Element {
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);

    // Emit form state
    let default_title = format!("Contrato de Prestação de Serviços - {}", patient_name);
    let mut emit_doc_title = use_signal(move || default_title.clone());
    let mut emit_doc_type = use_signal(|| "Contrato Odontológico (E-Sign)".to_string());
    let mut emit_template_id = use_signal(String::new);
    let mut is_emitting = use_signal(|| false);

    // Signature requirements
    let mut req_patient_sign = use_signal(|| true);
    let mut req_doctor_sign = use_signal(|| false);
    let mut dentist_sign_mode = use_signal(|| "any".to_string());

    let pat_id = patient_id.clone();
    let pat_name = patient_name.clone();
    let pat_cpf_str = patient_cpf.clone().unwrap_or_else(|| "000.000.000-00".into());
    let pat_phone_str = patient_phone.clone().unwrap_or_else(|| "(11) 90000-0000".into());
    let pat_ins_str = patient_insurance.clone().unwrap_or_else(|| "Particular".into());
    let cid = clinic_id.clone();
    let tok = token.clone();

    let mut handle_emit = move |_| {
        let title = emit_doc_title().trim().to_string();
        if title.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o título do documento/termo.".into()));
            return;
        }

        let tpl_id = if emit_template_id().is_empty() {
            None
        } else {
            Some(emit_template_id())
        };

        let allow_any = dentist_sign_mode() == "any";

        let req = CreatePatientDocumentRequest {
            clinic_id: cid.clone(),
            patient_id: pat_id.clone(),
            template_id: tpl_id,
            doctor_user_id: None,
            appointment_id: None,
            title,
            document_type: emit_doc_type(),
            pdf_url: None,
            signed_pdf_url: None,
            is_already_signed: Some(false),
            requires_patient_signature: Some(req_patient_sign()),
            requires_doctor_signature: Some(req_doctor_sign()),
            allow_any_dentist_signature: Some(allow_any),
        };

        let t = tok.clone();
        let mut open_sig = is_emit_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut emit_sig = is_emitting;
        let reload = reload_patient_details.clone();

        emit_sig.set(true);
        spawn(async move {
            match create_patient_document(&t, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Documento emitido com sucesso para assinatura!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao emitir documento: {}", e)));
                }
            }
            emit_sig.set(false);
        });
    };

    rsx! {
        div { class: "patient-tab-content",
            // Header da Aba (Alinhado com Exames e Tratamentos)
            div { class: "tab-header-actions-row",
                div { class: "tab-header-title-group",
                    h3 { class: "tab-header-title", "Contratos & Termos do Paciente" }
                    p { class: "tab-header-desc", "Documentos emitidos, assinaturas eletrônicas pendentes e concluídas." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| is_emit_modal_open.set(true),
                        IconSignature { size: 16, color: "#ffffff".to_string() }
                        span { " Emitir Contrato / Termo" }
                    }
                }
            }

            if documents.is_empty() {
                div { class: "empty-state-card",
                    div { class: "empty-state-icon-box",
                        IconFile { size: 32, color: "currentColor".to_string() }
                    }
                    h3 { "Nenhum contrato ou termo emitido" }
                    p { "Clique no botão 'Emitir Contrato / Termo' acima para vincular e gerar um documento para assinatura digital." }
                }
            } else {
                div { class: "table-container",
                    table { class: "modern-table",
                        thead {
                            tr {
                                th { "Título do Documento" }
                                th { "Tipo" }
                                th { "Data de Emissão" }
                                th { "Status de Assinatura" }
                                th { class: "text-right", "Ações e QR Code" }
                            }
                        }
                        tbody {
                            for doc in &documents {
                                {
                                    let doc_clone = doc.clone();
                                    let raw_pdf_url = doc.signed_pdf_url.clone().unwrap_or_else(|| doc.original_pdf_url.clone());
                                    let pdf_url_to_preview = resolve_file_url(&raw_pdf_url);
                                    let pdf_title = doc.title.clone();

                                    rsx! {
                                        tr { key: "{doc.id}",
                                            td {
                                                div { class: "flex items-center gap-2",
                                                    IconFile { size: 16, color: "#0052cc".to_string() }
                                                    span { class: "font-semibold", "{doc.title}" }
                                                }
                                            }
                                            td {
                                                span { class: "badge-outline", "{doc.document_type}" }
                                            }
                                            td { "{doc.created_at.chars().take(10).collect::<String>()}" }
                                            td {
                                                if doc.status == "signed" || doc.status == "completed" {
                                                    span { class: "badge-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        span { " Assinado" }
                                                    }
                                                } else {
                                                    span { class: "badge-pending",
                                                        IconSignature { size: 14, color: "#f59e0b".to_string() }
                                                        span { " Pendente de Assinatura" }
                                                    }
                                                }
                                            }
                                            td { class: "text-right",
                                                div { class: "table-actions-row",
                                                    button {
                                                        class: "btn-action-icon",
                                                        title: "Abrir QR Code / Link de Assinatura",
                                                        onclick: move |_| qr_modal_doc.set(Some(doc_clone.clone())),
                                                        IconQrCode { size: 16, color: "#0052cc".to_string() }
                                                    }

                                                    if !pdf_url_to_preview.is_empty() {
                                                        {
                                                            let u = pdf_url_to_preview.clone();
                                                            let t = pdf_title.clone();
                                                            rsx! {
                                                                button {
                                                                    class: "btn-action-icon",
                                                                    title: "Visualizar PDF",
                                                                    onclick: move |_| pdf_preview_target.set(Some((u.clone(), t.clone()))),
                                                                    IconEye { size: 16, color: "#475569".to_string() }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Emissão de Novo Documento
            if is_emit_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Emitir Novo Documento / Contrato" }
                                p { class: "modal-subtitle", "Selecione o modelo e preencha os dados do termo para assinatura." }
                            }
                            button { class: "modal-close", onclick: move |_| is_emit_modal_open.set(false), "×" }
                        }
                        div { class: "modal-body",
                            div { class: "form-group",
                                label { "Título do Documento *" }
                                input {
                                    class: "input-field",
                                    value: "{emit_doc_title}",
                                    placeholder: "Ex: Termo de Consentimento - Clareamento Dental",
                                    oninput: move |e| emit_doc_title.set(e.value())
                                }
                            }

                            div { class: "form-row",
                                div { class: "form-group col-md-6",
                                    label { "Tipo de Documento" }
                                    select {
                                        class: "input-field",
                                        value: "{emit_doc_type}",
                                        onchange: move |e| emit_doc_type.set(e.value()),
                                        option { value: "contract", "Contrato de Prestação de Serviços" }
                                        option { value: "consent", "Termo de Consentimento (TCLE)" }
                                        option { value: "budget", "Orçamento Formal" }
                                        option { value: "declaration", "Declaração de Comparecimento" }
                                    }
                                }

                                div { class: "form-group col-md-6",
                                    label { "Modelo de Contrato (Opcional)" }
                                    select {
                                        class: "input-field",
                                        value: "{emit_template_id}",
                                        onchange: move |e| {
                                            let val = e.value();
                                            emit_template_id.set(val.clone());
                                            if let Some(t) = templates.iter().find(|t| t.id == val) {
                                                emit_doc_title.set(format!("{} - {}", t.title, pat_name));
                                                emit_doc_type.set(t.category.clone());
                                                req_patient_sign.set(t.requires_patient_signature);
                                                req_doctor_sign.set(t.requires_doctor_signature);
                                                if !t.allow_any_dentist_signature {
                                                    dentist_sign_mode.set("specific".into());
                                                }
                                            }
                                        },
                                        option { value: "", "Documento em Branco / Padrão" }
                                        for tpl in &templates {
                                            option { value: "{tpl.id}", "{tpl.title}" }
                                        }
                                    }
                                }
                            }

                            // Configuração de Requisitos de Assinatura
                            div { style: "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px 14px; margin-top: 10px; display: flex; flex-direction: column; gap: 8px;",
                                span { style: "font-size: 12.5px; font-weight: 700; color: #0f172a;", "Requisitos de Assinatura" }
                                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                                    label { style: "display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 600; color: #334155; cursor: pointer;",
                                        input {
                                            r#type: "checkbox",
                                            checked: req_patient_sign(),
                                            onchange: move |e| req_patient_sign.set(e.value() == "true"),
                                        }
                                        span { "Assinatura do Paciente" }
                                    }
                                    label { style: "display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 600; color: #334155; cursor: pointer;",
                                        input {
                                            r#type: "checkbox",
                                            checked: req_doctor_sign(),
                                            onchange: move |e| req_doctor_sign.set(e.value() == "true"),
                                        }
                                        span { "Assinatura do Dentista" }
                                    }
                                }
                            }
                        }
                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_emit_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_emitting(),
                                onclick: move |e| handle_emit(e),
                                if is_emitting() { "Emitindo..." } else { "Emitir e Gerar QR Code" }
                            }
                        }
                    }
                }
            }

            // Modal do QR Code & Envio WhatsApp
            if let Some(ref doc) = *qr_modal_doc.read() {
                {
                    let sign_url = build_signing_url(&doc.signing_token);
                    let qr_svg = generate_qr_svg(&sign_url);
                    let wa_msg = format!("Olá {}, seu documento '{}' está pronto para assinatura digital: {}", patient_name, doc.title, sign_url);
                    let raw_phone: String = patient_phone.as_deref().unwrap_or("").chars().filter(|c| c.is_ascii_digit()).collect();
                    let wa_link = format!("https://wa.me/55{}?text={}", raw_phone, simple_url_encode(&wa_msg));

                    rsx! {
                        div { class: "modal-overlay",
                            onclick: move |_| qr_modal_doc.set(None),
                            div { class: "action-modal qr-modal-card",
                                onclick: move |e| e.stop_propagation(),
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "Assinatura Digital - {doc.title}" }
                                        p { class: "modal-subtitle", "Aponte a câmera do celular para o QR Code para abrir o portal de assinatura do paciente." }
                                    }
                                    button { class: "modal-close", onclick: move |_| qr_modal_doc.set(None), "×" }
                                }
                                div { class: "modal-body text-center",
                                    div { class: "qr-box-center",
                                        div {
                                            class: "qr-svg-wrapper",
                                            dangerous_inner_html: "{qr_svg}"
                                        }
                                    }

                                    p { class: "qr-doc-title", "{doc.title}" }
                                    p { class: "qr-hint", "O paciente poderá visualizar o contrato na íntegra, autenticar-se e desenhar a assinatura na tela do celular ou tablet." }

                                    div { class: "qr-link-copy-box",
                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            class: "input-field font-mono font-xs",
                                            value: "{sign_url}"
                                        }
                                        a {
                                            href: "{sign_url}",
                                            target: "_blank",
                                            class: "btn-secondary",
                                            IconExternalLink { size: 16, color: "#0052cc".to_string() }
                                            span { " Abrir Portal" }
                                        }
                                    }
                                }
                                div { class: "modal-footer", style: "display: flex; gap: 8px; justify-content: flex-end;",
                                    a {
                                        class: "btn-primary",
                                        style: "background-color: #25D366; border-color: #25D366; color: #ffffff; display: inline-flex; align-items: center; gap: 8px; text-decoration: none;",
                                        href: "{wa_link}",
                                        target: "_blank",
                                        IconWhatsApp { size: 18, color: "#ffffff".to_string() }
                                        span { "Enviar Link por WhatsApp" }
                                    }
                                    button { class: "btn-secondary", onclick: move |_| qr_modal_doc.set(None), "Fechar" }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Pré-Visualização de PDF
            if let Some((ref pdf_url, ref title)) = *pdf_preview_target.read() {
                {
                    let resolved_pdf_url = resolve_file_url(pdf_url);
                    rsx! {
                        div { class: "modal-overlay",
                            onclick: move |_| pdf_preview_target.set(None),
                            div { class: "action-modal pdf-viewer-modal",
                                onclick: move |e| e.stop_propagation(),
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "{title}" }
                                        p { class: "modal-subtitle", "Documento PDF Clínico" }
                                    }
                                    button { class: "modal-close", onclick: move |_| pdf_preview_target.set(None), "×" }
                                }
                                div { class: "modal-body p-0", style: "height: 68vh; overflow: hidden; background: #0f172a;",
                                    iframe {
                                        src: "{resolved_pdf_url}",
                                        style: "width: 100%; height: 100%; border: none;",
                                        title: "{title}"
                                    }
                                }
                                div { class: "modal-footer",
                                    div { class: "flex items-center justify-between full-width",
                                        a {
                                            class: "btn-secondary",
                                            href: "{resolved_pdf_url}",
                                            target: "_blank",
                                            IconExternalLink { size: 14, color: "currentColor".to_string() }
                                            span { " Abrir em Nova Aba" }
                                        }
                                        button { class: "btn-primary", onclick: move |_| pdf_preview_target.set(None), "Fechar" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
