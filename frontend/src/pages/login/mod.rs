//! # Módulo de Autenticação e Seleção de Unidade (Tooth Plus V2)

use crate::api::auth::login_user;
use crate::api::{save_active_clinic, save_session, ActiveClinicState, SessionState};
use crate::icons::{IconAlertTriangle, IconChevronRight, IconLock, IconUsers};
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn LoginScreen() -> Element {
    let session = consume_context::<Signal<Option<SessionState>>>();
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let navigator = use_navigator();

    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let is_loading = use_signal(|| false);

    let handle_login = move |e: Event<FormData>| {
        e.stop_propagation();
        let u = username().trim().to_string();
        let p = password().trim().to_string();

        if u.is_empty() || p.is_empty() {
            error_msg.set(Some("Por favor, preencha o usuário e a senha.".to_string()));
            return;
        }

        let mut loading_sig = is_loading;
        let mut err_sig = error_msg;
        let mut sess_sig = session;
        let mut act_sig = active_clinic;
        let nav = navigator;

        loading_sig.set(true);
        err_sig.set(None);

        spawn(async move {
            match login_user(u, p).await {
                Ok(sess) => {
                    save_session(&sess);
                    let clinics = sess.clinics.clone();
                    sess_sig.set(Some(sess));
                    loading_sig.set(false);

                    if clinics.len() == 1 {
                        // Se possui apenas 1 clínica, seleciona automaticamente
                        let cl = &clinics[0];
                        let active = ActiveClinicState {
                            clinic_id: cl.clinic_id.clone(),
                            trading_name: cl.trading_name.clone(),
                            theme_color: cl.theme_color.clone(),
                            logo_url: cl.logo_url.clone(),
                            role: cl.role.clone(),
                            permissions: cl.permissions.clone(),
                        };
                        save_active_clinic(&active);
                        act_sig.set(Some(active));
                        nav.replace(Route::AgendaView {});
                    } else {
                        // Se possui múltiplas ou nenhuma pré-selecionada, abre seletor
                        nav.replace(Route::ContextSelector {});
                    }
                }
                Err(err) => {
                    loading_sig.set(false);
                    err_sig.set(Some(err));
                }
            }
        });
    };

    rsx! {
        div { class: "login-wrapper",
            // Lado Esquerdo: Formulário de Login
            div { class: "login-form-side",
                div { class: "login-box",
                    // Logo SVG Tooth Plus
                    div { class: "login-logo-container",
                        img {
                            src: "/assets/icon.svg",
                            class: "brand-logo-svg",
                            alt: "Tooth Plus Logo",
                        }
                    }

                    h2 { class: "login-title-welcome", "Bem-vindo de volta" }
                    p { class: "login-subtitle", "Insira suas credenciais para acessar a plataforma odontológica" }

                    if let Some(ref err) = error_msg() {
                        div { class: "modern-error-box",
                            IconAlertTriangle { size: 20, color: "#dc2626".to_string() }
                            div { class: "error-box-content",
                                strong { "Falha na Autenticação" }
                                span { "{err}" }
                            }
                        }
                    }

                    form { class: "login-form", onsubmit: handle_login,
                        div { class: "login-input-group",
                            label { "Usuário ou E-mail" }
                            div { class: "login-input-wrapper",
                                span { class: "login-input-icon",
                                    IconUsers { size: 18, color: "#94a3b8".to_string() }
                                }
                                input {
                                    r#type: "text",
                                    class: "login-input-field",
                                    placeholder: "Ex: admin ou dr.lucas",
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value()),
                                    autofocus: true,
                                }
                            }
                        }

                        div { class: "login-input-group",
                            label { "Senha de Acesso" }
                            div { class: "login-input-wrapper",
                                span { class: "login-input-icon",
                                    IconLock { size: 18, color: "#94a3b8".to_string() }
                                }
                                input {
                                    r#type: "password",
                                    class: "login-input-field",
                                    placeholder: "Sua senha segura",
                                    value: "{password}",
                                    oninput: move |e| password.set(e.value()),
                                }
                            }
                        }

                        div { class: "forgot-password-link", "Esqueceu sua senha?" }

                        button {
                            r#type: "submit",
                            class: "btn-modern-submit",
                            disabled: is_loading(),
                            if is_loading() {
                                "Entrando..."
                            } else {
                                "Acessar Sistema"
                            }
                        }
                    }

                    div { class: "login-demo-credentials",
                        p { "💡 Dica de Acesso Rápido (Mock):" }
                        span { "Usuário: " strong { "admin" } " | Senha: " strong { "qualquer senha" } }
                    }
                }
            }

            // Lado Direito: Banner Visual Moderno
            div { class: "login-visual-side",
                div { class: "visual-overlay-mesh" }
                div { class: "login-visual-content",
                    div { class: "floating-badge", "Tooth Plus V2" }
                    h2 { "A evolução da gestão odontológica inteligente." }
                    p { "Prontuários eletrônicos completos, controle financeiro em tempo real e agendamento ágil integrados em uma experiência fluida inspirada nas melhores práticas clínicas." }

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
    let session = consume_context::<Signal<Option<SessionState>>>();
    let mut active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
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
                    p { class: "context-subtitle", "Selecione a clínica ou unidade em que você irá atuar hoje:" }
                }

                div { class: "card-grid-clinics",
                    for cl in clinics {
                        {
                            let cl_item = cl.clone();
                            rsx! {
                                div {
                                    key: "{cl.clinic_id}",
                                    class: "clinic-selection-card",
                                    onclick: move |_| {
                                        let active = ActiveClinicState {
                                            clinic_id: cl_item.clinic_id.clone(),
                                            trading_name: cl_item.trading_name.clone(),
                                            theme_color: cl_item.theme_color.clone(),
                                            logo_url: cl_item.logo_url.clone(),
                                            role: cl_item.role.clone(),
                                            permissions: cl_item.permissions.clone(),
                                        };
                                        save_active_clinic(&active);
                                        active_clinic.set(Some(active));
                                        navigator.replace(Route::AgendaView {});
                                    },
                                    div { class: "clinic-card-icon",
                                        img {
                                            src: "/assets/icon.svg",
                                            style: "width: 24px; height: 24px;",
                                            alt: "Ícone da Unidade",
                                        }
                                    }
                                    div { class: "clinic-card-info",
                                        h3 { "{cl.trading_name}" }
                                        span { class: "clinic-role-badge", "Perfil: {cl.role}" }
                                    }
                                    div { class: "clinic-card-arrow",
                                        IconChevronRight { size: 20, color: "#94a3b8".to_string() }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "context-footer-zone",
                    button {
                        r#type: "button",
                        class: "context-logout-link",
                        onclick: move |_| {
                            crate::api::clear_session();
                            let mut sess = session;
                            sess.set(None);
                            navigator.replace(Route::LoginScreen {});
                        },
                        "Alternar ou sair da conta"
                    }
                }
            }
        }
    }
}
