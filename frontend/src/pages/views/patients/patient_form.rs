//! # Modal de Cadastro de Paciente (Frontend)
//!
//! Modal em 2 colunas com suporte a CPF ou RG protegidos, convênio
//! e dados residenciais de acordo com o design system do Tooth-Plus.

use crate::api::{create_patient, lookup_cep};
use dioxus::prelude::*;
use shared::patients::CreatePatientRequest;

/// Componente Modal para cadastro de novo paciente.
#[component]
pub fn PatientFormModal(
    is_open: Signal<bool>,
    token: String,
    clinic_id: String,
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
    let mut form_insurance = use_signal(|| "Particular".to_string());
    let mut form_zip = use_signal(String::new);
    let mut form_street = use_signal(String::new);
    let mut form_num_comp = use_signal(String::new);
    let mut form_neighborhood = use_signal(String::new);
    let mut form_city = use_signal(|| "São Paulo".to_string());
    let mut form_state = use_signal(|| "SP".to_string());

    let mut is_submitting = use_signal(|| false);
    let mut is_loading_cep = use_signal(|| false);

    let mut handle_cep_input = move |raw: String| {
        let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).take(8).collect();
        let formatted = if digits.len() > 5 {
            format!("{}-{}", &digits[..5], &digits[5..])
        } else {
            digits.clone()
        };
        form_zip.set(formatted);

        if digits.len() == 8 {
            is_loading_cep.set(true);
            let mut street_sig = form_street;
            let mut neigh_sig = form_neighborhood;
            let mut city_sig = form_city;
            let mut state_sig = form_state;
            let mut loading_sig = is_loading_cep;
            let mut toast = toast_msg;
            let mut err_sig = error_toast;

            spawn(async move {
                match lookup_cep(&digits).await {
                    Ok(info) => {
                        let mut filled_any = false;
                        if let Some(logr) = info.logradouro {
                            if !logr.is_empty() {
                                street_sig.set(logr);
                                filled_any = true;
                            }
                        }
                        if let Some(bairro) = info.bairro {
                            if !bairro.is_empty() {
                                neigh_sig.set(bairro);
                                filled_any = true;
                            }
                        }
                        if let Some(cidade) = info.localidade {
                            if !cidade.is_empty() {
                                city_sig.set(cidade);
                                filled_any = true;
                            }
                        }
                        if let Some(uf) = info.uf {
                            if !uf.is_empty() {
                                state_sig.set(uf);
                                filled_any = true;
                            }
                        }
                        if filled_any {
                            toast.set(Some("Endereço preenchido automaticamente pelo CEP!".into()));
                        }
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("CEP não encontrado: {}", e)));
                    }
                }
                loading_sig.set(false);
            });
        }
    };

    let tok = token.clone();
    let cid = clinic_id.clone();

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

        let num_comp_val = form_num_comp().trim().to_string();
        let (num_val, comp_val) = if let Some((n, c)) = num_comp_val.split_once(',') {
            (Some(n.trim().to_string()), Some(c.trim().to_string()))
        } else if !num_comp_val.is_empty() {
            (Some(num_comp_val), None)
        } else {
            (None, None)
        };

        let zip_val = form_zip().trim().to_string();
        let neigh_val = form_neighborhood().trim().to_string();
        let city_val = form_city().trim().to_string();
        let state_val = form_state().trim().to_string();

        let req = CreatePatientRequest {
            clinic_id: cid.clone(),
            full_name,
            document_cpf: if cpf.is_empty() { None } else { Some(cpf) },
            document_rg: if rg.is_empty() { None } else { Some(rg) },
            legal_guardians: None,
            legal_guardian_name: None,
            legal_guardian_cpf: None,
            phone,
            email: if form_email().trim().is_empty() { None } else { Some(form_email().trim().to_string()) },
            birth_date: if form_birth_date().trim().is_empty() { None } else { Some(form_birth_date().trim().to_string()) },
            gender: Some(form_gender()),
            marital_status: None,
            profession: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            address_street: if form_street().trim().is_empty() { None } else { Some(form_street().trim().to_string()) },
            address_number: num_val,
            address_complement: comp_val,
            address_neighborhood: if neigh_val.is_empty() { None } else { Some(neigh_val) },
            address_city: if city_val.is_empty() { None } else { Some(city_val) },
            address_state: if state_val.is_empty() { None } else { Some(state_val) },
            address_zip: if zip_val.is_empty() { None } else { Some(zip_val) },
            insurance_plan: if form_insurance().trim().is_empty() { None } else { Some(form_insurance().trim().to_string()) },
            insurance_number: None,
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
            div { class: "action-modal stock-custom-modal", style: "width: 720px !important; max-width: 95vw !important; max-height: 90vh !important; display: flex !important; flex-direction: column !important; overflow: hidden !important;",
                div { class: "settings-header", style: "flex-shrink: 0;",
                    div {
                        h2 { class: "settings-title", "Cadastrar Novo Paciente" }
                        p { class: "text-muted font-xs mt-1",
                            "Preencha os dados do paciente. Os documentos serão protegidos e a senha de assinatura será definida pelo próprio paciente no primeiro acesso ao portal."
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

                form {
                    onsubmit: move |e| handle_submit(e),
                    style: "display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; margin: 0;",
                    div { class: "settings-content", style: "flex: 1; min-height: 0; overflow-y: auto; padding: 20px 24px;",
                        div { class: "form-grid-2",
                            // Linha 1: Nome Completo (full width)
                            div { class: "form-group full-width", style: "grid-column: 1 / -1;",
                                label { "Nome Completo *" }
                                input {
                                    class: "form-input",
                                    placeholder: "Nome completo do paciente",
                                    value: "{form_full_name}",
                                    oninput: move |e| form_full_name.set(e.value())
                                }
                            }

                            // Linha 2: CPF | RG (pelo menos um)
                            div { class: "form-group",
                                label { "CPF (Obrigatório se não tiver RG)" }
                                input {
                                    class: "form-input",
                                    placeholder: "000.000.000-00",
                                    value: "{form_cpf}",
                                    oninput: move |e| form_cpf.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "RG (Registro Geral)" }
                                input {
                                    class: "form-input",
                                    placeholder: "00.000.000-0",
                                    value: "{form_rg}",
                                    oninput: move |e| form_rg.set(e.value())
                                }
                            }

                            // Linha 3: WhatsApp / Celular | E-mail
                            div { class: "form-group",
                                label { "WhatsApp / Celular *" }
                                input {
                                    class: "form-input",
                                    placeholder: "(11) 90000-0000",
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

                            // Linha 4: Data de Nascimento | Sexo
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
                                label { "Sexo" }
                                select {
                                    class: "form-input",
                                    value: "{form_gender}",
                                    onchange: move |e| form_gender.set(e.value()),
                                    option { value: "Masculino", "Masculino" }
                                    option { value: "Feminino", "Feminino" }
                                    option { value: "Outro", "Outro" }
                                }
                            }

                            // Linha 5: Plano / Convênio
                            div { class: "form-group full-width", style: "grid-column: 1 / -1;",
                                label { "Plano / Convênio" }
                                select {
                                    class: "form-input",
                                    value: "{form_insurance}",
                                    onchange: move |e| form_insurance.set(e.value()),
                                    option { value: "Particular", "Particular" }
                                    option { value: "SulAmérica Odonto Mais", "SulAmérica Odonto Mais" }
                                    option { value: "Amil Dental Premium", "Amil Dental Premium" }
                                    option { value: "Bradesco Dental", "Bradesco Dental" }
                                    option { value: "Unimed Odonto", "Unimed Odonto" }
                                    option { value: "Porto Seguro Odonto", "Porto Seguro Odonto" }
                                    option { value: "MetLife Odonto", "MetLife Odonto" }
                                    option { value: "Outro Convênio", "Outro Convênio" }
                                }
                            }

                            // Linha 6: CEP | Bairro
                            div { class: "form-group",
                                label {
                                    span { "CEP" }
                                    if is_loading_cep() {
                                        span { style: "color: #0052cc; font-size: 11px; font-weight: 600; margin-left: 8px;", " (Buscando endereço...)" }
                                    }
                                }
                                input {
                                    class: "form-input font-mono",
                                    placeholder: "00000-000",
                                    maxlength: "9",
                                    value: "{form_zip}",
                                    oninput: move |e| handle_cep_input(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Bairro" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Jardins, Centro...",
                                    value: "{form_neighborhood}",
                                    oninput: move |e| form_neighborhood.set(e.value())
                                }
                            }

                            // Linha 7: Endereço (Rua/Av) | Número / Complemento
                            div { class: "form-group",
                                label { "Endereço (Rua/Av)" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: Av. Paulista",
                                    value: "{form_street}",
                                    oninput: move |e| form_street.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { "Número / Complemento" }
                                input {
                                    class: "form-input",
                                    placeholder: "Ex: 1000, Apto 42",
                                    value: "{form_num_comp}",
                                    oninput: move |e| form_num_comp.set(e.value())
                                }
                            }

                            // Linha 8: Cidade | Estado (UF)
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
                        }
                    }

                    div {
                        class: "modal-footer-actions",
                        style: "flex-shrink: 0; padding: 16px 24px; border-top: 1px solid #e2e8f0; background: #ffffff; display: flex; justify-content: flex-end; align-items: center; gap: 12px; margin-top: 0;",
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
                            style: "font-weight: 600; padding: 8px 20px;",
                            disabled: is_submitting(),
                            if is_submitting() { "Salvando..." } else { "Salvar Paciente" }
                        }
                    }
                }
            }
        }
    }
}

