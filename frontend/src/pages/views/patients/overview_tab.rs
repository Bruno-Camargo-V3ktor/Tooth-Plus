//! # Aba de Visão Geral do Prontuário (Frontend)
//!
//! Apresenta os dados cadastrais, contatos, endereço, status de assinatura digital
//! com opção de redefinição de senha e gestão de responsáveis legais para pacientes menores.

use crate::api::{reset_patient_signature_password, update_patient};
use crate::components::icons::{IconEdit, IconLock, IconPlus, IconRefresh, IconTrash, IconUsers};
use dioxus::prelude::*;
use shared::patients::{Patient, PatientGuardian, UpdatePatientRequest};

/// Componente da aba de Visão Geral do Prontuário do Paciente.
#[component]
pub fn PatientOverviewTab(
    patient: Patient,
    clinic_id: String,
    token: String,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_reset_pwd_modal_open = use_signal(|| false);
    let mut is_resetting_pwd = use_signal(|| false);

    let mut is_add_guardian_modal_open = use_signal(|| false);
    let mut guardian_name = use_signal(String::new);
    let mut guardian_doc = use_signal(String::new);
    let mut guardian_rel = use_signal(|| "Mãe".to_string());
    let mut guardian_phone = use_signal(String::new);
    let mut guardian_email = use_signal(String::new);
    let mut is_saving_guardian = use_signal(|| false);

    let gender_str = patient.gender.as_deref().unwrap_or("-");
    let marital_str = patient.marital_status.as_deref().unwrap_or("-");
    let gender_marital = format!("{} / {}", gender_str, marital_str);

    let em_contact_name = patient.emergency_contact_name.as_deref().unwrap_or("-");
    let em_contact_phone = patient.emergency_contact_phone.as_deref().unwrap_or("-");
    let em_contact_full = format!("{} ({})", em_contact_name, em_contact_phone);

    let street = patient.address_street.as_deref().unwrap_or("-");
    let number = patient.address_number.as_deref().unwrap_or("S/N");
    let logradouro = format!("{}, {}", street, number);

    let comp = patient.address_complement.as_deref().unwrap_or("-");
    let neigh = patient.address_neighborhood.as_deref().unwrap_or("-");
    let comp_neigh = format!("{} - {}", comp, neigh);

    let city = patient.address_city.as_deref().unwrap_or("São Paulo");
    let state = patient.address_state.as_deref().unwrap_or("SP");
    let city_uf = format!("{} - {}", city, state);

    // Identificar menor de idade
    let is_minor = if let Some(ref bd) = patient.birth_date {
        if let Ok(naive) = chrono::NaiveDate::parse_from_str(bd, "%Y-%m-%d") {
            let now = chrono::Local::now().date_naive();
            now.years_since(naive).unwrap_or(0) < 18
        } else {
            !patient.legal_guardians.is_empty()
        }
    } else {
        !patient.legal_guardians.is_empty()
    };

    let tok = token.clone();
    let cid = clinic_id.clone();
    let pat_id = patient.id.clone();
    let on_reload = reload_patient_details.clone();

    let mut handle_reset_password = move |_| {
        let t = tok.clone();
        let p = pat_id.clone();
        let c = cid.clone();
        let mut modal_sig = is_reset_pwd_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut reset_sig = is_resetting_pwd;
        let reload = on_reload.clone();

        reset_sig.set(true);
        spawn(async move {
            match reset_patient_signature_password(&t, &p, &c).await {
                Ok(msg) => {
                    modal_sig.set(false);
                    toast.set(Some(msg));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao resetar senha: {}", e)));
                }
            }
            reset_sig.set(false);
        });
    };

    let tok_g = token.clone();
    let cid_g = clinic_id.clone();
    let pat_g = patient.clone();
    let on_reload_g = reload_patient_details.clone();

    let mut handle_save_guardian = move |e: Event<FormData>| {
        e.prevent_default();
        let name = guardian_name().trim().to_string();
        let phone = guardian_phone().trim().to_string();

        if name.is_empty() || phone.is_empty() {
            let mut err = error_toast;
            err.set(Some("Informe o nome e telefone do responsável legal.".into()));
            return;
        }

        let doc_val = guardian_doc().trim().to_string();
        let (cpf_val, rg_val) = if doc_val.len() >= 11 && !doc_val.contains('.') {
            (Some(doc_val), None)
        } else if doc_val.is_empty() {
            (None, None)
        } else {
            (Some(doc_val), None)
        };

        let new_g = PatientGuardian {
            name,
            document_cpf: cpf_val,
            document_rg: rg_val,
            relationship: guardian_rel(),
            phone,
            email: if guardian_email().trim().is_empty() { None } else { Some(guardian_email().trim().to_string()) },
        };

        let mut updated_guardians = pat_g.legal_guardians.clone();
        updated_guardians.push(new_g);

        let req = UpdatePatientRequest {
            clinic_id: cid_g.clone(),
            full_name: pat_g.full_name.clone(),
            document_cpf: pat_g.document_cpf.clone(),
            document_rg: pat_g.document_rg.clone(),
            legal_guardians: Some(updated_guardians),
            legal_guardian_name: None,
            legal_guardian_cpf: None,
            phone: pat_g.phone.clone(),
            email: pat_g.email.clone(),
            birth_date: pat_g.birth_date.clone(),
            gender: pat_g.gender.clone(),
            marital_status: pat_g.marital_status.clone(),
            profession: pat_g.profession.clone(),
            emergency_contact_name: pat_g.emergency_contact_name.clone(),
            emergency_contact_phone: pat_g.emergency_contact_phone.clone(),
            address_street: pat_g.address_street.clone(),
            address_number: pat_g.address_number.clone(),
            address_complement: pat_g.address_complement.clone(),
            address_neighborhood: pat_g.address_neighborhood.clone(),
            address_city: pat_g.address_city.clone(),
            address_state: pat_g.address_state.clone(),
            address_zip: pat_g.address_zip.clone(),
            insurance_plan: pat_g.insurance_plan.clone(),
            insurance_number: pat_g.insurance_number.clone(),
        };

        let t = tok_g.clone();
        let p = pat_g.id.clone();
        let mut modal_sig = is_add_guardian_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut sub_sig = is_saving_guardian;
        let reload = on_reload_g.clone();

        sub_sig.set(true);
        spawn(async move {
            match update_patient(&t, &p, req).await {
                Ok(_) => {
                    modal_sig.set(false);
                    toast.set(Some("Responsável legal adicionado com sucesso!".into()));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao adicionar responsável: {}", e)));
                }
            }
            sub_sig.set(false);
        });
    };


    rsx! {
        div { class: "overview-two-cards-grid",
            // Card 1: Dados Cadastrais e Contatos
            div { class: "overview-details-card",
                h3 { class: "overview-details-card-title", "Dados Cadastrais e Contatos" }

                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Nome Completo" }
                    span { class: "overview-details-val", "{patient.full_name}" }
                }
                if let Some(ref cpf) = patient.document_cpf {
                    div { class: "overview-details-row",
                        span { class: "overview-details-label", "CPF (Protegido)" }
                        span { class: "overview-details-val font-mono", "{cpf}" }
                    }
                }
                if let Some(ref rg) = patient.document_rg {
                    div { class: "overview-details-row",
                        span { class: "overview-details-label", "RG (Registro Geral)" }
                        span { class: "overview-details-val font-mono", "{rg}" }
                    }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Telefone / WhatsApp" }
                    span { class: "overview-details-val", "{patient.phone}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "E-mail" }
                    span { class: "overview-details-val", "{patient.email.as_deref().unwrap_or(\"-\")}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Sexo / Estado Civil" }
                    span { class: "overview-details-val", "{gender_marital}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Profissão" }
                    span { class: "overview-details-val", "{patient.profession.as_deref().unwrap_or(\"-\")}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Contato de Emergência" }
                    span { class: "overview-details-val", "{em_contact_full}" }
                }
            }

            // Card 2: Endereço, Convênio e Assinatura Digital
            div { class: "overview-details-card",
                h3 { class: "overview-details-card-title", "Endereço e Convênio" }

                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Logradouro" }
                    span { class: "overview-details-val", "{logradouro}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Complemento / Bairro" }
                    span { class: "overview-details-val", "{comp_neigh}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Cidade / UF" }
                    span { class: "overview-details-val", "{city_uf}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "CEP" }
                    span { class: "overview-details-val font-mono", "{patient.address_zip.as_deref().unwrap_or(\"-\")}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Plano / Convênio" }
                    span { class: "overview-details-val", "{patient.insurance_plan.as_deref().unwrap_or(\"Particular\")}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Nº da Carteirinha" }
                    span { class: "overview-details-val font-mono", "{patient.insurance_number.as_deref().unwrap_or(\"-\")}" }
                }
                div { class: "overview-details-row", style: "align-items: center;",
                    span { class: "overview-details-label", "Senha de Assinatura Digital" }
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        span { class: "overview-details-val text-income",
                            if patient.has_signature_password {
                                "✓ Ativa"
                            } else {
                                "Pendente no Portal"
                            }
                        }
                        if can_write && patient.has_signature_password {
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "padding: 3px 8px; font-size: 11px;",
                                title: "Redefinir ou limpar senha de assinatura esquecida",
                                onclick: move |_| is_reset_pwd_modal_open.set(true),
                                IconRefresh { size: 12, color: "currentColor".to_string() }
                                span { " Limpar Senha" }
                            }
                        }
                    }
                }
            }

            // Card 3: Responsáveis Legais (Para Menores de Idade)
            if is_minor {
                div { class: "overview-details-card full-width", style: "grid-column: 1 / -1;",
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                        div {
                            h3 { class: "overview-details-card-title", style: "margin-bottom: 2px;",
                                IconUsers { size: 16, color: "#0052cc".to_string() }
                                span { " Responsáveis Legais (Paciente Menor de Idade)" }
                            }
                            p { class: "text-muted font-xs", "Cadastro de pais, mães ou tutores legais com consentimento para assinaturas e procedimentos." }
                        }
                        if can_write {
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                style: "font-size: 12px;",
                                onclick: move |_| is_add_guardian_modal_open.set(true),
                                IconPlus { size: 14, color: "currentColor".to_string() }
                                span { " Adicionar Responsável" }
                            }
                        }
                    }

                    if patient.legal_guardians.is_empty() {
                        div { class: "resource-empty-state",
                            "Nenhum responsável legal cadastrado para este menor. Clique em '+ Adicionar Responsável' para informar Pai, Mãe ou Tutor."
                        }
                    } else {
                        div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 12px;",
                            for g in patient.legal_guardians.iter() {
                                {
                                    let g_name_del = g.name.clone();
                                    let tok_rem = token.clone();
                                    let cid_rem = clinic_id.clone();
                                    let pat_rem = patient.clone();
                                    let on_reload_rem = reload_patient_details.clone();

                                    rsx! {
                                        div {
                                            key: "{g.name}",
                                            style: "padding: 12px 14px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; display: flex; flex-direction: column; gap: 4px; position: relative;",
                                            div { style: "display: flex; justify-content: space-between; align-items: center;",
                                                span { class: "badge-insurance-plan", "{g.relationship}" }
                                                if can_write {
                                                    button {
                                                        class: "btn-icon-danger",
                                                        onclick: move |_| {
                                                            let mut updated_guardians = pat_rem.legal_guardians.clone();
                                                            updated_guardians.retain(|g| g.name != g_name_del);

                                                            let req = UpdatePatientRequest {
                                                                clinic_id: cid_rem.clone(),
                                                                full_name: pat_rem.full_name.clone(),
                                                                document_cpf: pat_rem.document_cpf.clone(),
                                                                document_rg: pat_rem.document_rg.clone(),
                                                                legal_guardians: Some(updated_guardians),
                                                                legal_guardian_name: None,
                                                                legal_guardian_cpf: None,
                                                                phone: pat_rem.phone.clone(),
                                                                email: pat_rem.email.clone(),
                                                                birth_date: pat_rem.birth_date.clone(),
                                                                gender: pat_rem.gender.clone(),
                                                                marital_status: pat_rem.marital_status.clone(),
                                                                profession: pat_rem.profession.clone(),
                                                                emergency_contact_name: pat_rem.emergency_contact_name.clone(),
                                                                emergency_contact_phone: pat_rem.emergency_contact_phone.clone(),
                                                                address_street: pat_rem.address_street.clone(),
                                                                address_number: pat_rem.address_number.clone(),
                                                                address_complement: pat_rem.address_complement.clone(),
                                                                address_neighborhood: pat_rem.address_neighborhood.clone(),
                                                                address_city: pat_rem.address_city.clone(),
                                                                address_state: pat_rem.address_state.clone(),
                                                                address_zip: pat_rem.address_zip.clone(),
                                                                insurance_plan: pat_rem.insurance_plan.clone(),
                                                                insurance_number: pat_rem.insurance_number.clone(),
                                                            };

                                                            let t = tok_rem.clone();
                                                            let p = pat_rem.id.clone();
                                                            let mut toast = toast_msg;
                                                            let mut err_sig = error_toast;
                                                            let reload = on_reload_rem.clone();

                                                            spawn(async move {
                                                                match update_patient(&t, &p, req).await {
                                                                    Ok(_) => {
                                                                        toast.set(Some("Responsável legal removido.".into()));
                                                                        reload.call(());
                                                                    }
                                                                    Err(e) => {
                                                                        err_sig.set(Some(format!("Erro ao remover responsável: {}", e)));
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        title: "Remover responsável",
                                                        IconTrash { size: 14, color: "#ef4444".to_string() }
                                                    }
                                                }
                                            }
                                            p { style: "font-size: 14px; font-weight: 600; color: #1e293b; margin: 4px 0 2px 0;", "{g.name}" }
                                            if let Some(ref doc) = g.document_cpf.as_ref().or(g.document_rg.as_ref()) {
                                                p { style: "font-size: 12px; color: #64748b; margin: 0;", "Doc: {doc}" }
                                            }
                                            p { style: "font-size: 12px; color: #64748b; margin: 0;", "Tel / WhatsApp: {g.phone}" }
                                            if let Some(ref email) = g.email {
                                                p { style: "font-size: 12px; color: #64748b; margin: 0;", "E-mail: {email}" }
                                            }
                                        }
                                    }
                                }
                            }

                        }
                    }
                }
            }
        }

        // Modal de Confirmação: Limpar Senha de Assinatura
        if is_reset_pwd_modal_open() {
            div { class: "modal-overlay",
                div { class: "action-modal delete-modal-card",
                    div { class: "settings-header",
                        h2 { class: "settings-title text-danger", "Redefinir Senha de Assinatura" }
                        button { class: "close-btn", onclick: move |_| is_reset_pwd_modal_open.set(false), "×" }
                    }
                    div { class: "settings-content",
                        p { "Deseja realmente limpar a senha de assinatura de ", strong { "{patient.full_name}" }, "?" }
                        p { class: "text-muted font-xs mt-2",
                            "O paciente poderá cadastrar uma nova senha de 6 dígitos ao acessar qualquer link de assinatura enviado para ele no portal."
                        }
                    }
                    div { class: "modal-footer-actions",
                        button { class: "btn-secondary", onclick: move |_| is_reset_pwd_modal_open.set(false), "Cancelar" }
                        button {
                            class: "btn-danger",
                            disabled: is_resetting_pwd(),
                            onclick: move |e| handle_reset_password(e),
                            if is_resetting_pwd() { "Limpando..." } else { "Confirmar Limpeza de Senha" }
                        }
                    }
                }
            }
        }

        // Modal: Adicionar Responsável Legal
        if is_add_guardian_modal_open() {
            div { class: "modal-overlay",
                div { class: "action-modal stock-custom-modal", style: "max-width: 540px;",
                    div { class: "settings-header",
                        h2 { class: "settings-title", "Adicionar Responsável Legal" }
                        button { class: "close-btn", onclick: move |_| is_add_guardian_modal_open.set(false), "×" }
                    }
                    form { onsubmit: move |e| handle_save_guardian(e),
                        div { class: "settings-content",
                            div { class: "form-grid-2",
                                div { class: "form-group full-width", style: "grid-column: 1 / -1;",
                                    label { "Nome Completo do Responsável *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "Ex: Maria Silva",
                                        value: "{guardian_name}",
                                        oninput: move |e| guardian_name.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "Parentesco / Relação *" }
                                    select {
                                        class: "form-input",
                                        value: "{guardian_rel}",
                                        onchange: move |e| guardian_rel.set(e.value()),
                                        option { value: "Mãe", "Mãe" }
                                        option { value: "Pai", "Pai" }
                                        option { value: "Tutor Legal", "Tutor Legal" }
                                        option { value: "Avô / Avó", "Avô / Avó" }
                                        option { value: "Outro", "Outro" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "CPF ou RG" }
                                    input {
                                        class: "form-input",
                                        placeholder: "000.000.000-00",
                                        value: "{guardian_doc}",
                                        oninput: move |e| guardian_doc.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "WhatsApp / Telefone *" }
                                    input {
                                        class: "form-input",
                                        placeholder: "(11) 90000-0000",
                                        value: "{guardian_phone}",
                                        oninput: move |e| guardian_phone.set(e.value())
                                    }
                                }
                                div { class: "form-group",
                                    label { "E-mail" }
                                    input {
                                        class: "form-input",
                                        r#type: "email",
                                        placeholder: "responsavel@email.com",
                                        value: "{guardian_email}",
                                        oninput: move |e| guardian_email.set(e.value())
                                    }
                                }
                            }
                        }
                        div { class: "modal-footer-actions",
                            button {
                                r#type: "button",
                                class: "btn-secondary",
                                onclick: move |_| is_add_guardian_modal_open.set(false),
                                "Cancelar"
                            }
                            button {
                                r#type: "submit",
                                class: "btn-primary",
                                disabled: is_saving_guardian(),
                                if is_saving_guardian() { "Salvando..." } else { "Salvar Responsável" }
                            }
                        }
                    }
                }
            }
        }
    }
}

