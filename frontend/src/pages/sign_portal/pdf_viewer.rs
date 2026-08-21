//! # Visualizador de Documento e Prévia de Cláusulas (Frontend)
//!
//! Exibe os dados do documento (título, clínica emissora, data, participantes)
//! e o visualizador embutido do PDF ou texto do contrato para conferência antes da assinatura.

use crate::components::icons::{IconCheckCircle, IconExternalLink, IconHeartPulse, IconLock};
use crate::utils::resolve_file_url;
use dioxus::prelude::*;
use shared::documents::PublicSigningDocumentResponse;

/// Componente de cabeçalho e visualizador de declaração/contrato no portal de assinaturas.
#[component]
pub fn DocumentViewerCard(doc: PublicSigningDocumentResponse) -> Element {
    let patient_signed = doc.document.patient_signed_at.is_some();
    let doctor_signed = doc.document.doctor_signed_at.is_some();
    let is_anamnesis = doc.document.document_type == "anamnesis" || doc.anamnesis.is_some();
    let pdf_url = resolve_file_url(&doc.document.original_pdf_url);
    let show_patient_card = doc.requires_patient_signature || patient_signed;
    let show_doctor_card = doc.requires_doctor_signature || doctor_signed;

    rsx! {
        div { class: "portal-doc-header",
            div { class: "portal-doc-info-left",
                div { class: "portal-doc-badge",
                    if is_anamnesis { "Ficha de Anamnese" } else { "{doc.document.document_type}" }
                }
                h2 { class: "portal-doc-title", "{doc.document.title}" }
                p { class: "portal-doc-meta", "Emitido em: {doc.document.created_at.chars().take(10).collect::<String>()}" }
            }
            if !is_anamnesis && !pdf_url.is_empty() {
                a {
                    class: "portal-btn-secondary",
                    href: "{pdf_url}",
                    target: "_blank",
                    IconExternalLink { size: 14, color: "currentColor".to_string() }
                    span { "Abrir PDF" }
                }
            }
        }

        // Se for Anamnese: Renderiza o prontuário com perguntas e respostas conferidas pelo paciente
        if is_anamnesis {
            if let Some(ref anam) = doc.anamnesis {
                div { class: "portal-anamnesis-container", style: "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 14px; overflow: hidden; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.04);",
                    // Header do Prontuário
                    div { style: "padding: 14px 18px; background: #f8fafc; border-bottom: 1px solid #e2e8f0; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 10px;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            IconHeartPulse { size: 18, color: "#0052cc".to_string() }
                            span { style: "font-size: 13.5px; font-weight: 700; color: #0f172a; letter-spacing: 0.3px;", "RESUMO DAS RESPOSTAS CLÍNICAS" }
                        }
                        span { class: "badge-insurance-plan font-mono font-xs",
                            if anam.template_type.as_deref() == Some("minor") { "Ficha Odontopediátrica" } else { "Ficha Adulto" }
                        }
                    }

                    // Destaques Clínicos (se preenchidos)
                    if !anam.allergies.is_empty() || anam.continuous_medications.is_some() || !anam.systemic_diseases.is_empty() {
                        div { style: "padding: 12px 18px; background: #fffbeb; border-bottom: 1px solid #fde68a; display: flex; flex-direction: column; gap: 6px;",
                            if !anam.allergies.is_empty() {
                                div { style: "display: flex; gap: 8px; font-size: 12.5px;",
                                    strong { style: "color: #991b1b; min-width: 80px;", "Alergias:" }
                                    span { style: "color: #b91c1c; font-weight: 600;", "{anam.allergies.join(\", \")}" }
                                }
                            }
                            if let Some(ref meds) = anam.continuous_medications {
                                if !meds.is_empty() {
                                    div { style: "display: flex; gap: 8px; font-size: 12.5px;",
                                        strong { style: "color: #92400e; min-width: 80px;", "Medicações:" }
                                        span { style: "color: #b45309; font-weight: 600;", "{meds}" }
                                    }
                                }
                            }
                            if !anam.systemic_diseases.is_empty() {
                                div { style: "display: flex; gap: 8px; font-size: 12.5px;",
                                    strong { style: "color: #92400e; min-width: 80px;", "Saúde Geral:" }
                                    span { style: "color: #b45309; font-weight: 600;", "{anam.systemic_diseases.join(\", \")}" }
                                }
                            }
                        }
                    }

                    // Lista de Perguntas e Respostas
                    div { style: "max-height: 520px; overflow-y: auto; padding: 14px 18px;",
                        div { style: "display: flex; flex-direction: column; gap: 10px;",
                            for (idx, item) in anam.custom_responses.iter().enumerate() {
                                {
                                    let is_yes_no = item.question_type == "yes_no";
                                    let is_yes = item.answer_boolean == Some(true);
                                    let is_no = item.answer_boolean == Some(false);
                                    let text_ans = item.answer_text.as_deref().unwrap_or("");
                                    let notes = item.notes.as_deref().unwrap_or("");

                                    rsx! {
                                        div { key: "{item.question_id}", style: "padding: 10px 14px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; display: flex; justify-content: space-between; align-items: center; gap: 14px;",
                                            div { style: "flex: 1;",
                                                span { style: "font-size: 10.5px; text-transform: uppercase; color: #64748b; font-weight: 700; letter-spacing: 0.5px;", "{item.category}" }
                                                p { style: "margin: 2px 0 0 0; font-size: 13px; font-weight: 600; color: #1e293b; line-height: 1.35;", "#{idx + 1}. {item.question_text}" }
                                                if is_yes && !notes.is_empty() {
                                                    p { style: "margin: 4px 0 0 0; font-size: 12px; color: #dc2626; background: #fef2f2; padding: 3px 8px; border-radius: 4px; border: 1px solid #fee2e2;",
                                                        strong { "Detalhes: " }
                                                        "{notes}"
                                                    }
                                                }
                                            }
                                            div { style: "flex-shrink: 0;",
                                                if is_yes_no {
                                                    if is_yes {
                                                        span { style: "font-size: 11.5px; font-weight: 700; padding: 4px 12px; border-radius: 20px; background: #fee2e2; color: #dc2626; border: 1px solid #fca5a5;", "SIM" }
                                                    } else if is_no {
                                                        span { style: "font-size: 11.5px; font-weight: 700; padding: 4px 12px; border-radius: 20px; background: #eff6ff; color: #0052cc; border: 1px solid #bfdbfe;", "NÃO" }
                                                    } else {
                                                        span { style: "font-size: 11.5px; color: #94a3b8;", "—" }
                                                    }
                                                } else {
                                                    span { style: "font-size: 12px; font-weight: 600; color: #0f172a; background: #ffffff; padding: 4px 10px; border-radius: 6px; border: 1px solid #cbd5e1;",
                                                        if !text_ans.is_empty() { "{text_ans}" } else { "Não informado" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Termo Legal de Veracidade das Informações
                    div { style: "padding: 12px 18px; background: #f1f5f9; border-top: 1px solid #e2e8f0; font-size: 11.5px; color: #475569; line-height: 1.4;",
                        "Declaro sob a fé do meu grau que todas as informações e respostas acima são a expressão da verdade e que não omiti nenhum fato sobre meu estado de saúde ou tratamentos anteriores."
                    }
                }
            } else {
                div { style: "padding: 24px; text-align: center; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 12px; margin-bottom: 20px;",
                    p { class: "text-muted font-xs", "Carregando questionário de anamnese do paciente..." }
                }
            }
        } else if !pdf_url.is_empty() {
            div { class: "portal-preview-frame",
                iframe {
                    class: "portal-pdf-embed",
                    src: "{pdf_url}#toolbar=0&navpanes=0",
                    title: "{doc.document.title}"
                }
            }
        }


        if show_patient_card || show_doctor_card {
            div { class: "portal-signers-status",
                h4 { class: "portal-signers-title", "Status dos Signatários" }
                div { class: "portal-signers-grid",
                    if show_patient_card {
                        div { class: if patient_signed { "signer-card signed" } else { "signer-card pending" },
                            div { class: "signer-card-header",
                                span { class: "signer-role", "Paciente / Responsável" }
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
                    }

                    if show_doctor_card {
                        div { class: if doctor_signed { "signer-card signed" } else { "signer-card pending" },
                            div { class: "signer-card-header",
                                span { class: "signer-role", "Cirurgião-Dentista" }
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
                            p { class: "signer-name",
                                if let Some(ref doc_name) = doc.document.doctor_user_name {
                                    "{doc_name}"
                                } else if doc.allow_any_dentist_signature {
                                    "Qualquer Dentista da Clínica"
                                } else {
                                    "Responsável Técnico"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
