//! # Ponto de Entrada do Frontend (Tooth Plus V2)
//!
//! Inicializa o aplicativo Dioxus 0.7, provê os contextos globais de sessão e clínica ativa,
//! e renderiza o roteamento declarativo.

mod api;
mod components;
mod icons;
mod pages;
mod router;

use api::{load_active_clinic, load_session};
use dioxus::prelude::*;
use router::Route;

const MAIN_STYLE: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // 1. Inicializa o estado de sessão a partir do LocalStorage
    let _session = use_context_provider(|| Signal::new(load_session()));

    // 2. Inicializa o estado da clínica ativa
    let _active_clinic = use_context_provider(|| Signal::new(load_active_clinic()));

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_STYLE }
        Router::<Route> {}
    }
}
