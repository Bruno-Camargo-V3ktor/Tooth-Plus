//! # Menu Lateral de Navegação (Estilo Simples Dental)
//!
//! Exibe os itens do menu em fundo branco limpo, com ícones SVG lineares,
//! indicação de item ativo por pílula azul suave e botões de suporte/configuração no rodapé.

use crate::icons::{
    IconActivity, IconBox, IconCalendar, IconDollar, IconHelp, IconMessageSquare, IconPhone,
    IconSettings, IconTooth, IconTrendingUp, IconUsers,
};
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Sidebar(is_collapsed: bool) -> Element {
    let current_route = use_route::<Route>();

    let nav_items = vec![
        ("Inteligência", Route::DashboardView {}, rsx! { IconActivity { size: 19, color: "currentColor".to_string() } }),
        ("Pacientes", Route::PatientsView {}, rsx! { IconUsers { size: 19, color: "currentColor".to_string() } }),
        ("Agenda", Route::AgendaView {}, rsx! { IconCalendar { size: 19, color: "currentColor".to_string() } }),
        ("Vendas", Route::BudgetsView {}, rsx! { IconTrendingUp { size: 19, color: "currentColor".to_string() } }),
        ("Financeiro", Route::FinanceView {}, rsx! { IconDollar { size: 19, color: "currentColor".to_string() } }),
        ("Controle de Prótese", Route::TreatmentsView {}, rsx! { IconTooth { size: 19, color: "currentColor".to_string() } }),
        ("Marketing", Route::MarketingView {}, rsx! { IconMessageSquare { size: 19, color: "currentColor".to_string() } }),
        ("Estoque", Route::StockView {}, rsx! { IconBox { size: 19, color: "currentColor".to_string() } }),
    ];

    let collapsed_class = if is_collapsed { "sidebar-collapsed" } else { "" };

    rsx! {
        aside { class: "sidebar-simples {collapsed_class}",
            // Seção Superior de Navegação
            nav { class: "sidebar-nav-menu",
                for (label, route_dest, icon_el) in nav_items {
                    {
                        let is_active = current_route == route_dest;
                        let active_class = if is_active { "nav-item-active" } else { "" };
                        rsx! {
                            Link {
                                key: "{label}",
                                to: route_dest,
                                class: "sidebar-nav-link {active_class}",
                                title: if is_collapsed { "{label}" } else { "" },
                                div { class: "nav-icon-wrapper", {icon_el} }
                                if !is_collapsed {
                                    span { class: "nav-link-label", "{label}" }
                                }
                            }
                        }
                    }
                }
            }

            // Seção Inferior: Ajustes, Ajuda e Suporte
            div { class: "sidebar-footer-section",
                Link {
                    to: Route::SettingsView {},
                    class: format!("sidebar-nav-link {}", if current_route == (Route::SettingsView {}) { "nav-item-active" } else { "" }),
                    title: if is_collapsed { "Ajustes" } else { "" },
                    div { class: "nav-icon-wrapper", IconSettings { size: 19, color: "currentColor".to_string() } }
                    if !is_collapsed {
                        span { class: "nav-link-label", "Ajustes" }
                    }
                }

                div {
                    class: "sidebar-nav-link link-help",
                    title: if is_collapsed { "Como funciona" } else { "" },
                    div { class: "nav-icon-wrapper", IconHelp { size: 19, color: "currentColor".to_string() } }
                    if !is_collapsed {
                        span { class: "nav-link-label", "Como funciona" }
                    }
                }

                if !is_collapsed {
                    div { class: "sidebar-support-button-wrap",
                        button {
                            r#type: "button",
                            class: "btn-support-pill",
                            IconPhone { size: 15, color: "#ffffff".to_string() }
                            span { "Chamar suporte" }
                        }
                    }
                }
            }
        }
    }
}
