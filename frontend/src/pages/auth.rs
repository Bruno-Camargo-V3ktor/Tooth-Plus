use crate::api::authenticate;
use crate::router::Route;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::auth::LoginRequest;
use shared::models::ClinicAccess;

#[component]
pub fn LoginScreen() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);

    let mut session = consume_context::<Signal<SessionState>>();
    let navigator = use_navigator();

    let handle_login = move |_| {
        spawn(async move {
            is_loading.set(true);
            error_msg.set(String::new());

            let req = LoginRequest {
                username: username.cloned(),
                password_plain: password.cloned(),
            };

            match authenticate(req).await {
                Ok(response) => {
                    is_loading.set(false);
                    session.set(Some(response));
                    navigator.push(Route::ContextSelector {});
                }
                Err(e) => {
                    is_loading.set(false);
                    error_msg.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "login-wrapper",
            div { class: "login-box",
                h2 { class: "login-title", "ToothPlus" }

                if !error_msg.read().is_empty() {
                    div { class: "error-msg", "{error_msg}" }
                }

                div { class: "login-form",
                    input {
                        class: "input-field",
                        type: "text",
                        placeholder: "Username",
                        value: "{username}",
                        oninput: move |e| username.set(e.value())
                    }

                    input {
                        class: "input-field",
                        type: "password",
                        placeholder: "Password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value())
                    }

                    button {
                        class: "btn-primary",
                        onclick: handle_login,
                        disabled: *is_loading.read(),
                        if *is_loading.read() {
                            "Authenticating..."
                        } else {
                            "Sign In"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ContextSelector() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let mut active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let navigator = use_navigator();

    let clinics = session
        .read()
        .as_ref()
        .map(|s| s.clinics.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "context-wrapper",
            h2 { class: "login-title", "Select Clinic" }

            div { class: "card-grid",
                for clinic in clinics {
                    div {
                        key: "{clinic.clinic_id}",
                        class: "clinic-card",
                        style: "border-top: 4px solid {clinic.theme_color};",
                        onclick: move |_| {
                            active_clinic.set(Some(clinic.clone()));
                            navigator.push(Route::AgendaView {});
                        },

                        h3 { class: "clinic-name", "{clinic.trading_name}" }
                        span { class: "clinic-role", "Role: {clinic.role}" }
                    }
                }
            }
        }
    }
}
