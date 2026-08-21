use crate::components::icons::*;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn Sidebar(
    theme_color: String,
    logo_url: Option<String>,
    is_collapsed: bool,
    can_see_agenda: bool,
    can_see_patients: bool,
    can_see_treatments: bool,
    can_see_finance: bool,
    can_see_stock: bool,
    can_see_documents: bool,
    can_see_users: bool,
    can_see_settings: bool,
    on_toggle: EventHandler<()>,
    on_settings: EventHandler<()>,
    on_logout: EventHandler<()>,
) -> Element {
    let icon_color = "currentColor".to_string();

    rsx! {
        div { class: if is_collapsed { "sidebar collapsed" } else { "sidebar" },
            div {
                class: "sidebar-header",
                style: "background-color: {theme_color};",

                if let Some(url) = logo_url {
                    img { class: "sidebar-logo", src: "{url}" }
                }

                button {
                    class: "toggle-btn",
                    onclick: move |_| on_toggle.call(()),
                    IconMenu { size: 24, color: "white".to_string() }
                }
            }

            div { class: "nav-menu",
                if can_see_agenda {
                    Link { to: Route::AgendaView {}, class: "nav-item", active_class: "nav-item-active",
                        IconCalendar { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Agenda" }
                    }
                }
                if can_see_patients {
                    Link { to: Route::PatientsView {}, class: "nav-item", active_class: "nav-item-active",
                        IconUsers { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Pacientes" }
                    }
                }
                if can_see_treatments {
                    Link { to: Route::TreatmentsView {}, class: "nav-item", active_class: "nav-item-active",
                        IconTooth { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Tratamentos" }
                    }
                }
                if can_see_finance {
                    Link { to: Route::FinanceView {}, class: "nav-item", active_class: "nav-item-active",
                        IconFinance { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Financeiro" }
                    }
                }
                if can_see_stock {
                    Link { to: Route::StockView {}, class: "nav-item", active_class: "nav-item-active",
                        IconBox { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Estoque" }
                    }
                }
                if can_see_documents {
                    Link { to: Route::DocumentsView {}, class: "nav-item", active_class: "nav-item-active",
                        IconFile { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Documentos" }
                    }
                }
                if can_see_users {
                    Link { to: Route::UsersView {}, class: "nav-item", active_class: "nav-item-active",
                        IconUsers { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Usuários" }
                    }
                }
            }

            div { class: "sidebar-footer",
                if can_see_settings {
                    button {
                        class: "nav-item nav-btn",
                        onclick: move |_| on_settings.call(()),
                        IconSettings { size: 20, color: icon_color.clone() }
                        span { class: "nav-text", "Configurações" }
                    }
                }
                button {
                    class: "nav-item nav-btn nav-btn-danger",
                    onclick: move |_| on_logout.call(()),
                    IconLogout { size: 20, color: "#ef4444".to_string() }
                    span { class: "nav-text", "Sair" }
                }
            }
        }
    }
}
