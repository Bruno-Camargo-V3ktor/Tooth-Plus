use crate::icons::{IconCheck, IconCopy, IconExternalLink, IconFileText, IconTrash};
use shared::documents::PatientDocument;
use dioxus::prelude::*;

#[component]
pub fn IssuedDocumentsTable(
    documents: Vec<PatientDocument>,
    on_preview: EventHandler<String>,
    on_qr_code: EventHandler<String>,
    on_copy_link: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    if documents.is_empty() {
        return rsx! {
            div { class: "empty-debits-box",
                div { class: "empty-debits-icon",
                    IconFileText { size: 48, color: "#475569".to_string() }
                }
                h3 { class: "empty-debits-title", "Nenhum documento emitido" }
                p { class: "empty-debits-desc", "Emita contratos, termos TCLE e atestados para seus pacientes com assinatura digital." }
            }
        };
    }

    rsx! {
        div { class: "patients-table-card",
            table { class: "patients-list-table",
                thead {
                    tr {
                        th { "Documento ⇣" }
                        th { "Paciente" }
                        th { "Dentista Responsável" }
                        th { "Data de Emissão" }
                        th { "Status" }
                        th { style: "text-align: right; padding-right: 20px;", "Ações" }
                    }
                }
                tbody {
                    for doc in documents {
                        {
                            let did = doc.id.clone();
                            let did_prev = doc.id.clone();
                            let did_qr = doc.id.clone();
                            let did_copy = doc.id.clone();
                            let did_del = doc.id.clone();

                            let is_signed = doc.status == "signed";
                            let p_name = doc.patient_name.clone().unwrap_or_else(|| "Paciente".to_string());
                            let doc_name = doc.doctor_user_name.clone().unwrap_or_else(|| "Dr. Lucas Mendes".to_string());
                            let date_fmt = doc.created_at.split('T').next().unwrap_or(&doc.created_at);

                            let badge_cls = if is_signed { "badge badge-green" } else { "badge badge-yellow" };
                            let badge_text = if is_signed { "Assinado" } else { "Pendente Assinatura" };

                            rsx! {
                                tr { key: "{doc.id}", class: "patient-table-row",
                                    td {
                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            IconFileText { size: 16, color: if is_signed { "#22c55e".to_string() } else { "#38bdf8".to_string() } }
                                            strong { style: "color: #f1f5f9; font-size: 13.5px;", "{doc.title}" }
                                        }
                                    }
                                    td {
                                        span { class: "patient-name-link", "{p_name}" }
                                    }
                                    td { "{doc_name}" }
                                    td { "{date_fmt}" }
                                    td {
                                        span { class: "{badge_cls}", "{badge_text}" }
                                    }
                                    td {
                                        div { style: "display: flex; align-items: center; justify-content: flex-end; gap: 6px;",
                                            if !is_signed {
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Assinar via QR Code (Celular / Tablet)",
                                                    onclick: move |_| on_qr_code.call(did_qr.clone()),
                                                    span { "📱" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "action-btn-icon",
                                                    title: "Copiar Link de Assinatura",
                                                    onclick: move |_| on_copy_link.call(did_copy.clone()),
                                                    IconCopy { size: 14, color: "#38bdf8".to_string() }
                                                }
                                            } else {
                                                span { style: "color: #22c55e; margin-right: 4px;",
                                                    IconCheck { size: 15, color: "#22c55e".to_string() }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Visualizar Folha A4 / Imprimir",
                                                onclick: move |_| on_preview.call(did_prev.clone()),
                                                IconExternalLink { size: 14, color: "#94a3b8".to_string() }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Excluir Documento",
                                                onclick: move |_| on_delete.call(did_del.clone()),
                                                IconTrash { size: 14, color: "#ef4444".to_string() }
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
