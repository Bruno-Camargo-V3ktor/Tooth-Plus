use crate::components::sidebar::Sidebar;
use crate::components::topbar::Topbar;
use crate::router::Route;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;

#[component]
pub fn DashboardLayout() -> Element {
    // 1. Declarados com 'mut' (Exigência do Dioxus 0.7)
    let mut session = consume_context::<Signal<SessionState>>();
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let navigator = use_navigator();

    let mut is_collapsed = use_signal(|| false);
    let mut is_settings_open = use_signal(|| false);
    let mut active_tab = use_signal(|| "Geral".to_string());

    if session().is_none() || active_clinic().is_none() {
        spawn(async move {
            navigator.replace(Route::LoginScreen {});
        });
        return rsx! { div {} };
    }

    let clinic = active_clinic().as_ref().unwrap().clone();
    let user_name = session().as_ref().unwrap().full_name.clone();

    // 2. Extrair a leitura ANTES da macro para evitar o E0502
    let collapsed_val = is_collapsed();
    let settings_open_val = is_settings_open();
    let tab_val = active_tab();

    rsx! {
        div { class: "dashboard-layout",
            Sidebar {
                theme_color: clinic.theme_color,
                logo_url: clinic.logo_url,
                is_collapsed: collapsed_val, // Usa o valor lido
                on_toggle: move |_| is_collapsed.set(!is_collapsed()), // Captura o signal com permissão mutável
                on_settings: move |_| is_settings_open.set(true),
                on_logout: move |_| {
                    active_clinic.set(None);
                    session.set(None);
                }
            }

            div { class: "main-area",
                Topbar { user_name: user_name }

                div { class: "content-wrapper",
                    Outlet::<Route> {}
                }
            }

            if settings_open_val {
                div { class: "modal-overlay",
                    div { class: "settings-modal",
                        div { class: "settings-header",
                            h2 { class: "settings-title", "Configurações do Sistema" }
                            button { class: "close-btn", onclick: move |_| is_settings_open.set(false), "×" }
                        }
                        div { class: "settings-tabs",
                            for tab in ["Geral", "Aparência", "Impressão", "Segurança"] {
                                button {
                                    class: if tab_val == tab { "tab-btn active" } else { "tab-btn" },
                                    onclick: move |_| active_tab.set(tab.to_string()),
                                    "{tab}"
                                }
                            }
                        }
                        div { class: "settings-content",
                            h3 { "Categoria: {tab_val}" }
                            p { "As opções de {tab_val} serão renderizadas aqui." }
                        }
                    }
                }
            }
        }
    }
}
