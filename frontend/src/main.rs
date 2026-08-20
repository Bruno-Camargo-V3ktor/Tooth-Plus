use crate::router::Route;
use dioxus::prelude::*;
use shared::auth::LoginResponse;
use shared::models::ClinicAccess;

mod api;
mod components;
mod pages;
mod permissions;
mod router;
pub mod utils;

pub type SessionState = Option<LoginResponse>;
pub type ActiveClinicState = Option<ClinicAccess>;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const MANIFEST_JSON: Asset = asset!("/assets/manifest.json");
const FAVICON_SVG: Asset = asset!("/assets/favicon.svg");
const FAVICON_32: Asset = asset!("/assets/favicon-32x32.png");
const FAVICON_16: Asset = asset!("/assets/favicon-16x16.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/apple-touch-icon.png");
const _ICON_192: Asset = asset!("/assets/icon-192.png");
const _ICON_512: Asset = asset!("/assets/icon-512.png");
const _ICON_MASKABLE_512: Asset = asset!("/assets/icon-maskable-512.png");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let initial_session = utils::load_session();
    let initial_clinic = utils::load_active_clinic();

    use_context_provider(|| Signal::new(initial_session));
    use_context_provider(|| Signal::new(initial_clinic));

    rsx! {
        document::Title { "Tooth Plus - Gestão Odontológica" }
        document::Meta { name: "description", content: "Sistema integrado e moderno de gestão para clínicas e consultórios odontológicos" }
        document::Meta { name: "theme-color", content: "#00a0e4" }
        document::Meta { name: "mobile-web-app-capable", content: "yes" }
        document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
        document::Meta { name: "apple-mobile-web-app-status-bar-style", content: "default" }
        document::Meta { name: "apple-mobile-web-app-title", content: "Tooth Plus" }
        document::Meta { name: "application-name", content: "Tooth Plus" }
        document::Meta { name: "msapplication-TileColor", content: "#00a0e4" }

        document::Link { rel: "manifest", href: MANIFEST_JSON }
        document::Link { rel: "icon", r#type: "image/svg+xml", href: FAVICON_SVG }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "apple-touch-icon", sizes: "180x180", href: APPLE_TOUCH_ICON }

        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
