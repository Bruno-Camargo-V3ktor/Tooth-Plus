pub mod tab_extra;
pub mod tab_address;
pub mod tab_plan;

use crate::api::patients::PatientsApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use crate::icons::IconClose;
use shared::patients::CreatePatientRequest;
use dioxus::prelude::*;

pub use tab_extra::TabExtra;
pub use tab_address::TabAddress;
pub use tab_plan::TabPlan;

#[derive(Props, Clone, PartialEq)]
pub struct PatientFormModalProps {
    pub on_save: EventHandler<String>,
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
    let mut toast = consume_context::<ToastState>();

    let mut full_name = use_signal(String::new);
    let mut gender = use_signal(String::new);
    let mut birth_date = use_signal(String::new);
    let mut document_cpf = use_signal(String::new);
    let mut document_rg = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut email = use_signal(String::new);

    let mut active_tab = use_signal(|| PatientTab::ExtraInfo);
    let mut guardian_name = use_signal(String::new);
    let mut guardian_phone = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut insurance_plan = use_signal(String::new);

    let mut address_zip = use_signal(String::new);
    let mut address_street = use_signal(String::new);
    let mut address_number = use_signal(String::new);
    let mut address_neighborhood = use_signal(String::new);
    let mut address_city = use_signal(String::new);
    let mut address_state = use_signal(String::new);

    let mut is_saving = use_signal(|| false);

    let handle_save = {
        let clinic_id = active_clinic
            .read()
            .as_ref()
            .map(|c| c.clinic_id.clone())
            .unwrap_or_default();

        let mut toast_c = toast.clone();
        let on_save_handler = props.on_save;
        let mut saving_sig = is_saving;

        move |_| {
            let name = full_name.read().trim().to_string();
            let phone_val = phone.read().trim().to_string();

            if name.is_empty() {
                toast_c.show("O nome do paciente é obrigatório.", ToastVariant::Error);
                return;
            }

            saving_sig.set(true);

            let req = CreatePatientRequest {
                clinic_id: clinic_id.clone(),
                full_name: name,
                document_cpf: if document_cpf.read().is_empty() { None } else { Some(document_cpf.read().clone()) },
                document_rg: if document_rg.read().is_empty() { None } else { Some(document_rg.read().clone()) },
                legal_guardians: None,
                legal_guardian_name: if guardian_name.read().is_empty() { None } else { Some(guardian_name.read().clone()) },
                legal_guardian_cpf: None,
                phone: phone_val,
                email: if email.read().is_empty() { None } else { Some(email.read().clone()) },
                birth_date: if birth_date.read().is_empty() { None } else { Some(birth_date.read().clone()) },
                gender: if gender.read().is_empty() { None } else { Some(gender.read().clone()) },
                marital_status: None,
                profession: None,
                emergency_contact_name: None,
                emergency_contact_phone: None,
                address_street: if address_street.read().is_empty() { None } else { Some(address_street.read().clone()) },
                address_number: if address_number.read().is_empty() { None } else { Some(address_number.read().clone()) },
                address_complement: None,
                address_neighborhood: if address_neighborhood.read().is_empty() { None } else { Some(address_neighborhood.read().clone()) },
                address_city: if address_city.read().is_empty() { None } else { Some(address_city.read().clone()) },
                address_state: if address_state.read().is_empty() { None } else { Some(address_state.read().clone()) },
                address_zip: if address_zip.read().is_empty() { None } else { Some(address_zip.read().clone()) },
                insurance_plan: if insurance_plan.read().is_empty() { None } else { Some(insurance_plan.read().clone()) },
                insurance_number: None,
            };

            let mut toast_resp = toast_c.clone();
            let mut saving_resp = saving_sig;

            spawn(async move {
                match PatientsApi::create_patient(req).await {
                    Ok(new_pat) => {
                        toast_resp.show("Paciente cadastrado com sucesso!", ToastVariant::Success);
                        saving_resp.set(false);
                        on_save_handler.call(new_pat.id);
                    }
                    Err(err) => {
                        toast_resp.show(err, ToastVariant::Error);
                        saving_resp.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-box modal-patient-form",
                div { class: "modal-header",
                    h3 { class: "modal-title", "Adicionar Paciente" }
                    button {
                        r#type: "button",
                        class: "modal-close-btn",
                        onclick: move |_| props.on_close.call(()),
                        IconClose { size: 18, color: "currentColor".to_string() }
                    }
                }

                div { class: "modal-body",
                    div { class: "form-row-3 form-row",
                        div { class: "form-field",
                            label { class: "form-label", "Nome completo *" }
                            input { class: "form-input", r#type: "text", placeholder: "Ex: Mariana Silva", value: "{full_name}",
                                oninput: move |e| full_name.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Gênero" }
                            select { class: "form-select", value: "{gender}",
                                onchange: move |e| gender.set(e.value()),
                                option { value: "", "Selecionar" }
                                option { value: "female", "Feminino" }
                                option { value: "male", "Masculino" }
                                option { value: "other", "Outro" }
                            }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Data de Nascimento" }
                            input { class: "form-input", r#type: "date", value: "{birth_date}",
                                oninput: move |e| birth_date.set(e.value()) }
                        }
                    }

                    div { class: "form-row-3 form-row",
                        div { class: "form-field",
                            label { class: "form-label", "CPF" }
                            input { class: "form-input", r#type: "text", placeholder: "000.000.000-00", value: "{document_cpf}",
                                oninput: move |e| document_cpf.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "RG" }
                            input { class: "form-input", r#type: "text", placeholder: "00.000.000-0", value: "{document_rg}",
                                oninput: move |e| document_rg.set(e.value()) }
                        }
                        div { class: "form-field",
                            label { class: "form-label", "Celular" }
                            input { class: "form-input", r#type: "tel", placeholder: "(00) 00000-0000", value: "{phone}",
                                oninput: move |e| phone.set(e.value()) }
                        }
                    }

                    div { class: "form-field",
                        label { class: "form-label", "E-mail" }
                        input { class: "form-input", r#type: "email", placeholder: "email@exemplo.com", value: "{email}",
                            oninput: move |e| email.set(e.value()) }
                    }

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
                            TabExtra { guardian_name, guardian_phone, notes }
                        },
                        PatientTab::Plan => rsx! {
                            TabPlan { insurance_plan }
                        },
                        PatientTab::Address => rsx! {
                            TabAddress { address_zip, address_street, address_number, address_neighborhood, address_city, address_state }
                        },
                    }
                }

                div { class: "modal-footer",
                    button { class: "btn-modal-ghost", onclick: move |_| props.on_close.call(()), "FECHAR" }
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
