//! # Aba de Contratos e Documentos Digitais do Paciente (Frontend)
//!
//! Controla a listagem de termos e contratos emitidos para o paciente,
//! emissão de novos documentos, envio de link via WhatsApp, modal com QR Code e pré-visualização de PDF.

use crate::api::create_patient_document;
use crate::components::icons::{
    IconCheckCircle, IconExternalLink, IconEye, IconFile, IconLock, IconQrCode, IconSignature,
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

/// Componente da aba de Contratos e Documentos com suporte a QR Code, emissão e visualização de PDF.
#[component]
pub fn PatientDocumentsTab(
    patient_id: String,
    patient_name: String,
    clinic_id: String,
    token: String,
    documents: Vec<PatientDocument>,
    templates: Vec<ContractTemplate>,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_emit_modal_open = use_signal(|| false);
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);

    // Emit form state
    let mut emit_template_id = use_signal(String::new);
    let mut emit_doc_title = use_signal(String::new);
    let mut is_emitting = use_signal(|| false);

    let pat_id = patient_id.clone();
    let pat_name = patient_name.clone();
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
            document_type: "contract".to_string(),
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
            div { class: "tab-actions-header",
                div {
                    h3 { class: "tab-title", "Contratos & Termos do Paciente" }
                    p { class: "tab-subtitle", "Documentos emitidos, assinaturas eletrônicas pendentes e concluídas." }
                }
                if can_write {
                    button {
                        class: "btn-primary",
                        onclick: move |_| is_emit_modal_open.set(true),
                        IconSignature { size: 16, color: "currentColor".to_string() }
                        span { "Emitir Documento" }
                    }
                }
            }

            if documents.is_empty() {
                div { class: "empty-tab-state",
                    IconFile { size: 48, color: "var(--text-muted, #8c8c8c)".to_string() }
                    p { class: "empty-state-text", "Nenhum contrato ou termo emitido para este paciente." }
                }
            } else {
                div { class: "patient-docs-table-wrapper",
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
                                    let pdf_url_to_preview = if let Some(ref s) = doc.signed_pdf_url {
                                        s.clone()
                                    } else {
                                        doc.original_pdf_url.clone()
                                    };
                                    let pdf_title = doc.title.clone();

                                    rsx! {
                                        tr { key: "{doc.id}",
                                            td {
                                                div { class: "doc-title-cell",
                                                    IconFile { size: 18, color: "#0052cc".to_string() }
                                                    span { class: "font-semibold", "{doc.title}" }
                                                }
                                            }
                                            td {
                                                span { class: "badge-doc-type", "{doc.document_type}" }
                                            }
                                            td { "{doc.created_at.chars().take(10).collect::<String>()}" }
                                            td {
                                                if doc.status == "signed" || doc.status == "completed" {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        span { " Assinado" }
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending",
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
                                                    button {
                                                        class: "btn-action-icon",
                                                        title: "Visualizar Documento / PDF",
                                                        onclick: {
                                                            let u = pdf_url_to_preview.clone();
                                                            let tit = pdf_title.clone();
                                                            let mut preview_sig = pdf_preview_target;
                                                            move |_| preview_sig.set(Some((u.clone(), tit.clone())))
                                                        },
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

            // Modal de Emissão de Documento
            if is_emit_modal_open() {
                div { class: "modal-overlay",
                    div { class: "action-modal doc-emit-modal",
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "Emitir Novo Documento / Contrato" }
                                p { class: "modal-subtitle", "Gere o termo para assinatura digital do paciente." }
                            }
                            button { class: "modal-close", onclick: move |_| is_emit_modal_open.set(false), "×" }
                        }
                        div { class: "modal-body",
                            div { class: "form-group",
                                label { class: "form-label", "Modelo de Contrato Base" }
                                select {
                                    class: "select-field",
                                    value: "{emit_template_id}",
                                    onchange: move |e| {
                                        let val = e.value();
                                        emit_template_id.set(val.clone());
                                        if let Some(t) = templates.iter().find(|tpl| tpl.id == val) {
                                            emit_doc_title.set(format!("{} - {}", t.title, pat_name));
                                        }
                                    },
                                    option { value: "", "Documento em Branco / Personalizado" }
                                    for tpl in &templates {
                                        option { value: "{tpl.id}", "{tpl.title}" }
                                    }
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Título do Documento *" }
                                input {
                                    r#type: "text",
                                    class: "input-field",
                                    placeholder: "Ex: Termo de Consentimento para Implante",
                                    value: "{emit_doc_title}",
                                    oninput: move |e| emit_doc_title.set(e.value())
                                }
                            }
                        }
                        div { class: "modal-footer",
                            button { class: "btn-secondary", onclick: move |_| is_emit_modal_open.set(false), "Cancelar" }
                            button {
                                class: "btn-primary",
                                disabled: is_emitting(),
                                onclick: move |e| handle_emit(e),
                                if is_emitting() { "Emitindo..." } else { "Gerar e Enviar para Assinatura" }
                            }
                        }
                    }
                }
            }

            // Modal de QR Code
            if let Some(ref doc) = *qr_modal_doc.read() {
                {
                    let link_url = format!("http://localhost:8080/sign/{}", doc.signing_token);
                    let qr_svg = generate_qr_svg(&link_url);
                    rsx! {
                        div { class: "modal-overlay",
                            div { class: "action-modal qr-modal-card",
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "Assinatura Digital via QR Code" }
                                        p { class: "modal-subtitle", "Aponte a câmera do celular ou acesse o link para assinar digitalmente." }
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
                                    p { class: "qr-hint", "O paciente poderá visualizar o contrato na íntegra, autenticar-se e assinar na tela do celular ou tablet." }

                                    div { class: "qr-link-copy-box",
                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            class: "input-field",
                                            value: "{link_url}",
                                        }
                                        a {
                                            href: "{link_url}",
                                            target: "_blank",
                                            class: "btn-secondary",
                                            IconExternalLink { size: 16, color: "#0052cc".to_string() }
                                            span { " Abrir Portal" }
                                        }
                                    }
                                }

                                div { class: "modal-footer",
                                    button { class: "btn-primary full-width", onclick: move |_| qr_modal_doc.set(None), "Concluir" }
                                }
                            }
                        }
                    }
                }
            }

            // Modal: Visualizador Nativo de PDF / Documento
            if let Some((ref url, ref title)) = *pdf_preview_target.read() {
                div { class: "modal-overlay",
                    onclick: move |_| pdf_preview_target.set(None),
                    div { class: "action-modal pdf-viewer-modal",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            div {
                                h2 { class: "modal-title", "{title}" }
                                p { class: "modal-subtitle", "Documento PDF Clínico" }
                            }
                            div { class: "modal-header-actions",
                                a {
                                    href: "{url}",
                                    target: "_blank",
                                    rel: "noopener noreferrer",
                                    class: "btn-secondary btn-sm",
                                    IconExternalLink { size: 14, color: "#0052cc".to_string() }
                                    span { " Abrir em Nova Aba" }
                                }
                                button {
                                    class: "modal-close",
                                    onclick: move |_| pdf_preview_target.set(None),
                                    "×"
                                }
                            }
                        }
                        div { class: "modal-body pdf-viewer-modal-body",
                            object {
                                data: "{url}#toolbar=1&view=FitH",
                                r#type: "application/pdf",
                                class: "pdf-modal-embed",
                                iframe {
                                    src: "{url}",
                                    class: "pdf-modal-embed",
                                    title: "{title}",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
