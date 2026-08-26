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
        document::Link { rel: "stylesheet", href: MAIN_STYLE }
        document::Link { rel: "stylesheet", href: COMPONENTS_STYLE }
        Router::<Route> {}
        ToastContainer {}
    }
}
