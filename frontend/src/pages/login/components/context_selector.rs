use crate::api::{save_active_clinic, ActiveClinicState, SessionState};
use crate::icons::IconChevronRight;
use crate::router::Route;
use dioxus::prelude::*;

const CLINIC_ICON: Asset = asset!("/assets/icon.svg");

#[component]
pub fn ClinicSelector() -> Element {
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
                                            src: CLINIC_ICON,
                                            style: "width: 24px; height: 24px; max-width: 24px; max-height: 24px; object-fit: contain;",
                                            alt: "Ícone da Unidade",
                                            width: "24",
                                            height: "24",
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
