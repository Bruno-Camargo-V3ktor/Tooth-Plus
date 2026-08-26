use crate::api::documents::DocumentsApi;
use crate::icons::IconLock;
use shared::documents::{PatientSignAuthRequest, SignAuthResponse};
use dioxus::prelude::*;

#[component]
pub fn AuthCard(
    token: String,
    on_authenticated: EventHandler<SignAuthResponse>,
) -> Element {
    let mut cpf = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut is_submitting = use_signal(|| false);

    let handle_login = {
        let tok = token.clone();
        let mut err_sig = error_msg;
        let mut loading_sig = is_submitting;
        let mut cpf_sig = cpf;
        let mut pass_sig = password;

        move |_| {
            let cpf_val = cpf_sig.read().trim().to_string();
            let pass_val = pass_sig.read().trim().to_string();

            if cpf_val.is_empty() {
                err_sig.set(Some("Informe seu CPF para continuar.".to_string()));
                return;
            }

            err_sig.set(None);
            loading_sig.set(true);

            let tok_clone = tok.clone();
            let mut auth_sig = on_authenticated;
            let mut load_c = loading_sig;
            let mut err_c = err_sig;

            spawn(async move {
                let req = PatientSignAuthRequest {
                    cpf: cpf_val,
                    password: pass_val,
                };
                match DocumentsApi::authenticate_patient(&tok_clone, req).await {
                    Ok(res) => {
                        auth_sig.call(res);
                    }
                    Err(err) => {
                        err_c.set(Some(err));
                        load_c.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "portal-card",
            div { style: "display: flex; align-items: center; gap: 10px;",
                IconLock { size: 22, color: "#00a0e4".to_string() }
                div {
                    h3 { style: "font-size: 16px; font-weight: 800; color: #ffffff; margin: 0;", "Identificação do Signatário" }
                    p { style: "font-size: 12px; color: #94a3b8; margin: 0;", "Confirme seus dados para acessar e assinar o documento." }
                }
            }

            if let Some(err) = error_msg() {
                div { style: "background: rgba(239, 68, 68, 0.12); border: 1px solid rgba(239, 68, 68, 0.3); padding: 10px; border-radius: 6px; font-size: 12px; color: #f87171;",
                    "{err}"
                }
            }

            div { class: "form-field",
                label { class: "form-label", "CPF do Paciente / Responsável *" }
                input {
                    class: "form-input",
                    r#type: "text",
                    placeholder: "000.000.000-00",
                    value: "{cpf}",
                    oninput: move |e| cpf.set(e.value()),
                }
            }

            div { class: "form-field",
                label { class: "form-label", "Senha de Assinatura (Opcional)" }
                input {
                    class: "form-input",
                    r#type: "password",
                    placeholder: "Digite sua senha se cadastrada...",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                }
            }

            button {
                r#type: "button",
                class: "btn-primary",
                style: "width: 100%; height: 42px; font-size: 14px; font-weight: 700;",
                disabled: is_submitting(),
                onclick: handle_login,
                if is_submitting() {
                    "Validando..."
                } else {
                    "Continuar para Assinatura →"
                }
            }
        }
    }
}
