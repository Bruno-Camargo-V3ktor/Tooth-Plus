use crate::icons::{
    IconAlertTriangle, IconArrowLeft, IconClock, IconDollar, IconEdit, IconWhatsapp,
};
use shared::patients::Patient;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum PatientDetailTab {
    About,
    Budgets,
    Treatments,
    Anamnesis,
    Images,
    Documents,
    Debits,
}

#[component]
pub fn PatientProfileHeader(
    patient: Patient,
    active_tab: Signal<PatientDetailTab>,
    on_back: EventHandler<()>,
    on_edit: EventHandler<()>,
) -> Element {
    let clean_phone = patient.phone.replace(['(', ')', '-', ' ', '+'], "");

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 14px;",
            div { class: "patient-profile-top-bar",
                div { class: "profile-top-left",
                    button {
                        r#type: "button",
                        class: "btn-back-patients",
                        title: "Voltar para lista de pacientes",
                        onclick: move |_| on_back.call(()),
                        IconArrowLeft { size: 18, color: "currentColor".to_string() }
                    }

                    div { class: "patient-profile-avatar-big", "👤" }

                    div { class: "patient-profile-titles",
                        div { class: "patient-profile-name-row",
                            h2 { class: "patient-profile-name", "{patient.full_name}" }
                            button {
                                r#type: "button",
                                class: "btn-edit-patient-pill",
                                onclick: move |_| on_edit.call(()),
                                IconEdit { size: 12, color: "currentColor".to_string() }
                                span { "EDITAR" }
                            }
                            div { class: "profile-quick-actions",
                                button {
                                    class: "quick-action-pill",
                                    title: "Ver Débitos Financeiros",
                                    onclick: move |_| active_tab.set(PatientDetailTab::Debits),
                                    IconDollar { size: 14, color: "currentColor".to_string() }
                                }
                                button {
                                    class: "quick-action-pill",
                                    title: "Ver Tratamentos & Histórico",
                                    onclick: move |_| active_tab.set(PatientDetailTab::Treatments),
                                    IconClock { size: 14, color: "currentColor".to_string() }
                                }
                                button {
                                    class: "quick-action-pill",
                                    title: "Ver Anamnese & Alertas Clínicos",
                                    onclick: move |_| active_tab.set(PatientDetailTab::Anamnesis),
                                    IconAlertTriangle { size: 14, color: "currentColor".to_string() }
                                }
                                if !clean_phone.is_empty() {
                                    a {
                                        class: "quick-action-pill quick-whatsapp",
                                        title: "Chamar no WhatsApp",
                                        href: "https://wa.me/{clean_phone}",
                                        target: "_blank",
                                        IconWhatsapp { size: 15, color: "#22c55e".to_string() }
                                    }
                                }
                            }
                        }
                        span { class: "patient-profile-sub",
                            "{patient.phone} - Nº paciente: 25"
                        }
                    }
                }
            }

            // Barra de abas oficial do prontuário
            div { class: "patient-tabs-bar",
                button {
                    class: if *active_tab.read() == PatientDetailTab::About { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::About),
                    "SOBRE"
                }
                button {
                    class: if *active_tab.read() == PatientDetailTab::Budgets { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::Budgets),
                    "ORÇAMENTOS"
                }
                button {
                    class: if *active_tab.read() == PatientDetailTab::Treatments { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::Treatments),
                    "TRATAMENTOS"
                }
                button {
                    class: if *active_tab.read() == PatientDetailTab::Anamnesis { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::Anamnesis),
                    "ANAMNESE"
                }
                button {
                    class: if *active_tab.read() == PatientDetailTab::Images { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::Images),
                    "IMAGENS"
                }
                button {
                    class: if *active_tab.read() == PatientDetailTab::Documents { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::Documents),
                    "DOCUMENTOS"
                }
                button {
                    class: if *active_tab.read() == PatientDetailTab::Debits { "patient-tab-btn tab-active" } else { "patient-tab-btn" },
                    onclick: move |_| active_tab.set(PatientDetailTab::Debits),
                    "DÉBITOS"
                }
            }
        }
    }
}
