use crate::router::Route;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;

#[component]
pub fn DashboardLayout() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();
    let navigator = use_navigator();

    if session.read().is_none() || active_clinic.read().is_none() {
        spawn(async move {
            navigator.replace(Route::LoginScreen {});
        });
        return rsx! { div {} };
    }

    let clinic_name = active_clinic.read().as_ref().unwrap().trading_name.clone();
    let theme_color = active_clinic.read().as_ref().unwrap().theme_color.clone();
    let user_name = session.read().as_ref().unwrap().full_name.clone();

    rsx! {
        div { class: "dashboard-layout",
            div { class: "sidebar",
                div {
                    class: "sidebar-header",
                    style: "background-color: {theme_color};",
                    "{clinic_name}"
                }

                div { class: "nav-menu",
                    Link {
                        to: Route::AgendaView {},
                        class: "nav-item",
                        active_class: "nav-item-active",
                        "Agenda"
                    }
                    Link {
                        to: Route::PatientsView {},
                        class: "nav-item",
                        active_class: "nav-item-active",
                        "Pacientes"
                    }
                    Link {
                        to: Route::FinanceView {},
                        class: "nav-item",
                        active_class: "nav-item-active",
                        "Financeiro"
                    }
                    Link {
                        to: Route::StockView {},
                        class: "nav-item",
                        active_class: "nav-item-active",
                        "Estoque"
                    }
                }
            }

            div { class: "main-area",
                div { class: "topbar",
                    div { class: "topbar-user", "{user_name}" }
                }

                div { class: "content-wrapper",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
pub fn AgendaView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Agenda" }
            div { class: "content-card",
                "Calendar component will be rendered here."
            }
        }
    }
}

#[component]
pub fn PatientsView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Pacientes" }
            div { class: "content-card",
                "Patient list and registration form will be rendered here."
            }
        }
    }
}

#[component]
pub fn FinanceView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Fluxo de Caixa" }
            div { class: "content-card",
                "Financial data will be rendered here."
            }
        }
    }
}

#[component]
pub fn StockView() -> Element {
    rsx! {
        div {
            h1 { class: "page-title", "Estoque" }
            div { class: "content-card",
                "Inventory data will be rendered here."
            }
        }
    }
}
