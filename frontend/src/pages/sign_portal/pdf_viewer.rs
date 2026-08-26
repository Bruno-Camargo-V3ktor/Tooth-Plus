use crate::icons::IconFileText;
use shared::documents::PatientDocument;
use dioxus::prelude::*;

#[component]
pub fn DocumentViewerCard(doc: PatientDocument, clinic_name: String) -> Element {
    let p_name = doc.patient_name.clone().unwrap_or_else(|| "Paciente".to_string());
    let doc_name = doc.doctor_user_name.clone().unwrap_or_else(|| "Dr. Lucas Mendes - CRO 12345".to_string());

    rsx! {
        div { class: "portal-card",
            div { style: "display: flex; align-items: center; justify-content: space-between;",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    IconFileText { size: 20, color: "#38bdf8".to_string() }
                    h2 { class: "portal-doc-title", "{doc.title}" }
                }
                span { class: "badge badge-yellow", "Pendente Assinatura" }
            }

            div { class: "portal-doc-paper",
                div { style: "border-bottom: 1px solid rgba(255,255,255,0.08); padding-bottom: 10px; margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;",
                    strong { style: "color: #38bdf8; font-size: 13px;", "{clinic_name}" }
                    span { style: "font-size: 11px; color: #94a3b8;", "Documento Oficial" }
                }

                p {
                    strong { "PACIENTE: " } "{p_name}" br {}
                    strong { "PROFISSIONAL: " } "{doc_name}" br {}
                    strong { "DATA DE EMISSÃO: " } "{doc.created_at}"
                }

                h4 { style: "color: #f1f5f9; margin: 12px 0 6px 0; font-size: 13px;", "CLÁUSULA 1ª — DO OBJETO E PROCEDIMENTOS" }
                p {
                    "Pelo presente instrumento particular, o(a) PACIENTE autoriza a equipe clínica da "
                    strong { "{clinic_name}" }
                    " a realizar os procedimentos odontológicos diagnósticos e terapêuticos necessários, tendo sido previamente informado(a) acerca dos métodos, riscos, benefícios e alternativas de tratamento."
                }

                h4 { style: "color: #f1f5f9; margin: 12px 0 6px 0; font-size: 13px;", "CLÁUSULA 2ª — DO CONSENTIMENTO E ORIENTAÇÕES" }
                p {
                    "O(a) PACIENTE declara estar ciente de que o sucesso do tratamento depende rigorosamente da assiduidade às consultas e da observância estrita das recomendações e prescrições médicas/odontológicas pós-operatórias."
                }

                h4 { style: "color: #f1f5f9; margin: 12px 0 6px 0; font-size: 13px;", "CLÁUSULA 3ª — DA VALIDADE JURÍDICA E LGPD" }
                p {
                    "A presente assinatura eletrônica possui plena validade jurídica conforme Medida Provisória nº 2.200-2/2001 e Lei Federal nº 14.063/2020, com registro criptográfico de integridade e carimbo de data/hora."
                }
            }
        }
    }
}
