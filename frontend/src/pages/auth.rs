use crate::api::authenticate;
use crate::components::clinic_card::ClinicCard;
use crate::router::Route;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::auth::LoginRequest;

#[component]
pub fn LoginScreen() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);

    let mut session = consume_context::<Signal<SessionState>>();
    let navigator = use_navigator();

    let error_msg_val = error_msg();
    let loading_val = is_loading();

    let handle_login = move |_e: Event<FormData>| {
        spawn(async move {
            is_loading.set(true);
            error_msg.set(String::new());

            let req = LoginRequest {
                username: username.cloned(),
                password_plain: password.cloned(),
            };

            match authenticate(req).await {
                Ok(response) => {
                    is_loading.set(false);
                    session.set(Some(response));
                    navigator.push(Route::ContextSelector {});
                }
                Err(e) => {
                    is_loading.set(false);
                    error_msg.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "login-wrapper",
            div { class: "login-form-side",
                div { class: "login-box",
                    div { class: "login-logo-container",
                        svg {
                            view_box: "0 0 200 50",
                            fill: "none",
                            xmlns: "http://www.w3.org/2000/svg",
                            class: "brand-logo-svg",
                            path { d: "M20 10C25 18 22 28 14 32C6 36 2 28 5 20C8 12 15 2 20 10Z", fill: "#00a0e4" }
                            path { d: "M12 18C17 26 14 36 6 40C-2 44 -6 36 -3 28C0 20 7 10 12 18Z", fill: "#0284c7", opacity: "0.7" }
                            text { x: "45", y: "32", fill: "#0f172a", font_size: "24", font_weight: "700", font_family: "Inter, sans-serif", letter_spacing: "-0.04em", "Tooth" }
                            text { x: "108", y: "32", fill: "#00a0e4", font_size: "24", font_weight: "700", font_family: "Inter, sans-serif", letter_spacing: "-0.04em", "Plus" }
                        }
                    }

                    h2 { class: "login-title-welcome", "Bem-vindo de volta" }
                    p { class: "login-subtitle", "Insira suas credenciais para gerenciar a clínica" }

                    if !error_msg_val.is_empty() {
                        div { class: "modern-error-box",
                            svg { class: "error-box-icon", xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "2", stroke: "currentColor",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" }
                            }
                            div { class: "error-box-content",
                                strong { "Falha na Autenticação" }
                                span { "{error_msg_val}" }
                            }
                        }
                    }

                    form { class: "login-form",
                        onsubmit: handle_login,

                        div { class: "login-input-wrapper",
                            span { class: "login-input-icon",
                                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.8", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" }
                                }
                            }
                            input {
                                class: "login-input-field",
                                r#type: "text",
                                placeholder: "Nome de usuário",
                                value: "{username}",
                                oninput: move |e| username.set(e.value())
                            }
                        }

                        div { class: "login-input-wrapper",
                            span { class: "login-input-icon",
                                svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", view_box: "0 0 24 24", stroke_width: "1.8", stroke: "currentColor",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", d: "M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0V10.5m-2.25 0h13.5c.621 0 1.125.504 1.125 1.125v7.497c0 .621-.504 1.125-1.125 1.125H3.75c-.621 0-1.125-.504-1.125-1.125v-7.497c0-.621.504-1.125 1.125-1.125Z" }
                                }
                            }
                            input {
                                class: "login-input-field",
                                r#type: "password",
                                placeholder: "Senha de acesso",
                                value: "{password}",
                                oninput: move |e| password.set(e.value())
                            }
                        }

                        div { class: "forgot-password-link", "Esqueceu sua senha?" }

                        button {
                            class: "btn-modern-submit",
                            r#type: "submit",
                            disabled: loading_val,
                            if loading_val {
                                "Carregando..."
                            } else {
                                "Entrar no sistema"
                            }
                        }
                    }
                }
            }

            div { class: "login-visual-side",
                div { class: "visual-overlay-mesh" }
                div { class: "login-visual-content",
                    div { class: "floating-badge", "Versão 2026.1" }
                    h2 { "A evolução da gestão odontológica." }
                    p { "Prontuários eletrônicos, fluxos financeiros automatizados e agendamento inteligente em uma experiência fluida e sem fricção." }

                    div { class: "mock-ui-card",
                        div { class: "mock-line long" }
                        div { class: "mock-line medium" }
                        div { class: "mock-dots",
                            div { class: "dot" }
                            div { class: "dot" }
                            div { class: "dot" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ContextSelector() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let navigator = use_navigator();

    let clinics = session()
        .as_ref()
        .map(|s| s.clinics.clone())
        .unwrap_or_default();
    let user_name = session()
        .as_ref()
        .map(|s| s.full_name.clone())
        .unwrap_or_else(|| "Usuário".to_string());

    rsx! {
        div { class: "context-wrapper",
            div { class: "visual-overlay-mesh" }

            div { class: "context-container",
                div { class: "context-header-zone",
                    div { class: "user-avatar-badge",
                        "{user_name.chars().next().unwrap_or('U')}"
                    }
                    h2 { class: "context-welcome-title", "Olá, {user_name}" }
                    p { class: "context-subtitle", "Selecione em qual unidade da instituição você irá trabalhar hoje" }
                }


                div { class: "card-grid",
                    for clinic in clinics {
                        ClinicCard {
                            key: "{clinic.clinic_id}",
                            clinic: clinic.clone(),
                            on_select: move |_| {
                                active_clinic.set(Some(clinic.clone()));
                                navigator.push(Route::AgendaView {});
                            }
                        }
                    }
                }


                div { class: "context-footer-zone",
                    span { "Não é você? " }
                    button {
                        class: "context-logout-link",
                        onclick: move |_| {
                            let mut session_sig = consume_context::<Signal<SessionState>>();
                            session_sig.set(None);
                            navigator.push(Route::LoginScreen {});
                        },
                        "Alternar conta"
                    }
                }
            }
        }
    }
}
