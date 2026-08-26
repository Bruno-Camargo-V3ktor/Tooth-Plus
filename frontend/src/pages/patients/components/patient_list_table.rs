use crate::icons::{IconEdit, IconExternalLink, IconWhatsapp};
use shared::patients::Patient;
use dioxus::prelude::*;

fn calculate_age(birth_date_opt: &Option<String>) -> String {
    if let Some(bd) = birth_date_opt.as_ref() {
        if let Some(year_str) = bd.split('-').next() {
            if let Ok(year) = year_str.parse::<i32>() {
                let current_year = 2026;
                let age = current_year - year;
                if age > 0 && age < 120 {
                    return format!("{} anos", age);
                }
            }
        }
    }
    "".to_string()
}

#[component]
pub fn PatientListTable(
    patients: Vec<Patient>,
    on_open_profile: EventHandler<String>,
    on_edit_patient: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "patients-table-card",
            table { class: "patients-list-table",
                thead {
                    tr {
                        th { "Nome ⇡" }
                        th { "Prontuário" }
                        th { "Idade" }
                        th { "CPF" }
                        th { "Celular do paciente" }
                        th { style: "text-align: right; padding-right: 20px;", "Ações" }
                    }
                }
                tbody {
                    for (idx, p) in patients.iter().enumerate() {
                        {
                            let pid = p.id.clone();
                            let pid_edit = p.id.clone();
                            let pid_open = p.id.clone();
                            let age_str = calculate_age(&p.birth_date);
                            let cpf_str = p.document_cpf.clone().unwrap_or_default();
                            let phone_str = p.phone.clone();
                            let clean_phone = phone_str.replace(['(', ')', '-', ' ', '+'], "");
                            let prontuario_num = format!("{:02}", idx + 1);

                            rsx! {
                                tr {
                                    key: "{p.id}",
                                    class: "patient-table-row",

                                    td {
                                        div { class: "patient-name-cell",
                                            div { class: "patient-table-avatar", "👤" }
                                            span {
                                                class: "patient-name-link",
                                                title: "Ir para a ficha de {p.full_name}",
                                                onclick: move |_| on_open_profile.call(pid.clone()),
                                                "{p.full_name}"
                                            }
                                        }
                                    }
                                    td { "{prontuario_num}" }
                                    td { "{age_str}" }
                                    td { "{cpf_str}" }
                                    td { "{phone_str}" }
                                    td {
                                        div { class: "patient-actions-cell",
                                            if !clean_phone.is_empty() {
                                                a {
                                                    class: "action-btn-icon btn-whatsapp-green",
                                                    href: "https://wa.me/{clean_phone}",
                                                    target: "_blank",
                                                    title: "Conversar no WhatsApp",
                                                    IconWhatsapp { size: 16, color: "#22c55e".to_string() }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Editar Dados do Paciente",
                                                onclick: move |_| on_edit_patient.call(pid_edit.clone()),
                                                IconEdit { size: 15, color: "#94a3b8".to_string() }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "action-btn-icon",
                                                title: "Abrir Prontuário / Ficha Completa",
                                                onclick: move |_| on_open_profile.call(pid_open.clone()),
                                                IconExternalLink { size: 15, color: "#94a3b8".to_string() }
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
    }
}
