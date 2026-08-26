pub mod components;

use crate::api::ActiveClinicState;
use dioxus::prelude::*;

pub use components::{DocTemplate, PaperPreview, TemplateGrid};

const STYLE: Asset = asset!("/src/pages/documents/style.css");

#[component]
pub fn DocumentsView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();

    let clinic_name = active_clinic
        .read()
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "SmilePlus Odontologia".to_string());

    let mut selected_template = use_signal(|| None::<&'static str>);
    let patient_name = use_signal(|| "Mariana Castro Fernandes".to_string());
    let doctor_name = use_signal(|| "Dr. Roberto Alencar - CRO-SP 84920".to_string());

    let templates = vec![
        DocTemplate {
            id: "contrato",
            title: "Contrato de Prestação de Serviços",
            desc: "Contrato padrão para tratamentos odontológicos com cláusulas de valores, prazos e direitos.",
            icon: "📜",
            category: "Jurídico / Contratos",
        },
        DocTemplate {
            id: "tcle",
            title: "Termo de Consentimento (TCLE)",
            desc: "Consentimento informado do paciente sobre riscos, procedimentos e recomendações clínicas.",
            icon: "✍️",
            category: "Termos & Autorizações",
        },
        DocTemplate {
            id: "atestado",
            title: "Atestado Odontológico",
            desc: "Declaração de comparecimento e atestado de repouso para procedimentos executados.",
            icon: "🏥",
            category: "Atestados & Declarações",
        },
        DocTemplate {
            id: "recibo",
            title: "Recibo de Pagamento Odontológico",
            desc: "Comprovante financeiro detalhado de quitação de procedimentos para o paciente.",
            icon: "🧾",
            category: "Financeiro / Recibos",
        },
    ];

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "documents-page",
            if let Some(tpl_id) = selected_template() {
                PaperPreview {
                    template_id: tpl_id,
                    clinic_name,
                    patient_name,
                    doctor_name,
                    on_back: move |_| selected_template.set(None),
                }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 4px;",
                    h1 { style: "font-size: 22px; font-weight: 800; color: #f8fafc; margin: 0;", "Documentos & Modelos Oficiais" }
                    p { style: "font-size: 13.5px; color: #94a3b8; margin: 0 0 8px 0;",
                        "Emita contratos, termos de consentimento, atestados e recibos personalizados para impressão ou assinatura física."
                    }
                }

                TemplateGrid {
                    templates: templates.clone(),
                    on_select: move |tid| selected_template.set(Some(tid)),
                }
            }
        }
    }
}
