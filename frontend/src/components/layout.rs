//! # Layout Geral da Aplicação
//!
//! Envolve todas as páginas autenticadas com a Topbar azul e a Sidebar branca no estilo Simples Dental.

use crate::api::{ActiveClinicState, SessionState};
use crate::components::sidebar::Sidebar;
use crate::components::topbar::Topbar;
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn AppLayout() -> Element {
    let session = consume_context::<Signal<Option<SessionState>>>();
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let navigator = use_navigator();

    let mut is_sidebar_collapsed = use_signal(|| false);

    // Se o usuário não estiver logado, redireciona para o login
    if session().is_none() {
        navigator.replace(Route::LoginScreen {});
        return rsx! {};
    }

    // Se estiver logado mas sem clínica selecionada, vai para o seletor de unidade
    if active_clinic().is_none() {
        navigator.replace(Route::ContextSelector {});
        return rsx! {};
    }

    rsx! {
        div { class: "app-shell",
            // 1. Barra Superior Azul
            Topbar {
                on_toggle_sidebar: move |_| is_sidebar_collapsed.set(!is_sidebar_collapsed())
            }

            // 2. Área Central com Menu Lateral e Conteúdo
            div { class: "app-body-layout",
                Sidebar { is_collapsed: is_sidebar_collapsed() }

                main { class: "app-main-viewport",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
