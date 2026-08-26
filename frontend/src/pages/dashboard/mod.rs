//! # Módulo de Inteligência Artificial Clínica (Tooth Plus V2)
//!
//! Exibe uma interface elegante de "Em Breve" com visão geral das inovações
//! de IA e automação preditiva integradas à clínica odontológica.

use crate::components::toast::{ToastState, ToastVariant};
use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/dashboard/style.css");

#[component]
pub fn DashboardView() -> Element {
    let mut toast = consume_context::<ToastState>();
    let mut email_input = use_signal(String::new);

    let handle_subscribe = move |e: Event<FormData>| {
        e.prevent_default();
        let email = email_input.read().trim().to_string();
        if email.is_empty() || !email.contains('@') {
            toast.show("Por favor, insira um e-mail válido.", ToastVariant::Error);
            return;
        }
        toast.show("Você foi adicionado à lista de acesso antecipado da IA!", ToastVariant::Success);
        email_input.set(String::new());
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "intelligence-page",
            // Seção Hero
            div { class: "intelligence-hero-section",
                div { class: "intelligence-badge-pulse",
                    div { class: "pulse-dot" }
                    span { "Em Desenvolvimento • Lançamento 2026.2" }
                }

                h1 { class: "intelligence-title", "Tooth Plus Intelligence" }
                p { class: "intelligence-desc",
                    "A inteligência artificial nativa projetada para transformar o atendimento clínico, automatizar diagnósticos radiográficos e otimizar o fluxo financeiro do seu consultório."
                }

                form {
                    class: "intelligence-notify-box",
                    onsubmit: handle_subscribe,
                    input {
                        r#type: "email",
                        class: "intelligence-input",
                        placeholder: "Digite seu e-mail para acesso beta...",
                        value: "{email_input}",
                        oninput: move |e| email_input.set(e.value()),
                    }
                    button {
                        r#type: "submit",
                        class: "intelligence-btn-notify",
                        "Quero Acesso VIP"
                    }
                }
            }

            // Grade de Recursos Previstos
            div { class: "intelligence-grid",
                div { class: "intelligence-card",
                    div { class: "intelligence-card-icon-wrap icon-blue", "⚡" }
                    h3 { class: "intelligence-card-title", "Laudo Radiográfico Assistido" }
                    p { class: "intelligence-card-desc",
                        "Detecção instantânea de lesões de cárie, perda óssea periodontal e anomalias periapicais em radiografias periapicais e panorâmicas."
                    }
                    span { class: "intelligence-card-tag", "Visão Computacional" }
                }

                div { class: "intelligence-card",
                    div { class: "intelligence-card-icon-wrap icon-purple", "🤖" }
                    h3 { class: "intelligence-card-title", "Assistente de WhatsApp 24/7" }
                    p { class: "intelligence-card-desc",
                        "Confirmação inteligente de agendamentos, triagem prévia de sintomas e reagendamento automático em linguagem natural sem sobrecarregar a recepção."
                    }
                    span { class: "intelligence-card-tag", "IA Conversacional" }
                }

                div { class: "intelligence-card",
                    div { class: "intelligence-card-icon-wrap icon-amber", "📈" }
                    h3 { class: "intelligence-card-title", "Previsão Financeira & Inadimplência" }
                    p { class: "intelligence-card-desc",
                        "Modelos preditivos que antecipam riscos de churn, atraso de parcelas de tratamentos e recomendam estratégias de precificação ideais."
                    }
                    span { class: "intelligence-card-tag", "Analytics Preditivo" }
                }

                div { class: "intelligence-card",
                    div { class: "intelligence-card-icon-wrap icon-emerald", "📑" }
                    h3 { class: "intelligence-card-title", "Evolução Clínica por Voz" }
                    p { class: "intelligence-card-desc",
                        "Ditado inteligente direto para o prontuário eletrônico: fale o procedimento realizado e o sistema estrutura anamnese, prescrição e evolução."
                    }
                    span { class: "intelligence-card-tag", "Speech-to-Text Médico" }
                }
            }
        }
    }
}
