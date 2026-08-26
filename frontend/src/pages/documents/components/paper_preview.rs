use crate::icons::IconPrinter;
use dioxus::prelude::*;

#[component]
pub fn PaperPreview(
    template_id: &'static str,
    clinic_name: String,
    patient_name: Signal<String>,
    doctor_name: Signal<String>,
    on_back: EventHandler<()>,
) -> Element {
    let (title, body_text) = match template_id {
        "contrato" => (
            "CONTRATO DE PRESTAÇÃO DE SERVIÇOS ODONTOLÓGICOS",
            "Pelo presente instrumento particular, a clínica qualificada acima compromete-se a executar os procedimentos odontológicos acordados no plano de tratamento do paciente qualificado. As partes concordam mutuamente com os valores, cronograma e orientações clínicas fornecidas.",
        ),
        "tcle" => (
            "TERMO DE CONSENTIMENTO LIVRE E ESCLARECIDO (TCLE)",
            "Declaro para os devidos fins que fui plenamente informado(a) sobre a natureza, benefícios, riscos e alternativas do tratamento odontológico proposto, tendo todas as minhas dúvidas sanadas pela equipe profissional.",
        ),
        "atestado" => (
            "ATESTADO DE COMPARECIMENTO ODONTOLÓGICO",
            "Atesto para os devidos fins que o(a) paciente esteve sob cuidados odontológicos nesta unidade clínica nesta data, necessitando de repouso e afastamento de suas atividades habituais.",
        ),
        "recibo" => (
            "RECIBO DE QUITAÇÃO ODONTOLÓGICA",
            "Recebemos do(a) paciente a quantia discriminada referente aos serviços e procedimentos odontológicos prestados, dando plena e rasa quitação pelo valor pago.",
        ),
        _ => ("DOCUMENTO CLÍNICO ODONTOLÓGICO", "Documento gerado pelo sistema de gestão odontológica."),
    };

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 20px;",
            div { style: "display: flex; align-items: center; justify-content: space-between; background: #1a2035; padding: 12px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                button {
                    r#type: "button",
                    class: "btn-secondary",
                    onclick: move |_| on_back.call(()),
                    "← Voltar aos Modelos"
                }

                button {
                    r#type: "button",
                    class: "btn-primary",
                    onclick: move |_| {
                        let _ = web_sys::window().map(|w| w.print());
                    },
                    IconPrinter { size: 16, color: "#ffffff".to_string() }
                    span { "Imprimir Folha A4 / Salvar PDF" }
                }
            }

            div { class: "doc-paper-preview",
                div { class: "doc-paper-header",
                    div { class: "doc-paper-clinic-name", "{clinic_name}" }
                    div { class: "doc-paper-clinic-meta", "Unidade Odontológica Integrada • Responsável Técnico: Dr. Roberto Alencar CRO-SP 84920" }
                    h2 { class: "doc-paper-title", "{title}" }
                }

                div { class: "doc-paper-body",
                    p { "{body_text}" }
                }

                div { class: "doc-paper-signatures",
                    div { class: "doc-sign-line",
                        strong { "{patient_name}" }
                        div { "Assinatura do Paciente / Responsável" }
                    }
                    div { class: "doc-sign-line",
                        strong { "{doctor_name}" }
                        div { "Cirurgião-Dentista / Responsável" }
                    }
                }
            }
        }
    }
}
