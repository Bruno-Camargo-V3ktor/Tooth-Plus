use dioxus::prelude::*;
use shared::models::ClinicAccess;

#[component]
pub fn ContextSelector(
    clinics: Vec<ClinicAccess>,
    on_select: EventHandler<ClinicAccess>,
) -> Element {
    rsx! {
        div { class: "context-wrapper",
            h2 { class: "context-title", "Select Workspace" }

            div { class: "card-grid",
                for clinic in clinics {
                    div {
                        key: "{clinic.clinic_id}",
                        class: "clinic-card",
                        style: "border-top: 6px solid {clinic.theme_color};",
                        onclick: move |_| on_select.call(clinic.clone()),

                        h3 { class: "clinic-name", "{clinic.trading_name}" }
                        span { class: "clinic-role", "Role: {clinic.role}" }
                    }
                }
            }
        }
    }
}
