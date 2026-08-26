//! # Menu Lateral de Navegação (Dark Dental Theme)
//!
//! Exibe os itens do menu em tema escuro moderno, com ícones SVG lineares,
//! indicação de item ativo por pílula azul e seção de Ajustes no rodapé.

use crate::icons::{
    IconActivity, IconBox, IconCalendar, IconDollar, IconFileText,
    IconSettings, IconUsers,
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
        ("Financeiro", Route::FinanceView {}, rsx! { IconDollar { size: 19, color: "currentColor".to_string() } }),
        ("Inventário", Route::StockView {}, rsx! { IconBox { size: 19, color: "currentColor".to_string() } }),
        ("Documentos", Route::DocumentsView {}, rsx! { IconFileText { size: 19, color: "currentColor".to_string() } }),
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

            // Seção Inferior: Ajustes
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
            }
        }
    }
}
