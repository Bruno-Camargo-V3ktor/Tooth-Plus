//! # Aba de Contratos e Documentos Digitais do Paciente (Frontend)
//!
//! Controla a listagem de termos e contratos emitidos para o paciente,
//! emissão de novos documentos com modal completo idêntico ao módulo de documentos,
//! QR Code e pré-visualização de PDF.

use crate::components::icons::{
    IconCheckCircle, IconExternalLink, IconEye, IconFile, IconQrCode, IconSignature, IconWhatsApp,
};
use crate::pages::views::documents::issue_modal::IssueDocumentModal;
use crate::utils::{build_signing_url, resolve_file_url};
use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;
use shared::documents::{ContractTemplate, PatientDocument};
use shared::patients::Patient;
use shared::users::UserResponse;

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

/// Formata o tipo de documento técnico para exibição em português.
fn format_doc_type(doc_type: &str) -> &'static str {
    match doc_type.to_lowercase().as_str() {
        "consent" => "Consentimento (TCLE)",
        "contract" => "Contrato de Serviços",
        "orthodontics" => "Ortodontia / Alinhadores",
        "implant" => "Implantodontia / Cirurgia",
        "prescription" => "Receituário / Atestado",
        "budget" => "Orçamento",
        _ => "Documento Clínico",
    }
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
    #[props(default)]
    users: Vec<UserResponse>,
    #[props(default)]
    patients: Vec<Patient>,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
    is_emit_modal_open: Signal<bool>,
) -> Element {
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);
    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);

    let pat_phone_str = patient_phone.clone().unwrap_or_default();

    rsx! {
        div { class: "patient-tab-content",
            // Header da Aba
            div { class: "tab-header-actions-row",
                div { class: "tab-header-title-group",
                    h3 { class: "tab-header-title", "Contratos & Termos do Paciente" }
                    p { class: "tab-header-desc", "Documentos emitidos, assinaturas eletrônicas e termos arquivados." }
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
                                th { "Assinatura do Paciente" }
                                th { "Assinatura Médica" }
                                th { "Status Geral" }
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

                                    let is_anamnesis = doc.document_type == "anamnesis" || doc.document_type == "anamnese" || doc.title.to_lowercase().contains("anamnes");
                                    let patient_completed = !doc.requires_patient_signature || doc.patient_signed_at.is_some();
                                    let doctor_completed = !doc.requires_doctor_signature || doc.doctor_signed_at.is_some();
                                    let has_any_sign = doc.patient_signed_at.is_some() || doc.doctor_signed_at.is_some();
                                    let is_signed = doc.status == "signed"
                                        || doc.status == "completed"
                                        || (patient_completed && doctor_completed && (doc.requires_patient_signature || doc.requires_doctor_signature) && has_any_sign);

                                    rsx! {
                                        tr { key: "{doc.id}",
                                            td {
                                                div { class: "flex items-center gap-2",
                                                    IconFile { size: 16, color: "#0052cc".to_string() }
                                                    span { class: "font-semibold", "{doc.title}" }
                                                }
                                            }
                                            td {
                                                span { class: "badge-outline", "{format_doc_type(&doc.document_type)}" }
                                            }
                                            td { "{doc.created_at.chars().take(10).collect::<String>()}" }
                                            td {
                                                if !doc.requires_patient_signature {
                                                    span { class: "badge-status-neutral", "Não Exigida" }
                                                } else if doc.patient_signed_at.is_some() {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        span { " Assinado" }
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending", "Pendente" }
                                                }
                                            }
                                            td {
                                                if !doc.requires_doctor_signature {
                                                    span { class: "badge-status-neutral", "Não Exigida" }
                                                } else if doc.doctor_signed_at.is_some() {
                                                    span { class: "badge-status-completed",
                                                        IconCheckCircle { size: 14, color: "#10b981".to_string() }
                                                        span { " Assinado" }
                                                    }
                                                } else {
                                                    span { class: "badge-status-pending", "Pendente" }
                                                }
                                            }
                                            td {
                                                if is_signed {
                                                    span { class: "badge-status-completed", "Concluído (Válido)" }
                                                } else {
                                                    span { class: "badge-status-pending", "Aguardando E-Sign" }
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

                                                    if !is_anamnesis && !pdf_url_to_preview.is_empty() {
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

            // Modal de Emissão de Novo Documento (Reutiliza o Modal Completo do Módulo de Documentos)
            if is_emit_modal_open() {
                IssueDocumentModal {
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    is_open: is_emit_modal_open,
                    templates: templates.clone(),
                    patients: patients.clone(),
                    users: users.clone(),
                    reload_trigger: use_signal(|| 0usize),
                    toast_msg,
                    error_toast,
                    qr_modal_doc,
                    preselected_patient_id: Some(patient_id.clone()),
                    preselected_patient_name: Some(patient_name.clone()),
                    on_document_created: Some(reload_patient_details.clone()),
                }
            }

            // Modal do QR Code & Envio WhatsApp
            if let Some(ref doc) = *qr_modal_doc.read() {
                {
                    let link_url = build_signing_url(&doc.signing_token);
                    let qr_svg = generate_qr_svg(&link_url);
                    let wa_clean_phone: String = pat_phone_str.chars().filter(|c| c.is_ascii_digit()).collect();
                    let wa_phone_formatted = if !wa_clean_phone.starts_with("55") && wa_clean_phone.len() >= 10 {
                        format!("55{}", wa_clean_phone)
                    } else {
                        wa_clean_phone
                    };

                    let msg_text = format!(
                        "Olá, {}! Por favor, assine seu documento '{}' através do link seguro: {}",
                        patient_name,
                        doc.title,
                        link_url
                    );
                    let wa_href = format!("https://wa.me/{}?text={}", wa_phone_formatted, simple_url_encode(&msg_text));

                    rsx! {
                        div { class: "modal-overlay",
                            div { class: "action-modal qr-modal-card",
                                div { class: "modal-header",
                                    div {
                                        h2 { class: "modal-title", "Assinatura Digital via QR Code" }
                                        p { class: "modal-subtitle", "Aponte a câmera do celular ou compartilhe via WhatsApp." }
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
                                    p { class: "qr-hint", "O paciente poderá ler as cláusulas e assinar diretamente na tela touch." }

                                    div { class: "qr-link-copy-box",
                                        input {
                                            r#type: "text",
                                            readonly: true,
                                            class: "input-field font-mono font-xs",
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

                                    if !pat_phone_str.is_empty() {
                                        div { class: "qr-whatsapp-share-box mt-3",
                                            a {
                                                href: "{wa_href}",
                                                target: "_blank",
                                                class: "btn-whatsapp-share",
                                                IconWhatsApp { size: 16, color: "#ffffff".to_string() }
                                                span { " Enviar Link no WhatsApp ({pat_phone_str})" }
                                            }
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
                {
                    let resolved_url = resolve_file_url(url);
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
                                    div { class: "modal-header-actions",
                                        a {
                                            href: "{resolved_url}",
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
                                        data: "{resolved_url}#toolbar=1&view=FitH",
                                        r#type: "application/pdf",
                                        class: "pdf-modal-embed",
                                        iframe {
                                            src: "{resolved_url}",
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
    }
}
