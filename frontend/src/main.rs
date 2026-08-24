use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div { class: "app-container",
            header { class: "app-header",
                div { class: "brand-row",
                    span { class: "brand-logo", "🦷" }
                    h1 { class: "brand-title", "Tooth Plus" }
                    span { class: "version-badge", "v2.0 Refactor" }
                }
                p { class: "app-subtitle", "Sistema de Gestão Odontológica Inteligente" }
            }

            main { class: "main-content",
                div { class: "welcome-card",
                    h2 { "Bem-vindo ao Tooth Plus V2" }
                    p { "Ambiente limpo pronto para a nova arquitetura e design Simples Dental." }
                    div { class: "counter-box",
                        p { "Contador de Teste Reativo: {count}" }
                        button {
                            class: "btn-primary",
                            onclick: move |_| count.set(count() + 1),
                            "Incrementar"
                        }
                    }
                }
            }
        }
    }
}
