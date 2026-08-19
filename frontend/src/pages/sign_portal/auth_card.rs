//! # Cartões de Autenticação e Seleção de Papel (Frontend)
//!
//! Controla o fluxo de identificação do signatário (Paciente ou Dentista),
//! autocadastro de senha de 6 dígitos para o paciente e login seguro antes da emissão do OTP.

use crate::api::documents::{
    auth_doctor_signing, auth_patient_signing, check_patient_signing, register_patient_password,
};
use crate::components::icons::{IconCheck, IconLock, IconShieldCheck, IconTooth, IconUsers};
use dioxus::prelude::*;
use shared::documents::{
    DoctorSignAuthRequest, PatientCheckResponse, PatientSignAuthRequest, SignAuthResponse,
};

/// Componente de autenticação e identificação do signatário no portal.
#[component]
pub fn SignerAuthCard(
    token: String,
    auth_session: Signal<Option<SignAuthResponse>>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let mut active_role = use_signal(|| "patient".to_string()); // "patient" | "doctor"

    // Patient state
    let mut patient_cpf = use_signal(String::new);
    let mut patient_check_info = use_signal(|| None::<PatientCheckResponse>);
    let mut is_checking_cpf = use_signal(|| false);
    let mut patient_password = use_signal(String::new);
    let mut patient_confirm_pwd = use_signal(String::new);

    // Doctor state
    let mut doctor_username = use_signal(String::new);
    let mut doctor_password = use_signal(String::new);

    let mut is_authenticating = use_signal(|| false);

    // Handler: Verificar CPF ou RG
    let tok_check = token.clone();
    let mut handle_check_cpf = move |_| {
        let doc = patient_cpf().trim().to_string();
        let clean_doc: String = doc.chars().filter(|c| c.is_alphanumeric()).collect();
        if clean_doc.len() < 4 {
            let mut err = error_msg;
            err.set(Some("Digite um CPF ou RG válido.".into()));
            return;
        }

        let t = tok_check.clone();
        let mut check_sig = is_checking_cpf;
        let mut info_sig = patient_check_info;
        let mut err_sig = error_msg;

        check_sig.set(true);
        err_sig.set(None);
        spawn(async move {
            match check_patient_signing(&t, &doc).await {
                Ok(info) => {
                    info_sig.set(Some(info));
                }
                Err(e) => {
                    err_sig.set(Some(e));
                }
            }
            check_sig.set(false);
        });
    };

    // Handler: Autocadastrar senha paciente
    let tok_reg = token.clone();
    let mut handle_register_pwd = move |_| {
        let p1 = patient_password().trim().to_string();
        let p2 = patient_confirm_pwd().trim().to_string();
        if p1.len() < 6 {
            let mut err = error_msg;
            err.set(Some("A senha de assinatura deve ter no mínimo 6 dígitos.".into()));
            return;
        }
        if p1 != p2 {
            let mut err = error_msg;
            err.set(Some("As senhas digitadas não coincidem.".into()));
            return;
        }

        let t = tok_reg.clone();
        let cpf = patient_cpf().trim().to_string();
        let mut auth_sig = auth_session;
        let mut auth_load = is_authenticating;
        let mut err_sig = error_msg;
        let mut succ_sig = success_msg;

        auth_load.set(true);
        err_sig.set(None);
        spawn(async move {
            match register_patient_password(&t, &cpf, &p1).await {
                Ok(resp) => {
                    auth_sig.set(Some(resp));
                    succ_sig.set(Some("Senha cadastrada com sucesso! Prossiga para a validação do código OTP.".into()));
                }
                Err(e) => {
                    err_sig.set(Some(e));
                }
            }
            auth_load.set(false);
        });
    };

    // Handler: Login paciente
    let tok_login_pat = token.clone();
    let mut handle_login_patient = move |_| {
        let p = patient_password().trim().to_string();
        if p.is_empty() {
            let mut err = error_msg;
            err.set(Some("Digite sua senha de assinatura.".into()));
            return;
        }

        let t = tok_login_pat.clone();
        let req = PatientSignAuthRequest {
            cpf: patient_cpf().trim().to_string(),
            password: p,
        };

        let mut auth_sig = auth_session;
        let mut auth_load = is_authenticating;
        let mut err_sig = error_msg;

        auth_load.set(true);
        err_sig.set(None);
        spawn(async move {
            match auth_patient_signing(&t, req).await {
                Ok(resp) => {
                    auth_sig.set(Some(resp));
                }
                Err(e) => {
                    err_sig.set(Some(e));
                }
            }
            auth_load.set(false);
        });
    };

    // Handler: Login dentista
    let tok_login_doc = token.clone();
    let mut handle_login_doctor = move |_| {
        let u = doctor_username().trim().to_string();
        let p = doctor_password().trim().to_string();
        if u.is_empty() || p.is_empty() {
            let mut err = error_msg;
            err.set(Some("Preencha usuário e senha do dentista.".into()));
            return;
        }

        let t = tok_login_doc.clone();
        let req = DoctorSignAuthRequest {
            username: u,
            password: p,
        };

        let mut auth_sig = auth_session;
        let mut auth_load = is_authenticating;
        let mut err_sig = error_msg;

        auth_load.set(true);
        err_sig.set(None);
        spawn(async move {
            match auth_doctor_signing(&t, req).await {
                Ok(resp) => {
                    auth_sig.set(Some(resp));
                }
                Err(e) => {
                    err_sig.set(Some(e));
                }
            }
            auth_load.set(false);
        });
    };

    rsx! {
        div { class: "portal-auth-card",
            div { class: "portal-tabs",
                button {
                    class: if active_role() == "patient" { "portal-tab active" } else { "portal-tab" },
                    onclick: move |_| {
                        active_role.set("patient".to_string());
                        patient_check_info.set(None);
                        error_msg.set(None);
                    },
                    IconUsers { size: 16, color: "currentColor".to_string() }
                    span { "Sou o Paciente" }
                }
                button {
                    class: if active_role() == "doctor" { "portal-tab active" } else { "portal-tab" },
                    onclick: move |_| {
                        active_role.set("doctor".to_string());
                        error_msg.set(None);
                    },
                    IconTooth { size: 16, color: "currentColor".to_string() }
                    span { "Sou o Dentista" }
                }
            }

            if active_role() == "patient" {
                if let Some(ref check) = *patient_check_info.read() {
                    if check.has_password {
                        div { class: "portal-auth-form",
                            div { class: "patient-identified-badge",
                                IconCheck { size: 20, color: "#166534".to_string() }
                                div {
                                    strong { "{check.patient_name}" }
                                    span { class: "patient-id-sub", "Paciente Identificado" }
                                }
                            }
                            div { class: "form-group mt-2",
                                label { class: "portal-label", "Senha de Assinatura *" }
                                input {
                                    class: "portal-input font-mono text-center",
                                    r#type: "password",
                                    placeholder: "••••••",
                                    maxlength: "12",
                                    value: "{patient_password}",
                                    oninput: move |e| patient_password.set(e.value())
                                }
                                p { class: "portal-helper-text", "Digite sua senha de 6 dígitos para autenticar." }
                            }
                            button {
                                class: "portal-btn-primary full-width mt-2",
                                disabled: is_authenticating(),
                                onclick: move |e| handle_login_patient(e),
                                if is_authenticating() { "Autenticando..." } else { "Avançar para Assinatura" }
                            }
                        }
                    } else {
                        div { class: "portal-auth-form",
                            div { class: "patient-identified-badge",
                                IconCheck { size: 20, color: "#166534".to_string() }
                                div {
                                    strong { "{check.patient_name}" }
                                    span { class: "patient-id-sub", "Primeiro Acesso - Crie sua Senha" }
                                }
                            }
                            div { class: "form-group mt-2",
                                label { class: "portal-label", "Crie sua Senha de Assinatura (mínimo 6 dígitos) *" }
                                input {
                                    class: "portal-input font-mono text-center",
                                    r#type: "password",
                                    placeholder: "••••••",
                                    maxlength: "12",
                                    value: "{patient_password}",
                                    oninput: move |e| patient_password.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { class: "portal-label", "Confirme a Senha *" }
                                input {
                                    class: "portal-input font-mono text-center",
                                    r#type: "password",
                                    placeholder: "••••••",
                                    maxlength: "12",
                                    value: "{patient_confirm_pwd}",
                                    oninput: move |e| patient_confirm_pwd.set(e.value())
                                }
                            }
                            button {
                                class: "portal-btn-primary full-width mt-2",
                                disabled: is_authenticating(),
                                onclick: move |e| handle_register_pwd(e),
                                if is_authenticating() { "Cadastrando..." } else { "Cadastrar Senha e Continuar" }
                            }
                        }
                    }
                } else {
                    div { class: "portal-auth-form",
                        p { class: "portal-helper-text", "Para validar sua identidade, digite o número do seu CPF ou RG cadastrado na clínica:" }
                        div { class: "form-group",
                            label { class: "portal-label", "CPF ou RG *" }
                            input {
                                class: "portal-input font-mono text-center",
                                placeholder: "CPF ou RG (ex: 123.456.789-00 ou 12.345.678-9)",
                                value: "{patient_cpf}",
                                oninput: move |e| patient_cpf.set(e.value())
                            }
                            p { class: "portal-helper-text", style: "font-size: 11px; color: #64748b; margin-top: 4px;",
                                "ℹ️ Para pacientes menores de 18 anos, insira o CPF ou RG do responsável legal cadastrado."
                            }
                        }
                        button {
                            class: "portal-btn-primary full-width mt-2",
                            disabled: is_checking_cpf(),
                            onclick: move |e| handle_check_cpf(e),
                            if is_checking_cpf() { "Verificando..." } else { "Verificar Documento" }
                        }
                    }
                }
            } else {
                div { class: "portal-auth-form",
                    p { class: "portal-helper-text", "Acesso restrito ao dentista responsável pelo atendimento:" }
                    div { class: "form-group",
                        label { class: "portal-label", "Usuário / Login do Dentista *" }
                        input {
                            class: "portal-input",
                            placeholder: "Ex: dr.carlos",
                            value: "{doctor_username}",
                            oninput: move |e| doctor_username.set(e.value())
                        }
                    }
                    div { class: "form-group",
                        label { class: "portal-label", "Senha de Acesso *" }
                        input {
                            class: "portal-input",
                            r#type: "password",
                            placeholder: "••••••••",
                            value: "{doctor_password}",
                            oninput: move |e| doctor_password.set(e.value())
                        }
                    }
                    button {
                        class: "portal-btn-primary full-width mt-2",
                        disabled: is_authenticating(),
                        onclick: move |e| handle_login_doctor(e),
                        if is_authenticating() { "Autenticando..." } else { "Entrar como Dentista" }
                    }
                }
            }
        }
    }
}
