use crate::icons::{IconFileText, IconPrinter};
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn TabDocuments(patient: Patient) -> Element {
    let mock_docs = vec![
        ("Contrato de Prestação de Serviços Odontológicos", "Assinado digitalmente", "2026-08-01", "badge badge-green"),
        ("Termo de Consentimento Livre e Esclarecido (TCLE)", "Aguardando assinatura", "2026-08-15", "badge badge-blue"),
        ("Atestado Odontológico de Repouso (1 dia)", "Emitido", "2026-08-20", "badge badge-gray"),
    ];

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { style: "display: flex; align-items: center; justify-content: space-between; background: #121a2c; padding: 12px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                div {
                    h3 { style: "font-size: 15px; font-weight: 700; color: #f8fafc; margin: 0;", "Documentos & Contratos do Paciente" }
                    p { style: "font-size: 12.5px; color: #94a3b8; margin: 2px 0 0 0;", "Histórico de contratos, termos e atestados emitidos." }
                }

                button {
                    r#type: "button",
                    class: "btn-primary",
                    onclick: move |_| {
                        let _ = web_sys::window().map(|w| w.print());
                    },
                    IconPrinter { size: 15, color: "#ffffff".to_string() }
                    span { "IMPRIMIR SELECIONADO" }
                }
            }

            div { class: "patients-table-card",
                table { class: "patients-list-table",
                    thead {
                        tr {
                            th { "Documento / Título" }
                            th { "Status" }
                            th { "Data de Emissão" }
                            th { style: "text-align: right;", "Ação" }
                        }
                    }
                    tbody {
                        for (title, status, date, badge_cls) in mock_docs {
                            tr {
                                key: "{title}",
                                td {
                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                        IconFileText { size: 16, color: "#00a0e4".to_string() }
                                        strong { style: "color: #f8fafc; font-size: 13.5px;", "{title}" }
                                    }
                                }
                                td {
                                    span { class: "{badge_cls}", "{status}" }
                                }
                                td { "{date}" }
                                td { style: "text-align: right;",
                                    button {
                                        r#type: "button",
                                        class: "btn-secondary",
                                        style: "padding: 4px 10px; font-size: 11.5px;",
                                        onclick: move |_| {
                                            let _ = web_sys::window().map(|w| w.print());
                                        },
                                        "Visualizar"
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
