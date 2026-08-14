use crate::pages::auth::{ContextSelector, LoginScreen};
use crate::pages::dashboard::DashboardLayout;
use crate::pages::sign_portal::SignPortal;
use crate::pages::views::*;
use dioxus::prelude::*;


#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    LoginScreen {},

    #[route("/context")]
    ContextSelector {},

    #[route("/sign/:token")]
    SignPortal { token: String },

    #[layout(DashboardLayout)]
    #[route("/agenda")]
    AgendaView {},

    #[route("/patients")]
    PatientsView {},

    #[route("/finance")]
    FinanceView {},

    #[route("/stock")]
    StockView {},

    #[route("/users")]
    UsersView {},

    #[route("/documents")]
    DocumentsView {},

    #[end_layout]
    #[route("/:.._route")]
    NotFound { _route: Vec<String> },
}

impl Route {
    pub fn title(&self) -> &'static str {
        match self {
            Route::AgendaView {} => "Agenda de Atendimentos",
            Route::PatientsView {} => "Gestão de Pacientes",
            Route::FinanceView {} => "Fluxo Financeiro",
            Route::StockView {} => "Estoque e Patrimônio",
            Route::UsersView {} => "Equipe e Usuários",
            Route::DocumentsView {} => "Emissão de Documentos",
            _ => "Tooth Plus",
        }
    }
}

#[component]
pub fn NotFound(_route: Vec<String>) -> Element {
    rsx! {
        div { "Page not found" }
    }
}
