//! # Visualizador de Documento e Prévia de Cláusulas (Frontend)
//!
//! Exibe os dados do documento (título, clínica emissora, data, participantes)
//! e o visualizador embutido do PDF ou texto do contrato para conferência antes da assinatura.

use crate::components::icons::{IconCheckCircle, IconExternalLink, IconLock};
use dioxus::prelude::*;
use shared::documents::PublicSigningDocumentResponse;

/// Componente de cabeçalho e pré-visualização do contrato no portal de assinaturas.
#[component]
pub fn DocumentViewerCard(doc: PublicSigningDocumentResponse) -> Element {
    let patient_signed = doc.document.patient_signed_at.is_some();
    let doctor_signed = doc.document.doctor_signed_at.is_some();

    rsx! {
        div { class: "portal-doc-header",
            div { class: "portal-doc-info-left",
                div { class: "portal-doc-badge", "{doc.document.document_type}" }
                h2 { class: "portal-doc-title", "{doc.document.title}" }
                p { class: "portal-doc-meta", "Emitido em: {doc.document.created_at.chars().take(10).collect::<String>()}" }
            }
            if !doc.document.original_pdf_url.is_empty() {
                a {
                    class: "portal-btn-secondary",
                    href: "{doc.document.original_pdf_url}",
                    target: "_blank",
                    IconExternalLink { size: 14, color: "currentColor".to_string() }
                    span { "Abrir PDF" }
                }
            }
        }

        if !doc.document.original_pdf_url.is_empty() {
            div { class: "portal-preview-frame",
                iframe {
                    class: "portal-pdf-embed",
                    src: "{doc.document.original_pdf_url}#toolbar=0&navpanes=0",
                    title: "{doc.document.title}"
                }
            }
        }

        div { class: "portal-signers-status",
            h4 { class: "portal-signers-title", "Status dos Signatários" }
            div { class: "portal-signers-grid",
                div { class: if patient_signed { "signer-card signed" } else { "signer-card pending" },
                    div { class: "signer-card-header",
                        span { class: "signer-role", "Paciente" }
                        if patient_signed {
                            span { class: "signer-badge-ok",
                                IconCheckCircle { size: 12, color: "currentColor".to_string() }
                                span { "Assinado" }
                            }
                        } else {
                            span { class: "signer-badge-wait",
                                IconLock { size: 12, color: "currentColor".to_string() }
                                span { "Pendente" }
                            }
                        }
                    }
                    p { class: "signer-name", "{doc.document.patient_name.as_deref().unwrap_or(\"Paciente\")}" }
                }

                div { class: if doctor_signed { "signer-card signed" } else { "signer-card pending" },
                    div { class: "signer-card-header",
                        span { class: "signer-role", "Dentista Responsável" }
                        if doctor_signed {
                            span { class: "signer-badge-ok",
                                IconCheckCircle { size: 12, color: "currentColor".to_string() }
                                span { "Assinado" }
                            }
                        } else {
                            span { class: "signer-badge-wait",
                                IconLock { size: 12, color: "currentColor".to_string() }
                                span { "Pendente" }
                            }
                        }
                    }
                    p { class: "signer-name", "{doc.document.doctor_user_name.as_deref().unwrap_or(\"Responsável Técnico\")}" }
                }
            }
        }
    }
}
