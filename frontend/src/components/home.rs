use dioxus::prelude::*;
use shared::auth::LoginResponse;
use shared::models::ClinicAccess;

#[component]
pub fn Home(
    user: LoginResponse,
    active_clinic: ClinicAccess,
    on_switch_clinic: EventHandler<()>,
    on_logout: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "home-layout",

            div {
                class: "sidebar",
                style: "background-color: {active_clinic.theme_color};",

                h2 { class: "sidebar-title", "{active_clinic.trading_name}" }

                div { class: "spacer" }

                button {
                    class: "btn-secondary",
                    onclick: move |_| on_switch_clinic.call(()),
                    "Switch Context"
                }

                button {
                    class: "btn-danger",
                    onclick: move |_| on_logout.call(()),
                    "Logout"
                }
            }

            div { class: "main-content",

                h1 { class: "welcome-title", "Welcome back, {user.full_name}" }

                p { class: "role-text",
                    "Active Authorization Level: "
                    strong { class: "role-highlight", "{active_clinic.role}" }
                }

                div { class: "quick-actions-panel",
                    h3 { class: "quick-actions-title", "Quick Actions" }
                    p { class: "text-muted", "Patients, Odontogram, and Digital Signatures modules will be loaded here." }
                }
            }
        }
    }
}
