use shared::patients::Patient;
use dioxus::prelude::*;

#[component]
pub fn PatientTable(
    patients: Vec<Patient>,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "patients-table-container",
            table { class: "patients-table",
                thead {
                    tr {
                        th { "Nome / Paciente" }
                        th { "CPF" }
                        th { "Telefone" }
                        th { "Plano / Tipo" }
                        th { "Cadastrado em" }
                    }
                }
                tbody {
                    for patient in patients {
                        {
                            let pid = patient.id.clone();
                            let initial = patient.full_name.chars().next().unwrap_or('P').to_string();
                            let cpf_display = patient.document_cpf.clone().unwrap_or_else(|| "Não informado".to_string());
                            let plan_display = patient.insurance_plan.clone().unwrap_or_else(|| "Particular".to_string());
                            let is_particular = patient.insurance_plan.is_none();
                            let created_fmt = patient.created_at.split('T').next().unwrap_or(&patient.created_at).to_string();
                            let email_display = patient.email.clone().unwrap_or_default();

                            rsx! {
                                tr {
                                    key: "{pid}",
                                    class: "patient-row",
                                    onclick: move |_| on_select.call(pid.clone()),

                                    td {
                                        div { class: "patient-cell-name",
                                            div { class: "patient-avatar", "{initial}" }
                                            div {
                                                div { class: "patient-meta-name", "{patient.full_name}" }
                                                div { class: "patient-meta-sub", "{email_display}" }
                                            }
                                        }
                                    }
                                    td { "{cpf_display}" }
                                    td { "{patient.phone}" }
                                    td {
                                        span {
                                            class: if is_particular { "badge-particular" } else { "badge-plan" },
                                            "{plan_display}"
                                        }
                                    }
                                    td { "{created_fmt}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
