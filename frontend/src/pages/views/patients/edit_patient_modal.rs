//! # Modal de Edição de Dados Cadastrais do Paciente (Frontend)
//!
//! Permite editar todos os dados cadastrais, pessoais, contatos, convênio e endereço do paciente
//! diretamente a partir da aba Visão Geral do Prontuário.

use crate::api::update_patient;
use crate::components::icons::{IconCheck, IconEdit, IconUsers, IconX};
use dioxus::prelude::*;
use shared::patients::{Patient, UpdatePatientRequest};

#[component]
pub fn EditPatientModal(
    is_open: Signal<bool>,
    patient: Patient,
    clinic_id: String,
    token: String,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut form_full_name = use_signal(|| patient.full_name.clone());
    let mut form_cpf = use_signal(|| patient.document_cpf.clone().unwrap_or_default());
    let mut form_rg = use_signal(|| patient.document_rg.clone().unwrap_or_default());
    let mut form_phone = use_signal(|| patient.phone.clone());
    let mut form_email = use_signal(|| patient.email.clone().unwrap_or_default());
    let mut form_birth_date = use_signal(|| patient.birth_date.clone().unwrap_or_default());
    let mut form_gender = use_signal(|| patient.gender.clone().unwrap_or_else(|| "Masculino".into()));
    let mut form_marital = use_signal(|| patient.marital_status.clone().unwrap_or_else(|| "Solteiro(a)".into()));
    let mut form_profession = use_signal(|| patient.profession.clone().unwrap_or_default());
    let mut form_em_name = use_signal(|| patient.emergency_contact_name.clone().unwrap_or_default());
    let mut form_em_phone = use_signal(|| patient.emergency_contact_phone.clone().unwrap_or_default());

    let mut form_street = use_signal(|| patient.address_street.clone().unwrap_or_default());
    let mut form_number = use_signal(|| patient.address_number.clone().unwrap_or_default());
    let mut form_complement = use_signal(|| patient.address_complement.clone().unwrap_or_default());
    let mut form_neighborhood = use_signal(|| patient.address_neighborhood.clone().unwrap_or_default());
    let mut form_city = use_signal(|| patient.address_city.clone().unwrap_or_else(|| "São Paulo".into()));
    let mut form_state = use_signal(|| patient.address_state.clone().unwrap_or_else(|| "SP".into()));
    let mut form_zip = use_signal(|| patient.address_zip.clone().unwrap_or_default());

    let mut form_insurance = use_signal(|| patient.insurance_plan.clone().unwrap_or_else(|| "Particular".into()));
    let mut form_insurance_num = use_signal(|| patient.insurance_number.clone().unwrap_or_default());

    let mut is_submitting = use_signal(|| false);

    let tok = token.clone();
    let pat_id = patient.id.clone();
    let on_reload = reload_patient_details.clone();

    let mut handle_submit = move |e: Event<FormData>| {
        e.prevent_default();

        let full_name = form_full_name().trim().to_string();
        let cpf = form_cpf().trim().to_string();
        let rg = form_rg().trim().to_string();
        let phone = form_phone().trim().to_string();

        if full_name.is_empty() || phone.is_empty() {
            let mut err = error_toast;
            err.set(Some("Preencha o Nome Completo e WhatsApp / Celular.".into()));
            return;
        }

        if cpf.is_empty() && rg.is_empty() {
            let mut err = error_toast;
            err.set(Some("É obrigatório informar ao menos um documento de identificação (CPF ou RG).".into()));
            return;
        }

        let req = UpdatePatientRequest {
            full_name,
            document_cpf: if cpf.is_empty() { None } else { Some(cpf) },
            document_rg: if rg.is_empty() { None } else { Some(rg) },
            clinic_id: clinic_id.clone(),
            legal_guardians: if patient.legal_guardians.is_empty() { None } else { Some(patient.legal_guardians.clone()) },
            legal_guardian_name: patient.legal_guardian_name.clone(),
            legal_guardian_cpf: patient.legal_guardian_cpf.clone(),
            phone,
            email: if form_email().trim().is_empty() { None } else { Some(form_email().trim().to_string()) },
            birth_date: if form_birth_date().trim().is_empty() { None } else { Some(form_birth_date().trim().to_string()) },
            gender: Some(form_gender()),
            marital_status: if form_marital().trim().is_empty() { None } else { Some(form_marital().trim().to_string()) },
            profession: if form_profession().trim().is_empty() { None } else { Some(form_profession().trim().to_string()) },
            emergency_contact_name: if form_em_name().trim().is_empty() { None } else { Some(form_em_name().trim().to_string()) },
            emergency_contact_phone: if form_em_phone().trim().is_empty() { None } else { Some(form_em_phone().trim().to_string()) },
            address_street: if form_street().trim().is_empty() { None } else { Some(form_street().trim().to_string()) },
            address_number: if form_number().trim().is_empty() { None } else { Some(form_number().trim().to_string()) },
            address_complement: if form_complement().trim().is_empty() { None } else { Some(form_complement().trim().to_string()) },
            address_neighborhood: if form_neighborhood().trim().is_empty() { None } else { Some(form_neighborhood().trim().to_string()) },
            address_city: if form_city().trim().is_empty() { None } else { Some(form_city().trim().to_string()) },
            address_state: if form_state().trim().is_empty() { None } else { Some(form_state().trim().to_string()) },
            address_zip: if form_zip().trim().is_empty() { None } else { Some(form_zip().trim().to_string()) },
            insurance_plan: if form_insurance().trim().is_empty() { None } else { Some(form_insurance().trim().to_string()) },
            insurance_number: if form_insurance_num().trim().is_empty() { None } else { Some(form_insurance_num().trim().to_string()) },
        };

        let t = tok.clone();
        let p = pat_id.clone();
        let mut open_sig = is_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_submitting;
        let reload = on_reload.clone();

        sub_sig.set(true);
        spawn(async move {
            match update_patient(&t, &p, req).await {
                Ok(_) => {
                    open_sig.set(false);
                    toast.set(Some("Dados cadastrais do paciente atualizados com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao atualizar dados: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "action-modal patient-custom-modal", style: "max-width: 780px;",
                div { class: "settings-header",
                    div {
                        h2 { class: "settings-title",
                            IconEdit { size: 20, color: "#0052cc".to_string() }
                            span { " Editar Dados do Paciente" }
                        }
                        p { class: "text-muted font-xs mt-1",
                            "Atualize as informações de identificação, contato, endereço e convênio do paciente."
                        }
                    }
                    button {
                        class: "close-btn",
                        onclick: move |_| {
                            let mut o = is_open;
                            o.set(false);
                        },
                        "×"
                    }
                }

                form { onsubmit: handle_submit,
                    div { class: "settings-content", style: "max-height: 68vh; overflow-y: auto; padding-right: 4px;",
                        // Seção 1: Identificação & Documentos
                        div { class: "agenda-resource-box", style: "margin-bottom: 16px;",
                            div { class: "resource-section-header",
                                span { "1. Identificação e Documentos" }
                            }
                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group full-width", style: "grid-column: 1 / -1;",
                                    label { "Nome Completo *" }
                                    input {
                                        class: "form-input",
                                        required: true,
                                        placeholder: "Nome completo do paciente",
                                        value: "{form_full_name}",
                                        oninput: move |e| form_full_name.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "CPF (Protegido por Criptografia)" }
                                    input {
                                        class: "form-input font-mono",
                                        placeholder: "000.000.000-00",
                                        value: "{form_cpf}",
                                        oninput: move |e| form_cpf.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "RG (Registro Geral)" }
                                    input {
                                        class: "form-input font-mono",
                                        placeholder: "00.000.000-0",
                                        value: "{form_rg}",
                                        oninput: move |e| form_rg.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Data de Nascimento" }
                                    input {
                                        r#type: "date",
                                        class: "form-input",
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
                                        option { value: "Não Informado", "Não Informado" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Estado Civil" }
                                    select {
                                        class: "form-input",
                                        value: "{form_marital}",
                                        onchange: move |e| form_marital.set(e.value()),
                                        option { value: "Solteiro(a)", "Solteiro(a)" }
                                        option { value: "Casado(a)", "Casado(a)" }
                                        option { value: "Divorciado(a)", "Divorciado(a)" }
                                        option { value: "Viúvo(a)", "Viúvo(a)" }
                                        option { value: "União Estável", "União Estável" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Profissão / Ocupação" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Arquiteto(a), Estudante...",
                                        value: "{form_profession}",
                                        oninput: move |e| form_profession.set(e.value())
                                    }
                                }
                            }
                        }

                        // Seção 2: Contatos & Emergência
                        div { class: "agenda-resource-box", style: "margin-bottom: 16px;",
                            div { class: "resource-section-header",
                                span { "2. Contatos e Emergência" }
                            }
                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group",
                                    label { "WhatsApp / Celular *" }
                                    input {
                                        class: "form-input",
                                        required: true,
                                        placeholder: "(11) 99999-9999",
                                        value: "{form_phone}",
                                        oninput: move |e| form_phone.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "E-mail" }
                                    input {
                                        r#type: "email",
                                        class: "form-input",
                                        placeholder: "paciente@email.com",
                                        value: "{form_email}",
                                        oninput: move |e| form_email.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Nome Contato de Emergência" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Nome de um familiar ou amigo",
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
                        }

                        // Seção 3: Endereço Residencial
                        div { class: "agenda-resource-box", style: "margin-bottom: 16px;",
                            div { class: "resource-section-header",
                                span { "3. Endereço Residencial" }
                            }
                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group",
                                    label { "Logradouro / Rua / Avenida" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Rua das Flores",
                                        value: "{form_street}",
                                        oninput: move |e| form_street.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Número" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: 123",
                                        value: "{form_number}",
                                        oninput: move |e| form_number.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Complemento" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Apto 42, Bloco B...",
                                        value: "{form_complement}",
                                        oninput: move |e| form_complement.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Bairro" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Centro, Jardins...",
                                        value: "{form_neighborhood}",
                                        oninput: move |e| form_neighborhood.set(e.value())
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
                                div { class: "form-group",
                                    label { "Estado (UF)" }
                                    input {
                                        class: "form-input",
                                        value: "{form_state}",
                                        oninput: move |e| form_state.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "CEP" }
                                    input {
                                        class: "form-input font-mono",
                                        placeholder: "00000-000",
                                        value: "{form_zip}",
                                        oninput: move |e| form_zip.set(e.value())
                                    }
                                }
                            }
                        }

                        // Seção 4: Convênio / Plano Odontológico
                        div { class: "agenda-resource-box",
                            div { class: "resource-section-header",
                                span { "4. Convênio e Plano" }
                            }
                            div { class: "form-grid-2", style: "margin-top: 10px;",
                                div { class: "form-group",
                                    label { "Plano / Convênio" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Particular, Bradesco Dental, Amil...",
                                        value: "{form_insurance}",
                                        oninput: move |e| form_insurance.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Nº da Carteirinha" }
                                    input {
                                        class: "form-input font-mono",
                                        placeholder: "Ex: 987654321",
                                        value: "{form_insurance_num}",
                                        oninput: move |e| form_insurance_num.set(e.value())
                                    }
                                }
                            }
                        }
                    }

                    div { class: "modal-footer-actions",
                        button {
                            r#type: "button",
                            class: "btn-secondary",
                            onclick: move |_| {
                                let mut o = is_open;
                                o.set(false);
                            },
                            "Cancelar"
                        }
                        button {
                            r#type: "submit",
                            class: "btn-primary",
                            disabled: is_submitting(),
                            IconCheck { size: 16, color: "#ffffff".to_string() }
                            span { if is_submitting() { "Salvando..." } else { "Salvar Alterações" } }
                        }
                    }
                }
            }
        }
    }
}
