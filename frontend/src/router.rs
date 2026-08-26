//! # Roteamento Principal

use dioxus::prelude::*;
use crate::pages::login::LoginScreen;
use crate::pages::login::ContextSelector;
use crate::pages::dashboard::DashboardView;
use crate::pages::agenda::AgendaView;
use crate::pages::patients::PatientsView;
use crate::pages::treatments::TreatmentsView;
use crate::pages::finance::FinanceView;
use crate::pages::stock::StockView;
use crate::pages::documents::DocumentsView;
use crate::pages::settings::SettingsView;
use crate::pages::sign_portal::SignPortal;
use crate::components::layout::AppLayout;

#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/login")]
    LoginScreen {},

    #[route("/select-clinic")]
    ContextSelector {},

    #[route("/sign/:token")]
    SignPortal { token: String },

    #[layout(AppLayout)]
        #[route("/")]
        #[redirect("/dashboard", || Route::DashboardView {})]
        #[route("/dashboard")]
        DashboardView {},

        #[route("/agenda")]
        AgendaView {},

        #[route("/patients")]
        PatientsView {},

        #[route("/treatments")]
        TreatmentsView {},

        #[route("/finance")]
        FinanceView {},

        #[route("/stock")]
        StockView {},

        #[route("/documents")]
        DocumentsView {},

        #[route("/settings")]
        SettingsView {},
    #[end_layout]

    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
pub fn PageNotFound(route: Vec<String>) -> Element {
    let path_str = route.join("/");
    rsx! {
        div { class: "view-container empty-state-card", style: "margin: 60px auto; max-width: 500px;",
            h2 { "Página Não Encontrada (404)" }
            p { "O endereço acessado não existe ou foi movido: /{path_str}" }
            Link {
                to: Route::DashboardView {},
                class: "btn-primary",
                style: "margin-top: 16px; display: inline-block;",
                "Voltar ao Início"
            }
        }
    }
}
