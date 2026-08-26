//! # Barra Superior de Navegação

use crate::api::{clear_session, ActiveClinicState, SessionState};
use crate::icons::{IconBell, IconChevronDown, IconLogOut, IconMenu, IconUsers};
use crate::router::Route;
use dioxus::prelude::*;

const TOPBAR_ICON: Asset = asset!("/assets/icon.svg");

#[component]
fn UserDropdown(
    user_name: String,
    clinic_name: String,
    on_close: EventHandler<()>,
) -> Element {
    let session = consume_context::<Signal<Option<SessionState>>>();
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let navigator = use_navigator();

    rsx! {
        div { class: "topbar-user-dropdown-menu",
            div { class: "dropdown-header-user",
                strong { "{user_name}" }
                span { "{clinic_name}" }
            }
            div { class: "dropdown-divider" }
            button {
                r#type: "button",
                class: "dropdown-item-btn",
                onclick: move |_| {
                    on_close.call(());
                    navigator.push(Route::ContextSelector {});
                },
                IconUsers { size: 16, color: "#94a3b8".to_string() }
                span { "Trocar de Unidade / Clínica" }
            }
            button {
                r#type: "button",
                class: "dropdown-item-btn item-logout",
                onclick: move |_| {
                    clear_session();
                    let mut sess = session;
                    sess.set(None);
                    let mut act = active_clinic;
                    act.set(None);
                    navigator.push(Route::LoginScreen {});
                },
                IconLogOut { size: 16, color: "#ef4444".to_string() }
                span { "Sair da Conta" }
            }
        }
    }
}

#[component]
pub fn Topbar(on_toggle_sidebar: EventHandler<()>) -> Element {
    let session = consume_context::<Signal<Option<SessionState>>>();
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let current_route = use_route::<Route>();

    let mut is_user_menu_open = use_signal(|| false);

    let user_name = session()
        .as_ref()
        .map(|s| s.full_name.clone())
        .unwrap_or_else(|| "Usuário".to_string());

    let clinic_name = active_clinic()
        .as_ref()
        .map(|c| c.trading_name.clone())
        .unwrap_or_else(|| "Clínica Principal".to_string());

    let page_title = match current_route {
        Route::DashboardView {} => "Inteligência",
        Route::AgendaView {} => "Agenda",
        Route::PatientsView {} => "Pacientes",
        Route::FinanceView {} => "Financeiro",
        Route::StockView {} => "Inventário",
        Route::DocumentsView {} => "Documentos",
        Route::SettingsView {} => "Ajustes & Configurações",
        _ => "Tooth Plus",
    };

    rsx! {
        header { class: "topbar-simples",
            div { class: "topbar-left-zone",
                button {
                    r#type: "button",
                    class: "topbar-menu-btn",
                    title: "Recolher / Expandir Menu Lateral",
                    onclick: move |_| on_toggle_sidebar.call(()),
                    IconMenu { size: 20, color: "#ffffff".to_string() }
                }

                div { class: "topbar-brand-breadcrumb",
                    div { class: "topbar-logo-box",
                        img {
                            src: TOPBAR_ICON,
                            style: "width: 24px; height: 24px; max-width: 24px; max-height: 24px; object-fit: contain;",
                            alt: "Tooth Plus",
                            width: "24",
                            height: "24",
                        }
                        span { class: "topbar-brand-name", "Tooth Plus" }
                    }
                    span { class: "topbar-breadcrumb-separator", "›" }
                    span { class: "topbar-breadcrumb-active", "{page_title}" }
                }
            }

            div { class: "topbar-right-zone",
                button {
                    r#type: "button",
                    class: "topbar-action-icon-btn",
                    title: "Notificações do Sistema",
                    IconBell { size: 19, color: "#ffffff".to_string() }
                    span { class: "topbar-badge-notification", "3" }
                }

                div {
                    class: "topbar-clinic-pill",
                    onclick: move |_| is_user_menu_open.set(!is_user_menu_open()),
                    div { class: "clinic-pill-indicator" }
                    span { class: "clinic-pill-text", "{clinic_name}" }
                    IconChevronDown { size: 14, color: "#ffffff".to_string() }
                }

                div { class: "topbar-user-avatar-wrap",
                    button {
                        r#type: "button",
                        class: "topbar-avatar-btn",
                        title: "{user_name}",
                        onclick: move |_| is_user_menu_open.set(!is_user_menu_open()),
                        span { class: "avatar-initial", "{user_name.chars().next().unwrap_or('U')}" }
                    }

                    if is_user_menu_open() {
                        UserDropdown {
                            user_name: user_name.clone(),
                            clinic_name: clinic_name.clone(),
                            on_close: move |_| is_user_menu_open.set(false),
                        }
                    }
                }
            }
        }
    }
}
