use crate::api::authenticate;
use dioxus::prelude::*;
use shared::auth::LoginRequest;

#[component]
pub fn LoginScreen(on_login_success: EventHandler<shared::auth::LoginResponse>) -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);

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
                    on_login_success.call(response);
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
                h2 { class: "login-title", "Tooth Plus ERP" }

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
