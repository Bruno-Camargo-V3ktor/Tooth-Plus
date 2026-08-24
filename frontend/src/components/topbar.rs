//! # Barra Superior de Navegação (Estilo Simples Dental)
//!
//! Exibe a faixa azul superior com breadcrumbs do módulo ativo, notificações,
//! atalhos rápidos, clínica em operação e menu de usuário.

use crate::api::{clear_session, ActiveClinicState, SessionState};
use crate::icons::{IconBell, IconChevronDown, IconLogOut, IconMenu, IconUsers};
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Topbar(on_toggle_sidebar: EventHandler<()>) -> Element {
    let session = consume_context::<Signal<Option<SessionState>>>();
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let navigator = use_navigator();
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
        Route::DashboardView {} => "Inteligência & Dashboard",
        Route::AgendaView {} => "Agenda",
        Route::PatientsView {} => "Pacientes",
        Route::BudgetsView {} => "Vendas & Orçamentos",
        Route::FinanceView {} => "Financeiro",
        Route::TreatmentsView {} => "Tratamentos & Prótese",
        Route::MarketingView {} => "Marketing & Mensagens",
        Route::StockView {} => "Estoque & Materiais",
        Route::SettingsView {} => "Ajustes & Configurações",
        _ => "Tooth Plus",
    };

    rsx! {
        header { class: "topbar-simples",
            // Lado Esquerdo: Toggle Menu + Logo + Breadcrumb
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
                            src: "/assets/icon.svg",
                            alt: "Tooth Plus",
                        }
                        span { class: "topbar-brand-name", "Tooth Plus" }
                    }
                    span { class: "topbar-breadcrumb-separator", "›" }
                    span { class: "topbar-breadcrumb-active", "{page_title}" }
                }
            }

            // Lado Direito: Notificações, Atalhos e Badge da Clínica / Usuário
            div { class: "topbar-right-zone",
                // Sino de Notificações
                button {
                    r#type: "button",
                    class: "topbar-action-icon-btn",
                    title: "Notificações do Sistema",
                    IconBell { size: 19, color: "#ffffff".to_string() }
                    span { class: "topbar-badge-notification", "3" }
                }

                // Badge / Seletor de Clínica Ativa
                div {
                    class: "topbar-clinic-pill",
                    onclick: move |_| is_user_menu_open.set(!is_user_menu_open()),
                    div { class: "clinic-pill-indicator" }
                    span { class: "clinic-pill-text", "{clinic_name}" }
                    IconChevronDown { size: 14, color: "#ffffff".to_string() }
                }

                // Avatar / Dropdown de Usuário
                div { class: "topbar-user-avatar-wrap",
                    button {
                        r#type: "button",
                        class: "topbar-avatar-btn",
                        title: "{user_name}",
                        onclick: move |_| is_user_menu_open.set(!is_user_menu_open()),
                        span { class: "avatar-initial", "{user_name.chars().next().unwrap_or('U')}" }
                    }

                    if is_user_menu_open() {
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
                                    is_user_menu_open.set(false);
                                    navigator.push(Route::ContextSelector {});
                                },
                                IconUsers { size: 16, color: "#475569".to_string() }
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
                                IconLogOut { size: 16, color: "#dc2626".to_string() }
                                span { "Sair da Conta" }
                            }
                        }
                    }
                }
            }
        }
    }
}
