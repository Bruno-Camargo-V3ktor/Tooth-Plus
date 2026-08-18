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
        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
