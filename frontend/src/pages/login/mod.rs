//! # Módulo de Autenticação e Seleção de Unidade (Tooth Plus V2)

use crate::api::{save_active_clinic, save_session, ActiveClinicState, AuthApi, SessionState};
use crate::icons::{IconAlertTriangle, IconChevronRight, IconLock, IconUser};
use crate::router::Route;
use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/login/style.css");

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
        e.prevent_default();
        e.stop_propagation();

        let u = username().trim().to_string();
        let p = password().trim().to_string();

        if u.is_empty() || p.is_empty() {
            error_msg.set(Some("Informe o nome de usuário e a senha.".to_string()));
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
            match AuthApi::login(u, p).await {
                Ok(sess) => {
                    save_session(&sess);
                    let clinics = sess.clinics.clone();
                    sess_sig.set(Some(sess));
                    loading_sig.set(false);

                    if clinics.len() == 1 {
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
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "login-split-layout",
            // Lado Esquerdo: Formulário Minimalista de Login
            div { class: "login-form-side",
                div { class: "login-box",
                    div { class: "login-brand-logo-wrap",
                        img {
                            src: "/assets/logo.svg",
                            class: "login-brand-logo",
                            alt: "ToothPlus",
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
                        onsubmit: handle_login,

                        div { class: "login-input-wrapper",
                            span { class: "login-input-icon",
                                IconUser { size: 18, color: "#94a3b8".to_string() }
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
                                IconLock { size: 18, color: "#94a3b8".to_string() }
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

            // Lado Direito: Banner Escuro Minimalista (Visual Simples Dental V2)
            div { class: "login-visual-side",
                div { class: "login-visual-container",
                    span { class: "login-badge-pill", "Versão 2026.1" }
                    h2 { class: "login-visual-title", "A evolução da gestão odontológica." }
                    p { class: "login-visual-desc",
                        "Prontuários eletrônicos, fluxos financeiros automatizados e agendamento inteligente em uma experiência fluida e sem fricção."
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
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "context-wrapper",
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
