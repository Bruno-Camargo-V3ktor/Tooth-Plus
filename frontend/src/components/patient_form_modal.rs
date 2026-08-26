//! # Modal Reutilizável de Cadastro/Edição de Paciente (Tooth Plus V2)
//!
//! Componente compartilhado entre a tela de Agenda e a tela de Pacientes.

use dioxus::prelude::*;
use crate::api::{ActiveClinicState, SessionState};
use crate::api::patients::PatientsApi;
use crate::components::toast::{ToastState, ToastVariant};

#[derive(Props, Clone, PartialEq)]
pub struct PatientFormModalProps {
    pub on_save: EventHandler<String>, // patient_id salvo
    pub on_close: EventHandler<()>,
}

#[derive(Clone, PartialEq)]
enum PatientTab {
    ExtraInfo,
    Plan,
    Address,
}

#[component]
pub fn PatientFormModal(props: PatientFormModalProps) -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let session = consume_context::<Signal<Option<SessionState>>>();
    let mut toast = consume_context::<ToastState>();

    let mut full_name = use_signal(|| String::new());
    let mut gender = use_signal(|| String::new());
    let mut birth_date = use_signal(|| String::new());
    let mut document_cpf = use_signal(|| String::new());
    let mut document_rg = use_signal(|| String::new());
    let mut phone = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut how_arrived = use_signal(|| String::new());
    let mut guardian_name = use_signal(|| String::new());
    let mut guardian_phone = use_signal(|| String::new());
    let mut notes = use_signal(|| String::new());
    let mut insurance_plan = use_signal(|| String::new());
    let mut address_street = use_signal(|| String::new());
    let mut address_number = use_signal(|| String::new());
    let mut address_neighborhood = use_signal(|| String::new());
    let mut address_city = use_signal(|| String::new());
    let mut address_state = use_signal(|| String::new());
    let mut address_zip = use_signal(|| String::new());
    let mut active_tab = use_signal(|| PatientTab::ExtraInfo);
    let mut name_error = use_signal(|| false);
    let mut is_saving = use_signal(|| false);

    let on_save = props.on_save.clone();
    let on_close = props.on_close.clone();

    let handle_save = move |_| {
        let name = full_name.read().clone();
        if name.trim().is_empty() {
            name_error.set(true);
            return;
        }
        name_error.set(false);

        let clinic_id = active_clinic
            .read()
            .as_ref()
            .map(|c| c.clinic_id.clone())
            .unwrap_or_default();

        let name_val = name.clone();
        let phone_val = if phone.read().is_empty() { "(00) 00000-0000".to_string() } else { phone.read().clone() };
        let gender_val = if gender.read().is_empty() { None } else { Some(gender.read().clone()) };
        let birth_val = if birth_date.read().is_empty() { None } else { Some(birth_date.read().clone()) };
        let cpf_val = if document_cpf.read().is_empty() { None } else { Some(document_cpf.read().clone()) };
        let rg_val = if document_rg.read().is_empty() { None } else { Some(document_rg.read().clone()) };
        let email_val = if email.read().is_empty() { None } else { Some(email.read().clone()) };
        let guardian_n = if guardian_name.read().is_empty() { None } else { Some(guardian_name.read().clone()) };
        let notes_val = if notes.read().is_empty() { None } else { Some(notes.read().clone()) };
        let plan_val = if insurance_plan.read().is_empty() { None } else { Some(insurance_plan.read().clone()) };
        let street_val = if address_street.read().is_empty() { None } else { Some(address_street.read().clone()) };
        let number_val = if address_number.read().is_empty() { None } else { Some(address_number.read().clone()) };
        let neigh_val = if address_neighborhood.read().is_empty() { None } else { Some(address_neighborhood.read().clone()) };
        let city_val = if address_city.read().is_empty() { None } else { Some(address_city.read().clone()) };
        let state_val = if address_state.read().is_empty() { None } else { Some(address_state.read().clone()) };
        let zip_val = if address_zip.read().is_empty() { None } else { Some(address_zip.read().clone()) };

        let mut toast_clone = toast.clone();
        let on_save_clone = on_save.clone();

        is_saving.set(true);
        spawn(async move {
            let req = shared::patients::CreatePatientRequest {
                clinic_id,
                full_name: name_val,
                phone: phone_val,
                email: email_val,
                gender: gender_val,
                birth_date: birth_val,
                document_cpf: cpf_val,
                document_rg: rg_val,
                profession: None,
                marital_status: None,
                emergency_contact_name: guardian_n,
                emergency_contact_phone: None,
                legal_guardians: None,
                legal_guardian_name: None,
                legal_guardian_cpf: None,
                insurance_plan: plan_val,
                insurance_number: None,
                address_street: street_val,
                address_number: number_val,
                address_complement: None,
                address_neighborhood: neigh_val,
                address_city: city_val,
                address_state: state_val,
                address_zip: zip_val,
            };

            match PatientsApi::create_patient(req).await {
                Ok(p) => {
                    toast_clone.show("Paciente cadastrado com sucesso!", ToastVariant::Success);
                    on_save_clone.call(p.id);
                }
                Err(e) => {
                    web_sys::console::error_1(&e.clone().into());
                    toast_clone.show(format!("Erro ao salvar paciente: {}", e), ToastVariant::Error);
                }
            }
        });
    };

    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| {
                on_close.call(());
            },

            div { class: "modal-box modal-lg", onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "modal-header",
                    span { class: "modal-title", "Dados do paciente" }
                    button { class: "modal-close-btn", onclick: move |_| on_close.call(()), "✕" }
                }

                // Body
                div { class: "modal-body",

                    // Linha 1: Nome + Sexo
                    div { class: "form-row-2 form-row",
                        div { class: "form-field",
                            label { class: "form-label", "Nome do paciente *" }
                            input {
                                class: if *name_error.read() { "form-input input-error" } else { "form-input" },
                                r#type: "text",
                                placeholder: "Nome completo",
                                value: "{full_name}",
                                oninput: move |e| {
                                    full_name.set(e.value());
                                    if !e.value().is_empty() { name_error.set(false); }
                                },
                            }
                            if *name_error.read() {
                                span { class: "form-error-msg", "Este campo é obrigatório" }
                            }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Sexo" }
                            select { class: "form-select", value: "{gender}",
                                onchange: move |e| gender.set(e.value()),
                                option { value: "", "Selecionar" }
                                option { value: "male", "Masculino" }
                                option { value: "female", "Feminino" }
                                option { value: "other", "Outro" }
                            }
                        }
                    }

                    // Linha 2: Data de nascimento + CPF + RG + Celular
                    div { class: "form-row-4 form-row",
                        div { class: "form-field",
                            label { class: "form-label", "Data de nascimento" }
                            input { class: "form-input", r#type: "date", value: "{birth_date}",
                                oninput: move |e| birth_date.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "CPF" }
                            input { class: "form-input", r#type: "text", placeholder: "000.000.000-00", value: "{document_cpf}",
                                oninput: move |e| document_cpf.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "RG" }
                            input { class: "form-input", r#type: "text", placeholder: "RG", value: "{document_rg}",
                                oninput: move |e| document_rg.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Celular" }
                            input { class: "form-input", r#type: "tel", placeholder: "(00) 00000-0000", value: "{phone}",
                                oninput: move |e| phone.set(e.value()) }
                        }
                    }

                    // E-mail + Como chegou
                    div { class: "form-row-2 form-row",
                        div { class: "form-field",
                            label { class: "form-label", "E-mail" }
                            input { class: "form-input", r#type: "email", placeholder: "email@exemplo.com", value: "{email}",
                                oninput: move |e| email.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Como chegou na clínica" }
                            select { class: "form-select", value: "{how_arrived}",
                                onchange: move |e| how_arrived.set(e.value()),
                                option { value: "", "Selecionar" }
                                option { value: "indication", "Indicação" }
                                option { value: "google", "Google" }
                                option { value: "instagram", "Instagram" }
                                option { value: "facebook", "Facebook" }
                                option { value: "walk_in", "Passou na frente" }
                                option { value: "other", "Outro" }
                            }
                        }
                    }

                    // Abas extras
                    div { class: "tab-underline-bar",
                        button {
                            class: if *active_tab.read() == PatientTab::ExtraInfo { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                            onclick: move |_| active_tab.set(PatientTab::ExtraInfo),
                            "Informações Adicionais"
                        }
                        button {
                            class: if *active_tab.read() == PatientTab::Plan { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                            onclick: move |_| active_tab.set(PatientTab::Plan),
                            "Plano"
                        }
                        button {
                            class: if *active_tab.read() == PatientTab::Address { "tab-underline-btn tab-active" } else { "tab-underline-btn" },
                            onclick: move |_| active_tab.set(PatientTab::Address),
                            "Endereço"
                        }
                    }

                    match *active_tab.read() {
                        PatientTab::ExtraInfo => rsx! {
                            div { class: "form-row-2 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Nome do responsável" }
                                    input { class: "form-input", r#type: "text", placeholder: "Nome completo", value: "{guardian_name}",
                                        oninput: move |e| guardian_name.set(e.value()) }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Celular do responsável" }
                                    input { class: "form-input", r#type: "tel", placeholder: "(00) 00000-0000", value: "{guardian_phone}",
                                        oninput: move |e| guardian_phone.set(e.value()) }
                                }
                            }
                            div { class: "form-field",
                                label { class: "form-label", "Observação" }
                                textarea { class: "form-textarea", placeholder: "Observações gerais sobre o paciente...",
                                    rows: "4", value: "{notes}",
                                    oninput: move |e| notes.set(e.value()) }
                            }
                        },
                        PatientTab::Plan => rsx! {
                            div { class: "form-field",
                                label { class: "form-label", "Plano Odontológico" }
                                input { class: "form-input", r#type: "text", placeholder: "Ex: Unimed Odonto, Amil Dental...", value: "{insurance_plan}",
                                    oninput: move |e| insurance_plan.set(e.value()) }
                            }
                        },
                        PatientTab::Address => rsx! {
                            div { class: "form-row-2 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "CEP" }
                                    input { class: "form-input", r#type: "text", placeholder: "00000-000", value: "{address_zip}",
                                        oninput: move |e| address_zip.set(e.value()) }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Logradouro" }
                                    input { class: "form-input", r#type: "text", placeholder: "Rua, Av.", value: "{address_street}",
                                        oninput: move |e| address_street.set(e.value()) }
                                }
                            }
                            div { class: "form-row-3 form-row",
                                div { class: "form-field",
                                    label { class: "form-label", "Número" }
                                    input { class: "form-input", r#type: "text", placeholder: "Nº", value: "{address_number}",
                                        oninput: move |e| address_number.set(e.value()) }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Bairro" }
                                    input { class: "form-input", r#type: "text", placeholder: "Bairro", value: "{address_neighborhood}",
                                        oninput: move |e| address_neighborhood.set(e.value()) }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Cidade / Estado" }
                                    div { style: "display: flex; gap: 8px;",
                                        input { class: "form-input", r#type: "text", placeholder: "Cidade", value: "{address_city}",
                                            oninput: move |e| address_city.set(e.value()) }
                                        input { class: "form-input", r#type: "text", placeholder: "UF", style: "max-width: 60px;", value: "{address_state}",
                                            oninput: move |e| address_state.set(e.value()) }
                                    }
                                }
                            }
                        },
                    }
                }

                // Footer
                div { class: "modal-footer",
                    button { class: "btn-modal-ghost", onclick: move |_| on_close.call(()), "FECHAR" }
                    button {
                        class: "btn-modal-primary",
                        disabled: *is_saving.read(),
                        onclick: handle_save,
                        if *is_saving.read() { "Salvando..." } else { "SALVAR" }
                    }
                }
            }
        }
    }
}
