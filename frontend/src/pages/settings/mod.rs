pub mod components;

use crate::api::mock_db::DB;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::clinics::ClinicAddress;
use dioxus::prelude::*;

pub use components::*;

const STYLE: Asset = asset!("/src/pages/settings/style.css");

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Clinic,
    Plans,
    Anamnesis,
    Contracts,
    Categories,
    FinancialAccounts,
    Chairs,
    Copilot,
    Communication,
    PosRates,
    MyDoctor,
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

    let mut current_tab = use_signal(|| SettingsTab::Clinic);

    // Campos da aba Clínica
    let mut trading_name = use_signal(|| "Luria Odontologia Especializada".to_string());
    let mut cnpj = use_signal(|| "68.001.376/0001-54".to_string());
    let mut comm_name = use_signal(|| "Fernanda".to_string());
    let mut manager_name = use_signal(|| "Fernanda".to_string());
    let mut opening_hour = use_signal(|| 9u32);
    let mut closing_hour = use_signal(|| 19u32);
    let mut timezone = use_signal(|| "Brasilia/São Paulo".to_string());
    let mut fiscal_issuer = use_signal(|| "Clínica".to_string());
    let mut email = use_signal(|| "clinicaluria2026@gmail.com".to_string());
    let mut phone = use_signal(|| "(11) 98873-7247".to_string());
    let mut cellphone = use_signal(|| "(11) 99642-4551".to_string());
    let mut print_letterhead = use_signal(|| true);
    let mut letterhead_options = use_signal(|| vec![
        "Orçamento".to_string(),
        "Evoluções".to_string(),
        "Anamnese".to_string(),
        "Receituários".to_string(),
        "Atestados".to_string(),
        "Documentos personalizados".to_string(),
        "Controle de prótese".to_string(),
        "Recibos".to_string(),
    ]);
    let mut address = use_signal(|| ClinicAddress {
        street: "Rua Juvenal Ferreira dos Santos".to_string(),
        number: "45".to_string(),
        complement: Some("terreo".to_string()),
        neighborhood: "Parque Luíza".to_string(),
        city: "Embu das Artes".to_string(),
        state: "SP".to_string(),
        zip_code: "06816-240".to_string(),
    });

    use_effect({
        let cid = clinic_id.clone();
        let mut tn = trading_name.clone();
        let mut c_cnpj = cnpj.clone();
        let mut c_comm = comm_name.clone();
        let mut c_mgr = manager_name.clone();
        let mut oh = opening_hour.clone();
        let mut ch = closing_hour.clone();
        let mut tz = timezone.clone();
        let mut fi = fiscal_issuer.clone();
        let mut em = email.clone();
        let mut ph = phone.clone();
        let mut cell = cellphone.clone();
        let mut pl = print_letterhead.clone();
        let mut lo = letterhead_options.clone();
        let mut addr = address.clone();

        move || {
            if let Ok(db) = DB.lock() {
                if let Some(clinic) = db.clinics.iter().find(|c| c.id == cid) {
                    tn.set(clinic.trading_name.clone());
                    c_cnpj.set(clinic.document_cnpj.clone());
                    if let Some(ref cn) = clinic.communication_name { c_comm.set(cn.clone()); }
                    if let Some(ref mn) = clinic.manager_name { c_mgr.set(mn.clone()); }
                    oh.set(clinic.opening_hour);
                    ch.set(clinic.closing_hour);
                    tz.set(clinic.timezone.clone());
                    fi.set(clinic.fiscal_issuer.clone());
                    if let Some(ref e) = clinic.email { em.set(e.clone()); }
                    if let Some(ref p) = clinic.phone { ph.set(p.clone()); }
                    if let Some(ref cp) = clinic.cellphone { cell.set(cp.clone()); }
                    pl.set(clinic.print_letterhead);
                    lo.set(clinic.letterhead_options.clone());
                    addr.set(clinic.address.clone());
                }
            }
        }
    });

    let handle_save_clinic = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let tn_s = trading_name.clone();
        let cnpj_s = cnpj.clone();
        let comm_s = comm_name.clone();
        let mgr_s = manager_name.clone();
        let oh_s = opening_hour.clone();
        let ch_s = closing_hour.clone();
        let tz_s = timezone.clone();
        let fi_s = fiscal_issuer.clone();
        let em_s = email.clone();
        let ph_s = phone.clone();
        let cell_s = cellphone.clone();
        let pl_s = print_letterhead.clone();
        let lo_s = letterhead_options.clone();
        let addr_s = address.clone();

        move |_| {
            if let Ok(mut db) = DB.lock() {
                if let Some(clinic) = db.clinics.iter_mut().find(|c| c.id == cid) {
                    clinic.trading_name = tn_s.read().clone();
                    clinic.document_cnpj = cnpj_s.read().clone();
                    clinic.communication_name = Some(comm_s.read().clone());
                    clinic.manager_name = Some(mgr_s.read().clone());
                    clinic.opening_hour = *oh_s.read();
                    clinic.closing_hour = *ch_s.read();
                    clinic.timezone = tz_s.read().clone();
                    clinic.fiscal_issuer = fi_s.read().clone();
                    clinic.email = Some(em_s.read().clone());
                    clinic.phone = Some(ph_s.read().clone());
                    clinic.cellphone = Some(cell_s.read().clone());
                    clinic.print_letterhead = *pl_s.read();
                    clinic.letterhead_options = lo_s.read().clone();
                    clinic.address = addr_s.read().clone();
                }
            }
            toast_c.show("Configurações da clínica salvas com sucesso!", ToastVariant::Success);
        }
    };

    let active_tab_val = *current_tab.read();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "settings-page",
            // TOP NAVBAR (TABS CONFORME SCREENSHOTS)
            div { class: "settings-top-navbar",
                button {
                    class: if active_tab_val == SettingsTab::Clinic { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Clinic),
                    "CLÍNICA"
                }
                button {
                    class: if active_tab_val == SettingsTab::Plans { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Plans),
                    "PLANOS"
                }
                button {
                    class: if active_tab_val == SettingsTab::Anamnesis { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Anamnesis),
                    "ANAMNESE"
                }
                button {
                    class: if active_tab_val == SettingsTab::Contracts { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Contracts),
                    "CONTRATO"
                }
                button {
                    class: if active_tab_val == SettingsTab::Categories { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Categories),
                    "CATEGORIAS"
                }
                button {
                    class: if active_tab_val == SettingsTab::FinancialAccounts { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::FinancialAccounts),
                    "CONTAS FINANCEIRAS"
                }
                button {
                    class: if active_tab_val == SettingsTab::Chairs { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Chairs),
                    "CADEIRAS"
                }
                button {
                    class: if active_tab_val == SettingsTab::Copilot { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Copilot),
                    "COPILOTO"
                }
                button {
                    class: if active_tab_val == SettingsTab::Communication { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Communication),
                    "COMUNICAÇÃO"
                }
                button {
                    class: if active_tab_val == SettingsTab::PosRates { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::PosRates),
                    "TAXAS MAQUININHA"
                }
                button {
                    class: if active_tab_val == SettingsTab::MyDoctor { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::MyDoctor),
                    "MEU DOUTOR"
                }
                button {
                    class: if active_tab_val == SettingsTab::Team { "settings-nav-tab active" } else { "settings-nav-tab" },
                    onclick: move |_| current_tab.set(SettingsTab::Team),
                    "EQUIPE & ACESSOS"
                }
            }

            // CORPO DO CONTEÚDO
            div { class: "settings-content-body",
                match active_tab_val {
                    SettingsTab::Clinic => rsx! {
                        TabClinic {
                            trading_name,
                            cnpj,
                            comm_name,
                            manager_name,
                            opening_hour,
                            closing_hour,
                            timezone,
                            fiscal_issuer,
                            email,
                            phone,
                            cellphone,
                            print_letterhead,
                            letterhead_options,
                            address,
                            on_save: handle_save_clinic,
                        }
                    },
                    SettingsTab::Plans => rsx! {
                        TabPlans {}
                    },
                    SettingsTab::Anamnesis => rsx! {
                        TabAnamnesis {}
                    },
                    SettingsTab::Team => rsx! {
                        TabTeam { clinic_id: clinic_id.clone() }
                    },
                    _ => rsx! {
                        div { style: "padding: 40px 20px; text-align: center; color: #64748b;",
                            h3 { style: "font-size: 16px; color: #94a3b8; margin-bottom: 8px;", "Configurações em Sincronização" }
                            p { style: "font-size: 13px;", "Este módulo utiliza os padrões integrados da clínica ativa." }
                        }
                    },
                }
            }
        }
    }
}
