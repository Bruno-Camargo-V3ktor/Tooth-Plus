use dioxus::prelude::*;
use shared::models::ClinicAccess;

#[component]
pub fn ClinicCard(clinic: ClinicAccess, on_select: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "clinic-card-premium",
            onclick: move |_| on_select.call(()),

            div {
                class: "clinic-badge-line",
                style: "background-color: {clinic.theme_color};"
            }

            if let Some(url) = &clinic.logo_url {
                div { class: "clinic-card-logo-wrapper",
                    img { class: "clinic-card-logo-img", src: "{url}", alt: "{clinic.trading_name}" }
                }
            } else {
                h3 { class: "clinic-card-title", "{clinic.trading_name}" }
            }

            div {
                class: "clinic-card-tag",
                style: "color: {clinic.theme_color}; background-color: {clinic.theme_color}15;",
                "{clinic.role}"
            }
        }
    }
}
