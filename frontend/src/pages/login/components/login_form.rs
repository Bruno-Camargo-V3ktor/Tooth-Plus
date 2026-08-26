use crate::icons::{IconAlertTriangle, IconLock, IconUser};
use dioxus::prelude::*;

const LOGO: Asset = asset!("/assets/logo.svg");

#[component]
pub fn LoginForm(
    username: Signal<String>,
    password: Signal<String>,
    error_msg: Signal<Option<String>>,
    is_loading: Signal<bool>,
    on_submit: EventHandler<Event<FormData>>,
) -> Element {
    rsx! {
        div { class: "login-form-side",
            div { class: "login-box",
                div {
                    class: "login-brand-logo-wrap",
                    style: "margin-bottom: 28px; display: flex; align-items: center; height: 42px;",
                    img {
                        src: LOGO,
                        class: "login-brand-logo",
                        style: "width: 160px; height: 40px; max-width: 160px; max-height: 40px; object-fit: contain; display: block;",
                        alt: "ToothPlus",
                        width: "160",
                        height: "40",
                    }
                }

                h1 { class: "login-title-welcome", "Bem-vindo de volta" }
                p { class: "login-subtitle", "Insira suas credenciais para gerenciar a clínica" }

                if let Some(ref err) = error_msg() {
                    div { class: "login-error-alert",
                        IconAlertTriangle { size: 18, color: "#ef4444".to_string() }
                        span { "{err}" }
                    }
                }

                form {
                    class: "login-form",
                    onsubmit: move |e| on_submit.call(e),

                    div { class: "login-input-wrapper",
                        span { class: "login-input-icon",
                            IconUser { size: 18, color: "#64748b".to_string() }
                        }
                        input {
                            r#type: "text",
                            class: "login-input-field",
                            placeholder: "Nome de usuário",
                            value: "{username}",
                            oninput: move |e| username.set(e.value()),
                            autofocus: true,
                        }
                    }

                    div { class: "login-input-wrapper",
                        span { class: "login-input-icon",
                            IconLock { size: 18, color: "#64748b".to_string() }
                        }
                        input {
                            r#type: "password",
                            class: "login-input-field",
                            placeholder: "Senha de acesso",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }

                    div { class: "forgot-password-row",
                        span { class: "forgot-password-link", "Esqueceu sua senha?" }
                    }

                    button {
                        r#type: "submit",
                        class: "btn-login-submit",
                        disabled: is_loading(),
                        if is_loading() {
                            "Entrando..."
                        } else {
                            "Entrar no sistema"
                        }
                    }
                }
            }
        }
    }
}
