//! # Aba de Contratos e Documentos Digitais do Paciente (Frontend)
//!
//! Controla a listagem de termos e contratos emitidos para o paciente,
//! emissão de novos documentos com QR Code e pré-visualização de PDF.

use crate::api::create_patient_document;
use crate::components::icons::{
    IconCheckCircle, IconExternalLink, IconEye, IconFile, IconQrCode, IconSignature, IconUpload,
};
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
                                    let pdf_url_to_preview = doc.signed_pdf_url.clone().unwrap_or_else(|| doc.original_pdf_url.clone());
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

            // Modal de Emissão de Documento / Contrato
            if is_emit_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal stock-custom-modal", style: "max-width: 620px;",
                        div { class: "settings-header",
                            div {
                                h2 { class: "settings-title", "Emitir Contrato / Termo de Assinatura" }
                                p { class: "text-muted font-xs mt-1",
                                    "Vincule um modelo de contrato para assinatura digital imediata ou anexe um documento pronto."
                                }
                            }
                            button { class: "close-btn", onclick: move |_| is_emit_modal_open.set(false), "×" }
                        }

                        div { class: "settings-content",
                            // Patient Info Banner (Light Blue Box)
                            div { class: "patient-info-banner-blue",
                                div {
                                    span { class: "banner-blue-item-label", "PACIENTE" }
                                    p { class: "banner-blue-item-val", "{pat_name}" }
                                }
                                div {
                                    span { class: "banner-blue-item-label", "CPF PROTEGIDO" }
                                    p { class: "banner-blue-item-val", "{pat_cpf_str}" }
                                }
                                div {
                                    span { class: "banner-blue-item-label", "WHATSAPP" }
                                    p { class: "banner-blue-item-val", "{pat_phone_str}" }
                                }
                                div {
                                    span { class: "banner-blue-item-label", "CONVÊNIO" }
                                    p { class: "banner-blue-item-val", "{pat_ins_str}" }
                                }
                            }

                            div { class: "form-group",
                                label { "Título do Documento *" }
                                input {
                                    class: "form-input",
                                    value: "{emit_doc_title}",
                                    oninput: move |e| emit_doc_title.set(e.value())
                                }
                            }

                            div { class: "form-grid-2",
                                div { class: "form-group",
                                    label { "Tipo de Documento" }
                                    select {
                                        class: "form-input",
                                        value: "{emit_doc_type}",
                                        onchange: move |e| emit_doc_type.set(e.value()),
                                        option { value: "Contrato Odontológico (E-Sign)", "Contrato Odontológico (E-Sign)" }
                                        option { value: "Termo de Consentimento (TCLE)", "Termo de Consentimento (TCLE)" }
                                        option { value: "Atestado Odontológico", "Atestado Odontológico" }
                                        option { value: "Receituário Especial", "Receituário Especial" }
                                        option { value: "Outro", "Outro" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Modelo Base de Contrato" }
                                    select {
                                        class: "form-input",
                                        value: "{emit_template_id}",
                                        onchange: move |e| emit_template_id.set(e.value()),
                                        option { value: "", "Documento em Branco / Padrão" }
                                        for tpl in &templates {
                                            option { value: "{tpl.id}", "{tpl.title}" }
                                        }
                                    }
                                }
                            }

                            div { class: "form-group",
                                label { "Arquivo PDF do Documento (Opcional se usar modelo)" }
                                div { class: "contract-upload-dropzone",
                                    IconUpload { size: 18, color: "#0052cc".to_string() }
                                    span { "Fazer Upload do PDF" }
                                }
                            }
                        }

                        div { class: "modal-footer-actions",
                            button { class: "btn-secondary", onclick: move |_| is_emit_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_emitting(),
                                onclick: move |e| handle_emit(e),
                                if is_emitting() { "Emitindo..." } else { "Emitir e Gerar QR Code de Assinatura" }
                            }
                        }
                    }
                }
            }

            // Modal do QR Code & Envio WhatsApp
            if let Some(ref doc) = *qr_modal_doc.read() {
                {
                    let sign_url = format!("https://app.smileplus.com.br/sign/{}", doc.signing_token);
                    let qr_svg = generate_qr_svg(&sign_url);
                    let wa_msg = format!("Olá {}, seu documento '{}' está pronto para assinatura digital: {}", patient_name, doc.title, sign_url);
                    let raw_phone: String = patient_phone.as_deref().unwrap_or("").chars().filter(|c| c.is_ascii_digit()).collect();
                    let wa_link = format!("https://wa.me/55{}?text={}", raw_phone, simple_url_encode(&wa_msg));

                    rsx! {
                        div { class: "modal-overlay",
                            div { class: "action-modal qr-modal-card",
                                div { class: "settings-header",
                                    h2 { class: "settings-title", "Assinatura Digital - {doc.title}" }
                                    button { class: "close-btn", onclick: move |_| qr_modal_doc.set(None), "×" }
                                }
                                div { class: "settings-content text-center",
                                    p { class: "text-muted font-xs", "Aponte a câmera do celular para o QR Code para abrir o portal de assinatura do paciente." }

                                    div { class: "qr-code-box my-4",
                                        dangerous_inner_html: "{qr_svg}"
                                    }

                                    div { class: "signing-link-input-group",
                                        input {
                                            class: "form-input font-mono font-xs",
                                            readonly: true,
                                            value: "{sign_url}"
                                        }
                                    }
                                }
                                div { class: "modal-footer-actions",
                                    a {
                                        class: "btn-primary flex items-center gap-2",
                                        href: "{wa_link}",
                                        target: "_blank",
                                        "📱 Enviar Link por WhatsApp"
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
                div { class: "modal-overlay",
                    div { class: "action-modal pdf-modal-card", style: "max-width: 900px; height: 85vh;",
                        div { class: "settings-header",
                            h2 { class: "settings-title", "{title}" }
                            button { class: "close-btn", onclick: move |_| pdf_preview_target.set(None), "×" }
                        }
                        div { class: "settings-content p-0", style: "height: calc(100% - 120px);",
                            iframe {
                                src: "{pdf_url}",
                                style: "width: 100%; height: 100%; border: none;",
                            }
                        }
                        div { class: "modal-footer-actions",
                            a {
                                class: "btn-secondary flex items-center gap-2",
                                href: "{pdf_url}",
                                target: "_blank",
                                IconExternalLink { size: 16, color: "currentColor".to_string() }
                                span { "Abrir em Nova Aba" }
                            }
                            button { class: "btn-primary", onclick: move |_| pdf_preview_target.set(None), "Fechar" }
                        }
                    }
                }
            }
        }
    }
}
