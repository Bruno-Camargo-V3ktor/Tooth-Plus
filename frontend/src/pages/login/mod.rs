pub mod components;

use crate::api::auth::AuthApi;
use crate::api::{save_active_clinic, save_session, ActiveClinicState, SessionState};
use crate::router::Route;
use dioxus::prelude::*;

pub use components::{ClinicSelector, LoginBanner, LoginForm};

const STYLE: Asset = asset!("/src/pages/login/style.css");

#[component]
pub fn LoginScreen() -> Element {
    let mut session = consume_context::<Signal<Option<SessionState>>>();
    let mut active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let navigator = use_navigator();

    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let handle_login = move |e: Event<FormData>| {
        e.prevent_default();
        let user = username.read().trim().to_string();
        let pass = password.read().trim().to_string();

        if user.is_empty() || pass.is_empty() {
            error_msg.set(Some("Informe usuário e senha para continuar.".to_string()));
            return;
        }

        is_loading.set(true);
        error_msg.set(None);

        let mut sess_sig = session;
        let mut act_sig = active_clinic;
        let nav = navigator.clone();

        spawn(async move {
            match AuthApi::login(user, pass).await {
                Ok(sess) => {
                    save_session(&sess);
                    let clinics = sess.clinics.clone();
                    sess_sig.set(Some(sess));

                    if clinics.len() == 1 {
                        let cl = &clinics[0];
                        let active = ActiveClinicState {
                            clinic_id: cl.clinic_id.clone(),
                            trading_name: cl.trading_name.clone(),
                            theme_color: cl.theme_color.clone(),
                            logo_url: cl.logo_url.clone(),
                            role: cl.role.clone(),
                            permissions: cl.permissions.clone(),
                        };
                        save_active_clinic(&active);
                        act_sig.set(Some(active));
                        nav.replace(Route::AgendaView {});
                    } else {
                        nav.replace(Route::ContextSelector {});
                    }
                }
                Err(err) => {
                    error_msg.set(Some(err));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "login-split-layout",
            LoginForm {
                username,
                password,
                error_msg,
                is_loading,
                on_submit: handle_login,
            }
            LoginBanner {}
        }
    }
}

#[component]
pub fn ContextSelector() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }
        ClinicSelector {}
    }
}
