//! # Módulo de Documentos, Contratos e Recibos (Tooth Plus V2)
//!
//! Catálogo de modelos da clínica, geração de contratos de tratamento com assinatura física/digital,
//! atestados, anamnese para impressão e recibos fiscais.

use crate::api::mock_db::DB;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::{IconFileText, IconSearch};
use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/documents/style.css");

#[derive(Clone, PartialEq, Debug)]
struct DocumentTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}

#[component]
pub fn DocumentsView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let clinic_name = active_clinic
        .read()
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Clínica SmilePlus Odontologia".to_string());

    let templates = vec![
        DocumentTemplate {
            id: "contrato_tratamento",
            title: "Contrato de Prestação de Serviços Odontológicos",
            category: "Contratos",
            description: "Termo padrão de contratação de procedimentos e planos de tratamento com cláusulas de pagamento e responsabilidades.",
            icon: "📑",
        },
        DocumentTemplate {
            id: "termo_consentimento",
            title: "Termo de Consentimento Livre e Esclarecido (TCLE)",
            category: "Termos Clínicos",
            description: "Esclarecimento de riscos, benefícios e alternativas do procedimento cirúrgico, endodôntico ou protético.",
            icon: "✍️",
        },
        DocumentTemplate {
            id: "atestado_odontologico",
            title: "Atestado Odontológico",
            category: "Declarações",
            description: "Emissão de atestado de comparecimento ou dispensa de atividades com justificativa clínica e horário.",
            icon: "🩺",
        },
        DocumentTemplate {
            id: "recibo_pagamento",
            title: "Recibo de Pagamento de Procedimento",
            category: "Financeiro",
            description: "Comprovante formal de quitação de tratamento com dados do paciente, CPF, valor em reais e assinatura profissional.",
            icon: "🧾",
        },
        DocumentTemplate {
            id: "anamnese_impressao",
            title: "Ficha de Anamnese Médica (Física)",
            category: "Prontuário",
            description: "Questionário de saúde impresso para preenchimento manual e validação por assinatura do paciente na recepção.",
            icon: "📋",
        },
    ];

    let mut selected_template = use_signal(|| Option::<DocumentTemplate>::None);
    let mut selected_patient_name = use_signal(|| "Maria Oliveira da Silva".to_string());
    let mut selected_patient_cpf = use_signal(|| "123.456.789-00".to_string());

    let patients_options = use_signal(|| {
        if let Ok(db) = DB.lock() {
            db.patients
                .iter()
                .map(|p| (p.full_name.clone(), p.document_cpf.clone().unwrap_or_default()))
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    });

    let today_display = {
        let (y, m, d) = (2026, 8, 25);
        format!("{:02}/08/{}", d, y)
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "documents-page",

            // Header & Informações
            div { style: "display: flex; flex-direction: column; gap: 4px;",
                h1 { style: "font-size: 22px; font-weight: 800; color: #0f172a; margin: 0;", "Central de Documentos & Contratos" }
                p { style: "font-size: 13.5px; color: #64748b; margin: 0;",
                    "Emita contratos de tratamentos, atestados, termos de consentimento e recibos com preenchimento automático para impressão ou assinatura física."
                }
            }

            // Grade de Modelos de Documento
            div { class: "doc-templates-grid",
                for tmpl in templates.iter() {
                    {
                        let t = tmpl.clone();
                        let mut sel = selected_template.clone();
                        rsx! {
                            div {
                                key: "{t.id}",
                                class: "doc-template-card",
                                div { class: "doc-card-header",
                                    div { class: "doc-icon-badge", "{t.icon}" }
                                    div {
                                        h3 { class: "doc-card-title", "{t.title}" }
                                        span { class: "badge badge-blue", style: "margin-top: 4px;", "{t.category}" }
                                    }
                                }
                                p { class: "doc-card-desc", "{t.description}" }
                                div { class: "doc-card-footer",
                                    span { style: "font-size: 11.5px; color: #94a3b8;", "Modelo Oficial" }
                                    button {
                                        class: "btn-emit-doc",
                                        onclick: move |_| sel.set(Some(t.clone())),
                                        "Gerar Documento"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal de Visualização / Impressão do Documento
            if let Some(doc) = selected_template.read().clone() {
                div { class: "modal-overlay",
                    onclick: move |_| selected_template.set(None),

                    div { class: "modal-box modal-lg", onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            span { class: "modal-title", "{doc.title}" }
                            button { class: "modal-close-btn", onclick: move |_| selected_template.set(None), "✕" }
                        }

                        div { class: "modal-body", style: "background: #0f172a; padding: 24px;",

                            // Seletor de Paciente para o documento
                            div { class: "form-field", style: "margin-bottom: 16px;",
                                label { class: "form-label", "Selecione o Paciente para preenchimento:" }
                                select {
                                    class: "form-select",
                                    onchange: move |e| {
                                        let name = e.value();
                                        if let Some((_, cpf)) = patients_options.read().iter().find(|(n, _)| n == &name) {
                                            selected_patient_name.set(name);
                                            selected_patient_cpf.set(cpf.clone());
                                        }
                                    },
                                    for (pname, _) in patients_options.read().iter() {
                                        option { value: "{pname}", "{pname}" }
                                    }
                                }
                            }

                            // Folha de Visualização do Documento (A4)
                            div { class: "doc-paper-preview",
                                div { class: "doc-paper-header",
                                    div { class: "doc-paper-clinic-name", "{clinic_name}" }
                                    div { class: "doc-paper-clinic-meta", "Odontologia Especializada • CNPJ 12.345.678/0001-90 • CRO-SP Jurídico 4920" }
                                    div { class: "doc-paper-clinic-meta", "Av. Paulista, 1000, Cj. 120 - São Paulo/SP • (11) 3333-4444" }
                                    h2 { class: "doc-paper-title", "{doc.title}" }
                                }

                                div { class: "doc-paper-body",
                                    p {
                                        "Pelo presente instrumento particular, a clínica ", strong { "{clinic_name}" },
                                        " presta os serviços odontológicos ao(à) paciente ", strong { "{selected_patient_name}" },
                                        ", portador(a) do CPF nº ", strong { "{selected_patient_cpf}" }, ", conforme plano terapêutico acordado."
                                    }
                                    p {
                                        "O(A) paciente declara ter sido devidamente informado(a) e esclarecido(a) quanto aos objetivos, riscos clínicos inerentes, cuidados pós-operatórios e valores estipulados para a execução do tratamento odontológico."
                                    }
                                    p {
                                        "E por estarem de pleno acordo, firmam o presente termo para que produza os devidos efeitos legais e clínicos."
                                    }
                                    p { style: "margin-top: 24px; text-align: right;",
                                        "São Paulo/SP, {today_display}."
                                    }
                                }

                                div { class: "doc-paper-signatures",
                                    div { class: "doc-sign-line",
                                        div { strong { "{selected_patient_name}" } }
                                        div { "Paciente / Responsável Legal" }
                                    }
                                    div { class: "doc-sign-line",
                                        div { strong { "Dr. Roberto Alencar" } }
                                        div { "Cirurgião-Dentista • CRO-SP 84920" }
                                    }
                                }
                            }
                        }

                        div { class: "modal-footer",
                            button { class: "btn-modal-ghost", onclick: move |_| selected_template.set(None), "Fechar" }
                            button {
                                class: "btn-modal-primary",
                                onclick: move |_| {
                                    let _ = js_sys::eval("window.print()");
                                    toast.show("Comando de impressão acionado.", ToastVariant::Success);
                                },
                                "🖨 Imprimir / Salvar PDF"
                            }
                        }
                    }
                }
            }
        }
    }
}
