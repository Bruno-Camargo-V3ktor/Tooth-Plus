pub mod components;

use crate::api::mock_db::DB;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use dioxus::prelude::*;

pub use components::{TabClinic, TabHours, TabTeam};

const STYLE: Asset = asset!("/src/pages/settings/style.css");

#[derive(Clone, PartialEq)]
pub enum SettingsTab {
    ClinicData,
    Hours,
    Team,
}

#[component]
pub fn SettingsView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let mut toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut current_tab = use_signal(|| SettingsTab::Hours);

    let mut trading_name = use_signal(|| "SmilePlus Odontologia".to_string());
    let mut cnpj = use_signal(|| "12.345.678/0001-90".to_string());
    let mut phone = use_signal(|| "(11) 3333-4444".to_string());
    let street = use_signal(|| "Av. Paulista".to_string());
    let number = use_signal(|| "1000, Cj. 120".to_string());

    let mut opening_hour = use_signal(|| 8u32);
    let mut closing_hour = use_signal(|| 19u32);

    use_effect({
        let cid = clinic_id.clone();
        let mut tn = trading_name.clone();
        let mut c_cnpj = cnpj.clone();
        let mut oh = opening_hour.clone();
        let mut ch = closing_hour.clone();

        move || {
            if let Ok(db) = DB.lock() {
                if let Some(clinic) = db.clinics.iter().find(|c| c.id == cid) {
                    tn.set(clinic.trading_name.clone());
                    c_cnpj.set(clinic.document_cnpj.clone());
                    oh.set(clinic.opening_hour);
                    ch.set(clinic.closing_hour);
                }
            }
        }
    });

    let handle_save = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let t_name = trading_name.clone();
        let oh_val = opening_hour.clone();
        let ch_val = closing_hour.clone();

        move |_| {
            if let Ok(mut db) = DB.lock() {
                if let Some(clinic) = db.clinics.iter_mut().find(|c| c.id == cid) {
                    clinic.trading_name = t_name.read().clone();
                    clinic.opening_hour = *oh_val.read();
                    clinic.closing_hour = *ch_val.read();
                }
            }
            toast_c.show("Configurações salvas com sucesso!", ToastVariant::Success);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "settings-page",
            div { class: "settings-container",
                div { class: "settings-header",
                    h1 { class: "settings-header-title", "Ajustes da Clínica" }
                    p { class: "settings-header-subtitle",
                        "Gerencie os dados da unidade, horário de funcionamento da agenda e membros da equipe."
                    }
                }

                div { class: "tab-underline-bar settings-tabs-bar",
                    button {
                        class: if *current_tab.read() == SettingsTab::Hours { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| current_tab.set(SettingsTab::Hours),
                        "Horários da Agenda"
                    }
                    button {
                        class: if *current_tab.read() == SettingsTab::ClinicData { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| current_tab.set(SettingsTab::ClinicData),
                        "Dados da Clínica"
                    }
                    button {
                        class: if *current_tab.read() == SettingsTab::Team { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                        onclick: move |_| current_tab.set(SettingsTab::Team),
                        "Equipe & Acessos"
                    }
                }

                match *current_tab.read() {
                    SettingsTab::Hours => rsx! {
                        TabHours { opening_hour, closing_hour, on_save: handle_save }
                    },
                    SettingsTab::ClinicData => rsx! {
                        TabClinic { trading_name, cnpj, phone, street, number, on_save: handle_save }
                    },
                    SettingsTab::Team => rsx! {
                        TabTeam {}
                    },
                }
            }
        }
    }
}
