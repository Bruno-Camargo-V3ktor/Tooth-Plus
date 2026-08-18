//! # Formulário Modal de Cadastro de Paciente (Frontend)
//!
//! Componente modal para inserção de novo paciente com validação de campos
//! obrigatórios (Nome, CPF ou RG, Telefone) e endereço/convênio.

use crate::api::create_patient;
use dioxus::prelude::*;
use shared::patients::CreatePatientRequest;

/// Modal para criação e cadastro inicial de paciente na clínica.
#[component]
pub fn PatientFormModal(
    token: String,
    clinic_id: String,
    is_open: Signal<bool>,
    reload_trigger: Signal<usize>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut form_full_name = use_signal(String::new);
    let mut form_cpf = use_signal(String::new);
    let mut form_rg = use_signal(String::new);
    let mut form_phone = use_signal(String::new);
    let mut form_email = use_signal(String::new);
    let mut form_birth_date = use_signal(String::new);
    let mut form_gender = use_signal(|| "Masculino".to_string());
    let mut form_marital_status = use_signal(|| "Solteiro(a)".to_string());
    let mut form_profession = use_signal(String::new);
    let mut form_em_name = use_signal(String::new);
    let mut form_em_phone = use_signal(String::new);
    let mut form_street = use_signal(String::new);
    let mut form_num = use_signal(String::new);
    let mut form_comp = use_signal(String::new);
    let mut form_neigh = use_signal(String::new);
    let mut form_city = use_signal(|| "São Paulo".to_string());
    let mut form_state = use_signal(|| "SP".to_string());
    let mut form_zip = use_signal(String::new);
    let mut form_insurance = use_signal(|| "Particular".to_string());
    let mut form_insurance_num = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);

    if !is_open() {
        return rsx! {};
    }

    let tok = token.clone();
    let cid = clinic_id.clone();

    let mut handle_submit = move |_| {
        let name = form_full_name().trim().to_string();
        let phone = form_phone().trim().to_string();
        let cpf = form_cpf().trim().to_string();
        let rg = form_rg().trim().to_string();

        if name.is_empty() || phone.is_empty() {
            let mut err = error_toast;
            err.set(Some("Preencha ao menos Nome Completo e Telefone/WhatsApp.".into()));
            return;
        }

        if cpf.is_empty() && rg.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe ao menos o CPF ou RG do paciente.".into()));
            return;
        }

        let req = CreatePatientRequest {
            clinic_id: cid.clone(),
            full_name: name,
            document_cpf: cpf,
            document_rg: if rg.is_empty() { None } else { Some(rg) },
            legal_guardian_name: None,
            legal_guardian_cpf: None,
            phone,
            email: if form_email().trim().is_empty() { None } else { Some(form_email().trim().to_string()) },
            birth_date: if form_birth_date().trim().is_empty() { None } else { Some(form_birth_date().trim().to_string()) },
            gender: Some(form_gender()),
            marital_status: Some(form_marital_status()),
            profession: if form_profession().trim().is_empty() { None } else { Some(form_profession().trim().to_string()) },
            emergency_contact_name: if form_em_name().trim().is_empty() { None } else { Some(form_em_name().trim().to_string()) },
            emergency_contact_phone: if form_em_phone().trim().is_empty() { None } else { Some(form_em_phone().trim().to_string()) },
            address_street: if form_street().trim().is_empty() { None } else { Some(form_street().trim().to_string()) },
            address_number: if form_num().trim().is_empty() { None } else { Some(form_num().trim().to_string()) },
            address_complement: if form_comp().trim().is_empty() { None } else { Some(form_comp().trim().to_string()) },
            address_neighborhood: if form_neigh().trim().is_empty() { None } else { Some(form_neigh().trim().to_string()) },
            address_city: Some(form_city()),
            address_state: Some(form_state()),
            address_zip: if form_zip().trim().is_empty() { None } else { Some(form_zip().trim().to_string()) },
            insurance_plan: Some(form_insurance()),
            insurance_number: if form_insurance_num().trim().is_empty() { None } else { Some(form_insurance_num().trim().to_string()) },
            signature_password: None,
        };

        let t = tok.clone();
        let mut open_sig = is_open;
        let mut rel_sig = reload_trigger;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_submitting;

        sub_sig.set(true);
        spawn(async move {
            match create_patient(&t, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    rel_sig.set(rel_sig() + 1);
                    toast.set(Some("Paciente cadastrado com sucesso!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao cadastrar paciente: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal modal-large",
                div { class: "modal-header",
                    div {
                        h2 { class: "modal-title", "Cadastrar Novo Paciente" }
                        p { class: "modal-subtitle", "Cadastre os dados pessoais, endereço e convênio do paciente no prontuário digital." }
                    }
                    button {
                        class: "modal-close",
                        onclick: move |_| {
                            let mut o = is_open;
                            o.set(false);
                        },
                        "×"
                    }
                }
                div { class: "modal-body scrollable",
                    div { class: "form-section-title", "Dados Pessoais & Documentos" }
                    div { class: "form-grid-2",
                        div { class: "form-group full-width",
                            label { "Nome Completo *" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Maria Oliveira Santos",
                                value: "{form_full_name}",
                                oninput: move |e| form_full_name.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "CPF (Protegido por Criptografia)" }
                            input {
                                class: "form-input",
                                placeholder: "000.000.000-00",
                                value: "{form_cpf}",
                                oninput: move |e| form_cpf.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "RG / Órgão Emissor" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: 12.345.678-9 SSP",
                                value: "{form_rg}",
                                oninput: move |e| form_rg.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Telefone / WhatsApp *" }
                            input {
                                class: "form-input",
                                placeholder: "(11) 99999-9999",
                                value: "{form_phone}",
                                oninput: move |e| form_phone.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "E-mail" }
                            input {
                                class: "form-input",
                                r#type: "email",
                                placeholder: "paciente@email.com",
                                value: "{form_email}",
                                oninput: move |e| form_email.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Data de Nascimento" }
                            input {
                                class: "form-input",
                                r#type: "date",
                                value: "{form_birth_date}",
                                oninput: move |e| form_birth_date.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Gênero" }
                            select {
                                class: "form-input",
                                value: "{form_gender}",
                                onchange: move |e| form_gender.set(e.value()),
                                option { value: "Masculino", "Masculino" }
                                option { value: "Feminino", "Feminino" }
                                option { value: "Outro", "Outro" }
                            }
                        }
                        div { class: "form-group",
                            label { "Estado Civil" }
                            select {
                                class: "form-input",
                                value: "{form_marital_status}",
                                onchange: move |e| form_marital_status.set(e.value()),
                                option { value: "Solteiro(a)", "Solteiro(a)" }
                                option { value: "Casado(a)", "Casado(a)" }
                                option { value: "Divorciado(a)", "Divorciado(a)" }
                                option { value: "Viúvo(a)", "Viúvo(a)" }
                            }
                        }
                        div { class: "form-group",
                            label { "Profissão" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Arquiteta",
                                value: "{form_profession}",
                                oninput: move |e| form_profession.set(e.value())
                            }
                        }
                    }

                    div { class: "form-section-title", "Convênio & Contato de Emergência" }
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "Plano / Convênio" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Unimed Odonto / Particular",
                                value: "{form_insurance}",
                                oninput: move |e| form_insurance.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Nº da Carteirinha" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: 98765432100",
                                value: "{form_insurance_num}",
                                oninput: move |e| form_insurance_num.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Nome do Contato de Emergência" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: João Santos (Esposo)",
                                value: "{form_em_name}",
                                oninput: move |e| form_em_name.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Telefone de Emergência" }
                            input {
                                class: "form-input",
                                placeholder: "(11) 98888-8888",
                                value: "{form_em_phone}",
                                oninput: move |e| form_em_phone.set(e.value())
                            }
                        }
                    }

                    div { class: "form-section-title", "Endereço Residencial" }
                    div { class: "form-grid-2",
                        div { class: "form-group",
                            label { "CEP" }
                            input {
                                class: "form-input",
                                placeholder: "00000-000",
                                value: "{form_zip}",
                                oninput: move |e| form_zip.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Logradouro / Rua" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Av. Paulista",
                                value: "{form_street}",
                                oninput: move |e| form_street.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Número" }
                            input {
                                class: "form-input",
                                placeholder: "1000",
                                value: "{form_num}",
                                oninput: move |e| form_num.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Complemento / Apto" }
                            input {
                                class: "form-input",
                                placeholder: "Apto 42 Bloco B",
                                value: "{form_comp}",
                                oninput: move |e| form_comp.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Bairro" }
                            input {
                                class: "form-input",
                                placeholder: "Bela Vista",
                                value: "{form_neigh}",
                                oninput: move |e| form_neigh.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { "Cidade" }
                            input {
                                class: "form-input",
                                value: "{form_city}",
                                oninput: move |e| form_city.set(e.value())
                            }
                        }
                    }
                }
                div { class: "modal-footer",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            let mut o = is_open;
                            o.set(false);
                        },
                        "Cancelar"
                    }
                    button {
                        class: "btn-primary",
                        disabled: is_submitting(),
                        onclick: move |e| handle_submit(e),
                        if is_submitting() { "Salvando..." } else { "Salvar Paciente" }
                    }
                }
            }
        }
    }
}
