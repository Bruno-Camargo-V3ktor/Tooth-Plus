//! # Módulo de Configurações e Ajustes da Clínica (Tooth Plus V2)
//!
//! Permite personalizar horários de funcionamento (abertura/fechamento da agenda),
//! dados cadastrais da clínica, equipe e preferências de comunicação.

use crate::api::mock_db::DB;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use dioxus::prelude::*;

const STYLE: Asset = asset!("/src/pages/settings/style.css");

#[derive(Clone, PartialEq)]
enum SettingsTab {
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

    // Campos da clínica
    let mut trading_name = use_signal(|| "SmilePlus Odontologia".to_string());
    let mut cnpj = use_signal(|| "12.345.678/0001-90".to_string());
    let mut phone = use_signal(|| "(11) 3333-4444".to_string());
    let mut street = use_signal(|| "Av. Paulista".to_string());
    let mut number = use_signal(|| "1000, Cj. 120".to_string());

    // Horários de funcionamento
    let mut opening_hour = use_signal(|| 8u32);
    let mut closing_hour = use_signal(|| 19u32);

    // Carrega dados iniciais da clínica
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
            toast_c.show("Configurações da clínica salvas com sucesso!", ToastVariant::Success);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "settings-page",
            div { class: "settings-container",

                // Header
                div { class: "settings-header",
                    h1 { class: "settings-header-title", "Ajustes da Clínica" }
                    p { class: "settings-header-subtitle",
                        "Gerencie os dados da unidade, horário de funcionamento da agenda e membros da equipe."
                    }
                }

                // Abas
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

                // Conteúdo da Aba Ativa
                match *current_tab.read() {
                    SettingsTab::Hours => rsx! {
                        div { class: "settings-card",
                            div { class: "settings-card-header",
                                h3 { class: "settings-card-title", "Horário de Atendimento e Grade da Agenda" }
                            }
                            div { class: "settings-card-body",
                                p { style: "font-size: 13px; color: #94a3b8; margin: 0 0 8px 0;",
                                    "Defina os horários em que a clínica realiza consultas. A grade da página Agenda será ajustada automaticamente para iniciar e terminar nestes horários."
                                }

                                div { class: "form-row-2 form-row",
                                    div { class: "form-field",
                                        label { class: "form-label", "Hora de Abertura (Início da Agenda) *" }
                                        select {
                                            class: "form-select",
                                            value: "{opening_hour}",
                                            onchange: move |e| {
                                                if let Ok(v) = e.value().parse::<u32>() { opening_hour.set(v); }
                                            },
                                            option { value: "6", "06:00" }
                                            option { value: "7", "07:00" }
                                            option { value: "8", "08:00 (Padrão)" }
                                            option { value: "9", "09:00" }
                                            option { value: "10", "10:00" }
                                        }
                                    }
                                    div { class: "form-field",
                                        label { class: "form-label", "Hora de Fechamento (Fim da Agenda) *" }
                                        select {
                                            class: "form-select",
                                            value: "{closing_hour}",
                                            onchange: move |e| {
                                                if let Ok(v) = e.value().parse::<u32>() { closing_hour.set(v); }
                                            },
                                            option { value: "17", "17:00" }
                                            option { value: "18", "18:00" }
                                            option { value: "19", "19:00 (Padrão)" }
                                            option { value: "20", "20:00" }
                                            option { value: "21", "21:00" }
                                            option { value: "22", "22:00" }
                                        }
                                    }
                                }
                            }
                            div { class: "settings-card-footer",
                                button {
                                    class: "btn-modal-primary",
                                    onclick: handle_save,
                                    "Salvar Alterações de Horário"
                                }
                            }
                        }
                    },
                    SettingsTab::ClinicData => rsx! {
                        div { class: "settings-card",
                            div { class: "settings-card-header",
                                h3 { class: "settings-card-title", "Identificação e Contato da Clínica" }
                            }
                            div { class: "settings-card-body",
                                div { class: "form-row-2 form-row",
                                    div { class: "form-field",
                                        label { class: "form-label", "Nome Fantasia" }
                                        input {
                                            class: "form-input",
                                            r#type: "text",
                                            value: "{trading_name}",
                                            oninput: move |e| trading_name.set(e.value()),
                                        }
                                    }
                                    div { class: "form-field",
                                        label { class: "form-label", "CNPJ" }
                                        input {
                                            class: "form-input",
                                            r#type: "text",
                                            value: "{cnpj}",
                                            oninput: move |e| cnpj.set(e.value()),
                                        }
                                    }
                                }

                                div { class: "form-row-2 form-row",
                                    div { class: "form-field",
                                        label { class: "form-label", "Telefone Comercial" }
                                        input {
                                            class: "form-input",
                                            r#type: "text",
                                            value: "{phone}",
                                            oninput: move |e| phone.set(e.value()),
                                        }
                                    }
                                    div { class: "form-field",
                                        label { class: "form-label", "Logradouro & Número" }
                                        input {
                                            class: "form-input",
                                            r#type: "text",
                                            value: "{street}, {number}",
                                        }
                                    }
                                }
                            }
                            div { class: "settings-card-footer",
                                button {
                                    class: "btn-modal-primary",
                                    onclick: handle_save,
                                    "Salvar Dados da Clínica"
                                }
                            }
                        }
                    },
                    SettingsTab::Team => rsx! {
                        div { class: "settings-card",
                            div { class: "settings-card-header",
                                h3 { class: "settings-card-title", "Membros da Equipe e Permissões" }
                            }
                            div { class: "settings-card-body",
                                div { class: "team-member-row",
                                    div { class: "team-member-info",
                                        div { class: "team-avatar", "RA" }
                                        div {
                                            div { style: "font-weight: 700; color: #f8fafc;", "Dr. Roberto Alencar" }
                                            div { style: "font-size: 12px; color: #94a3b8;", "admin • CRO-SP 84920" }
                                        }
                                    }
                                    span { class: "badge badge-blue", "Administrador Geral" }
                                }
                                div { class: "team-member-row",
                                    div { class: "team-member-info",
                                        div { class: "team-avatar", "LM" }
                                        div {
                                            div { style: "font-weight: 700; color: #f8fafc;", "Dr. Lucas Mendes" }
                                            div { style: "font-size: 12px; color: #94a3b8;", "dr.lucas • CRO-SP 99120" }
                                        }
                                    }
                                    span { class: "badge badge-green", "Cirurgião-Dentista" }
                                }
                                div { class: "team-member-row",
                                    div { class: "team-member-info",
                                        div { class: "team-avatar", "FO" }
                                        div {
                                            div { style: "font-weight: 700; color: #f8fafc;", "Fernanda Oliveira" }
                                            div { style: "font-size: 12px; color: #94a3b8;", "recepcao • Atendimento" }
                                        }
                                    }
                                    span { class: "badge badge-gray", "Recepção / Secretária" }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
