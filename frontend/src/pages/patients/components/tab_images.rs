use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::IconPlus;
use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn TabImages(patient: Patient) -> Element {
    let mut toast = consume_context::<ToastState>();

    let mock_images = vec![
        ("Radiografia Panorâmica Inicial", "2026-08-01", "🦷"),
        ("Periapical Dente 21 e 22", "2026-08-10", "📷"),
        ("Foto Intraoral Oclusal Superior", "2026-08-15", "📸"),
    ];

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { style: "display: flex; align-items: center; justify-content: space-between; background: #121a2c; padding: 12px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);",
                div {
                    h3 { style: "font-size: 15px; font-weight: 700; color: #f8fafc; margin: 0;", "Imagens Clínicas & Radiografias" }
                    p { style: "font-size: 12.5px; color: #94a3b8; margin: 2px 0 0 0;", "Armazenamento de exames por imagem e fotos intraorais." }
                }

                button {
                    r#type: "button",
                    class: "btn-primary",
                    onclick: move |_| {
                        toast.show("Selecione um arquivo de imagem para upload.", ToastVariant::Info);
                    },
                    IconPlus { size: 15, color: "#ffffff".to_string() }
                    span { "ENVIAR IMAGEM" }
                }
            }

            div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 16px;",
                for (title, date, icon) in mock_images {
                    div {
                        key: "{title}",
                        class: "patient-card",
                        div { style: "height: 120px; background: #0a0f1d; display: flex; align-items: center; justify-content: center; font-size: 40px;",
                            "{icon}"
                        }
                        div { style: "padding: 12px;",
                            strong { style: "font-size: 13px; color: #f8fafc; display: block; margin-bottom: 2px;", "{title}" }
                            span { style: "font-size: 11.5px; color: #94a3b8;", "Data: {date}" }
                        }
                    }
                }
            }
        }
    }
}
