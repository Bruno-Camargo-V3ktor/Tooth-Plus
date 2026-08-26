//! # Módulo de Inteligência Artificial Clínica
//!
//! Tela limpa e minimalista de "Em desenvolvimento".

use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/dashboard/style.css");

#[component]
pub fn DashboardView() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "intelligence-page",
            div { class: "intelligence-centered-container",
                div { class: "intelligence-pulse-ring",
                    div { class: "intelligence-icon-glow", "⚡" }
                }
                h1 { class: "intelligence-main-title", "Em desenvolvimento" }
                p { class: "intelligence-main-subtitle", "Módulo de Inteligência Artificial Tooth Plus" }
            }
        }
    }
}
