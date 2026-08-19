//! # Aba de Visão Geral do Prontuário (Frontend)
//!
//! Apresenta os dados cadastrais, contatos, endereço e status de assinatura digital
//! em dois blocos harmoniosos lado a lado.

use dioxus::prelude::*;
use shared::patients::Patient;

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

    rsx! {
        div { class: "overview-two-cards-grid",
            // Card 1: Dados Cadastrais e Contatos
            div { class: "overview-details-card",
                h3 { class: "overview-details-card-title", "Dados Cadastrais e Contatos" }

                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Nome Completo" }
                    span { class: "overview-details-val", "{patient.full_name}" }
                }
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "CPF (Protegido por Criptografia)" }
                    span { class: "overview-details-val font-mono", "{patient.document_cpf}" }
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

            // Card 2: Endereço e Convênio
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
                div { class: "overview-details-row",
                    span { class: "overview-details-label", "Senha de Assinatura Digital" }
                    span { class: "overview-details-val text-income",
                        if patient.has_signature_password {
                            "✓ Cadastrada e Ativa"
                        } else {
                            "Pendente"
                        }
                    }
                }
            }
        }
    }
}
