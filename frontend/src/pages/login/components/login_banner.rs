use dioxus::prelude::*;

#[component]
pub fn LoginBanner() -> Element {
    rsx! {
        div { class: "login-visual-side",
            div { class: "login-visual-container",
                span { class: "login-badge-pill", "Plataforma Odontológica" }
                h2 { class: "login-visual-title", "A evolução da gestão odontológica." }
                p { class: "login-visual-desc",
                    "Prontuários eletrônicos, fluxos financeiros integrados e agendamento inteligente em uma experiência rápida e segura."
                }

                div { class: "login-visual-skeleton-card",
                    div { class: "skeleton-bar bar-gray" }
                    div { class: "skeleton-bar bar-blue" }
                    div { class: "skeleton-dots", "..." }
                }
            }
        }
    }
}
