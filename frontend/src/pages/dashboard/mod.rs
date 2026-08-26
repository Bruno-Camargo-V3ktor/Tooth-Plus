use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/dashboard/style.css");

#[component]
pub fn DashboardView() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "dashboard-page", style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; min-height: 400px; text-align: center; gap: 8px;",
            h2 { style: "font-size: 20px; font-weight: 800; color: #ffffff; margin: 0;",
                "Em desenvolvimento"
            }
            p { style: "font-size: 14px; color: #94a3b8; margin: 0;",
                "Módulo de BI Tooth Plus"
            }
        }
    }
}
