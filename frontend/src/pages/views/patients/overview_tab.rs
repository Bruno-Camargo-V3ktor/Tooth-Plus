//! # Aba de Visão Geral do Paciente (Frontend)
//!
//! Exibe os dados cadastrais completos, contatos, dados de convênio, endereço,
//! status da assinatura digital e ação de reset da senha de assinatura.

use crate::api::reset_patient_signature_password;
use crate::components::icons::{IconCheckCircle, IconLock, IconRefresh, IconShieldCheck};
use dioxus::prelude::*;
use shared::patients::Patient;

/// Formata data no formato brasileiro DD/MM/AAAA.
fn format_br_date(date_str: &str) -> String {
    let clean = date_str.chars().take(10).collect::<String>();
    let parts: Vec<&str> = clean.split('-').collect();
    if parts.len() == 3 {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        clean
    }
}

/// Componente da aba de Visão Geral com ficha cadastral e gestão de senha de assinatura digital.
#[component]
pub fn PatientOverviewTab(
    patient: Patient,
    token: String,
    clinic_id: String,
    can_write: bool,
    reload_patient_details: EventHandler<()>,
    toast_msg: Signal<Option<String>>,
    error_toast: Signal<Option<String>>,
) -> Element {
    let mut is_resetting = use_signal(|| false);

    let pat_id = patient.id.clone();
    let pat_name = patient.full_name.clone();
    let tok = token.clone();
    let cid = clinic_id.clone();

    let handle_reset_password = move |_| {
        let p_id = pat_id.clone();
        let p_name = pat_name.clone();
        let t = tok.clone();
        let c = cid.clone();
        let mut resetting = is_resetting;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let reload = reload_patient_details.clone();

        resetting.set(true);
        spawn(async move {
            match reset_patient_signature_password(&t, &p_id, &c).await {
                Ok(_) => {
                    toast.set(Some(format!(
                        "Senha de assinatura de {} resetada com sucesso! O paciente poderá cadastrar uma nova senha no portal.",
                        p_name
                    )));
                    reload.call(());
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao resetar senha: {}", e)));
                }
            }
            resetting.set(false);
        });
    };

    rsx! {
        div { class: "patient-overview-grid",
            div { class: "patient-info-card",
                h3 { class: "card-title", "Identificação & Documentos" }
                div { class: "info-row",
                    span { class: "info-label", "Nome Completo:" }
                    strong { class: "info-value", "{patient.full_name}" }
                }
                div { class: "info-row",
                    span { class: "info-label", "CPF:" }
                    span { class: "info-value font-mono", "{patient.document_cpf}" }
                }
                if let Some(ref rg) = patient.document_rg {
                    div { class: "info-row",
                        span { class: "info-label", "RG:" }
                        span { class: "info-value font-mono", "{rg}" }
                    }
                }
                if let Some(ref bdate) = patient.birth_date {
                    div { class: "info-row",
                        span { class: "info-label", "Data de Nascimento:" }
                        span { class: "info-value", "{format_br_date(bdate)}" }
                    }
                }
                if let Some(ref g) = patient.gender {
                    div { class: "info-row",
                        span { class: "info-label", "Gênero:" }
                        span { class: "info-value", "{g}" }
                    }
                }
                if let Some(ref ms) = patient.marital_status {
                    div { class: "info-row",
                        span { class: "info-label", "Estado Civil:" }
                        span { class: "info-value", "{ms}" }
                    }
                }
                if let Some(ref prof) = patient.profession {
                    div { class: "info-row",
                        span { class: "info-label", "Profissão:" }
                        span { class: "info-value", "{prof}" }
                    }
                }
            }

            div { class: "patient-info-card",
                h3 { class: "card-title", "Contatos & Emergência" }
                div { class: "info-row",
                    span { class: "info-label", "Telefone / WhatsApp:" }
                    strong { class: "info-value", "{patient.phone}" }
                }
                if let Some(ref email) = patient.email {
                    div { class: "info-row",
                        span { class: "info-label", "E-mail:" }
                        span { class: "info-value", "{email}" }
                    }
                }
                if let Some(ref em_name) = patient.emergency_contact_name {
                    div { class: "info-row",
                        span { class: "info-label", "Contato de Emergência:" }
                        span { class: "info-value", "{em_name}" }
                    }
                }
                if let Some(ref em_phone) = patient.emergency_contact_phone {
                    div { class: "info-row",
                        span { class: "info-label", "Tel. de Emergência:" }
                        span { class: "info-value", "{em_phone}" }
                    }
                }
                if let Some(ref plan) = patient.insurance_plan {
                    div { class: "info-row",
                        span { class: "info-label", "Convênio / Plano:" }
                        span { class: "info-value badge-insurance", "{plan}" }
                    }
                }
                if let Some(ref num) = patient.insurance_number {
                    div { class: "info-row",
                        span { class: "info-label", "Nº Carteirinha:" }
                        span { class: "info-value font-mono", "{num}" }
                    }
                }
            }

            div { class: "patient-info-card full-width",
                h3 { class: "card-title", "Endereço Cadastrado" }
                div { class: "info-row",
                    span { class: "info-label", "Logradouro:" }
                    span { class: "info-value",
                        "{patient.address_street.as_deref().unwrap_or(\"Não informado\")}, "
                        "{patient.address_number.as_deref().unwrap_or(\"S/N\")} "
                        "{patient.address_complement.as_deref().unwrap_or(\"\")}"
                    }
                }
                div { class: "info-row",
                    span { class: "info-label", "Bairro / Cidade / UF:" }
                    span { class: "info-value",
                        "{patient.address_neighborhood.as_deref().unwrap_or(\"\")}, "
                        "{patient.address_city.as_deref().unwrap_or(\"\")} - "
                        "{patient.address_state.as_deref().unwrap_or(\"\")}"
                    }
                }
                if let Some(ref zip) = patient.address_zip {
                    div { class: "info-row",
                        span { class: "info-label", "CEP:" }
                        span { class: "info-value font-mono", "{zip}" }
                    }
                }
            }

            div { class: "patient-info-card full-width signature-security-card",
                div { class: "signature-security-header",
                    div { class: "signature-security-title-group",
                        IconShieldCheck { size: 28, color: "var(--primary-color, #0052cc)".to_string() }
                        div {
                            h3 { class: "card-title mb-0", "Assinatura Digital & Autenticação" }
                            p { class: "security-card-subtitle", "Status do cadastro de senha de 6 dígitos para o portal de assinaturas." }
                        }
                    }
                    div { class: "signature-status-badge-container",
                        if patient.has_signature_password {
                            div { class: "badge-pwd-status badge-pwd-active",
                                IconCheckCircle { size: 14, color: "currentColor".to_string() }
                                span { "Senha Cadastrada" }
                            }
                        } else {
                            div { class: "badge-pwd-status badge-pwd-pending",
                                IconLock { size: 14, color: "currentColor".to_string() }
                                span { "Pendente de Autocadastro" }
                            }
                        }
                    }
                }

                div { class: "signature-security-body",
                    p { class: "security-explainer-text",
                        if patient.has_signature_password {
                            "O paciente já possui uma senha de 6 dígitos definida para assinar termos e contratos digitalmente. Caso o paciente tenha esquecido a senha, utilize o botão abaixo para resetá-la."
                        } else {
                            "O paciente ainda não possui uma senha cadastrada. Ele poderá criar sua própria senha de 6 dígitos no primeiro acesso ao portal de assinaturas digitais via QR Code ou link WhatsApp."
                        }
                    }

                    if patient.has_signature_password && can_write {
                        div { class: "security-action-container",
                            button {
                                class: "btn-reset-pwd",
                                disabled: is_resetting(),
                                onclick: move |e| handle_reset_password(e),
                                IconRefresh { size: 16, color: "currentColor".to_string() }
                                span { if is_resetting() { "Resetando Senha..." } else { "Resetar Senha de Assinatura" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
