//! # Módulo de Visualização e Gestão de Documentos Digitais (Frontend)
//!
//! Agrega sub-módulos para controle e emissão de contratos odontológicos, termos
//! de consentimento (TCLE), modelos com tags dinâmicas e auditoria de assinaturas eletrônicas.

pub mod documents_list;
pub mod issue_modal;
pub mod template_editor_modal;
pub mod templates_list;

pub use documents_list::*;
pub use issue_modal::*;
pub use template_editor_modal::*;
pub use templates_list::*;

use crate::api::{fetch_documents, fetch_patients, fetch_users};
use crate::components::icons::{IconExternalLink, IconFile, IconSignature};
use crate::permissions;
use crate::utils::{build_signing_url, resolve_file_url};
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;
use shared::documents::{ContractTemplate, DocumentsKpis, PatientDocument};

/// Gera o SVG do QR Code para um link de assinatura digital.
pub fn generate_qr_svg(url: &str) -> String {
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

/// Formata uma data ISO para o padrão brasileiro DD/MM/AAAA.
pub fn format_br_date(date_str: &str) -> String {
    let clean = date_str.chars().take(10).collect::<String>();
    let parts: Vec<&str> = clean.split('-').collect();
    if parts.len() == 3 {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        clean
    }
}

/// Componente principal da tela de Gestão de Documentos e Assinaturas Digitais.
#[component]
pub fn DocumentsView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "documents:read");
    let can_write = permissions::has_permission(&sess, &clinic, "documents:write");
    let can_delete = permissions::has_permission(&sess, &clinic, "documents:delete");

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();
    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar os documentos desta unidade." }
            }
        };
    }

    let mut active_main_tab = use_signal(|| "emitted".to_string());
    let search_query = use_signal(String::new);
    let status_filter = use_signal(|| "all".to_string());
    let mut reload_trigger = use_signal(|| 0usize);

    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    let mut pdf_preview_target = use_signal(|| None::<(String, String)>);
    let mut qr_modal_doc = use_signal(|| None::<PatientDocument>);

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();
    let documents_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        let st = status_filter();
        let _ = reload_trigger();
        let st_opt = if st == "all" { None } else { Some(st) };

        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(shared::documents::DocumentsListResponse {
                    documents: vec![],
                    templates: vec![],
                    kpis: DocumentsKpis::default(),
                });
            }
            let st_ref = st_opt.as_deref();
            fetch_documents(&t, &cid, None, st_ref).await
        }
    });

    let tok_pat = token.clone();
    let cid_pat = clinic_id.clone();
    let patients_resource = use_resource(move || {
        let t = tok_pat.clone();
        let cid = cid_pat.clone();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(shared::patients::PatientListResponse {
                    items: vec![],
                    kpis: shared::patients::PatientKpis::default(),
                    total: 0,
                });
            }
            fetch_patients(&t, &cid, None).await
        }
    });

    let tok_usr = token.clone();
    let cid_usr = clinic_id.clone();
    let users_resource = use_resource(move || {
        let t = tok_usr.clone();
        let cid = cid_usr.clone();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(vec![]);
            }
            fetch_users(&t, &cid).await
        }
    });

    let (documents_list, templates_list, kpis, is_loading) = match &*documents_resource.read() {
        Some(Ok(resp)) => (
            resp.documents.clone(),
            resp.templates.clone(),
            resp.kpis.clone(),
            false,
        ),
        Some(Err(_)) => (vec![], vec![], DocumentsKpis::default(), false),
        None => (vec![], vec![], DocumentsKpis::default(), true),
    };

    let patients_list = match &*patients_resource.read() {
        Some(Ok(resp)) => resp.items.clone(),
        _ => vec![],
    };

    let users_list = match &*users_resource.read() {
        Some(Ok(resp)) => resp.clone(),
        _ => vec![],
    };

    let mut is_issue_modal_open = use_signal(|| false);
    let mut is_template_modal_open = use_signal(|| false);
    let mut editing_template = use_signal(|| None::<ContractTemplate>);

    rsx! {
        div { class: "documents-view-container",
            if let Some(ref toast) = *toast_msg.read() {
                div { class: "toast toast-success",
                    span { "{toast}" }
                    button { class: "toast-close", onclick: move |_| toast_msg.set(None), "✕" }
                }
            }

            if let Some(ref err) = *error_toast.read() {
                div { class: "toast toast-error",
                    span { "{err}" }
                    button { class: "toast-close", onclick: move |_| error_toast.set(None), "✕" }
                }
            }

            // Main Tabs Switcher
            div { class: "documents-tab-bar",
                button {
                    class: if active_main_tab() == "emitted" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_main_tab.set("emitted".to_string()),
                    IconFile { size: 16, color: "currentColor".to_string() }
                    span { " Documentos & Contratos Emitidos ({documents_list.len()})" }
                }
                button {
                    class: if active_main_tab() == "templates" { "doc-main-tab active" } else { "doc-main-tab" },
                    onclick: move |_| active_main_tab.set("templates".to_string()),
                    IconSignature { size: 16, color: "currentColor".to_string() }
                    span { " Modelos de Contratos & E-Sign ({templates_list.len()})" }
                }
            }

            if active_main_tab() == "emitted" {
                DocumentsListSection {
                    documents: documents_list.clone(),
                    kpis: kpis.clone(),
                    is_loading,
                    search_query,
                    status_filter,
                    can_write,
                    can_delete,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    on_open_issue_modal: move |_| is_issue_modal_open.set(true),
                    reload_trigger,
                    toast_msg,
                    error_toast,
                    pdf_preview_target,
                    qr_modal_doc,
                }
            } else {
                TemplatesListSection {
                    templates: templates_list.clone(),
                    is_loading,
                    can_write,
                    can_delete,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    on_open_create_template: move |_| {
                        editing_template.set(None);
                        is_template_modal_open.set(true);
                    },
                    on_edit_template: move |tpl: ContractTemplate| {
                        editing_template.set(Some(tpl));
                        is_template_modal_open.set(true);
                    },
                    reload_trigger,
                    toast_msg,
                    error_toast,
                    pdf_preview_target,
                }
            }

            // Modal: Emitir Novo Contrato / Documento
            if is_issue_modal_open() {
                IssueDocumentModal {
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    is_open: is_issue_modal_open,
                    templates: templates_list.clone(),
                    patients: patients_list.clone(),
                    users: users_list.clone(),
                    reload_trigger,
                    toast_msg,
                    error_toast,
                    qr_modal_doc,
                }
            }

            // Modal: Editor de Modelo de Contrato
            if is_template_modal_open() {
                TemplateEditorModal {
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    editing_template: editing_template(),
                    is_open: is_template_modal_open,
                    reload_trigger,
                    toast_msg,
                    error_toast,
                }
            }

            // Modal: QR Code de Assinatura
            if let Some(ref doc) = *qr_modal_doc.read() {
                {
                    let link_url = build_signing_url(&doc.signing_token);
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
                                    p { class: "qr-hint", "O paciente poderá visualizar o contrato na íntegra, autenticar-se e desenhar a assinatura na tela do celular ou tablet." }

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
