use crate::pages::auth::{ContextSelector, LoginScreen};
use crate::pages::dashboard::{AgendaView, DashboardLayout, FinanceView, PatientsView, StockView};
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    LoginScreen {},

    #[route("/context")]
    ContextSelector {},

    #[layout(DashboardLayout)]
    #[route("/agenda")]
    AgendaView {},

    #[route("/patients")]
    PatientsView {},

    #[route("/finance")]
    FinanceView {},

    #[route("/stock")]
    StockView {},
    #[end_layout]
    #[route("/:.._route")]
    NotFound { _route: Vec<String> },
}

#[component]
pub fn NotFound(_route: Vec<String>) -> Element {
    rsx! {
        div { "Page not found" }
    }
}
