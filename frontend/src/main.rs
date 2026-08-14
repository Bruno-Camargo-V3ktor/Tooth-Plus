use crate::router::Route;
use dioxus::prelude::*;
use shared::auth::LoginResponse;
use shared::models::ClinicAccess;

mod api;
mod components;
mod pages;
mod permissions;
mod router;

pub type SessionState = Option<LoginResponse>;
pub type ActiveClinicState = Option<ClinicAccess>;
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(None::<LoginResponse>));
    use_context_provider(|| Signal::new(None::<ClinicAccess>));

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}
