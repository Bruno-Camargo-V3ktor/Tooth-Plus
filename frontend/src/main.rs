//! # Ponto de Entrada do Frontend (Tooth Plus V2)
//!
//! Inicializa o aplicativo Dioxus 0.7, provê os contextos globais de sessão, clínica ativa e toast,
//! e renderiza o roteamento declarativo.

mod api;
mod components;
mod icons;
mod pages;
mod router;

use api::{load_active_clinic, load_session};
use components::toast::{ToastContainer, ToastState};
use dioxus::prelude::*;
use router::Route;

const MAIN_STYLE: Asset = asset!("/assets/main.css");
const COMPONENTS_STYLE: Asset = asset!("/assets/components.css");
const FAVICON: Asset = asset!("/assets/favicon.ico");
const FAVICON_SVG: Asset = asset!("/assets/favicon.svg");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/apple-touch-icon.png");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // 1. Inicializa o estado de sessão a partir do LocalStorage
    let _session = use_context_provider(|| Signal::new(load_session()));

    // 2. Inicializa o estado da clínica ativa
    let _active_clinic = use_context_provider(|| Signal::new(load_active_clinic()));

    // 3. Inicializa o contexto global de Toast
    use_context_provider(ToastState::new);

    rsx! {
        // Meta e ícones da aplicação
        document::Title { "Tooth Plus — Gestão Odontológica" }
        document::Link { rel: "icon", href: FAVICON_SVG, r#type: "image/svg+xml" }
        document::Link { rel: "alternate icon", href: FAVICON }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }

        // Folhas de estilo globais
        document::Link { rel: "stylesheet", href: MAIN_STYLE }
        document::Link { rel: "stylesheet", href: COMPONENTS_STYLE }

        Router::<Route> {}
        ToastContainer {}
    }
}
