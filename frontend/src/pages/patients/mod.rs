pub mod components;

use crate::api::appointments::AppointmentsApi;
use crate::api::finance::FinanceApi;
use crate::api::mock_db::DB;
use crate::api::patients::PatientsApi;
use crate::api::ActiveClinicState;
use crate::components::patient_form_modal::PatientFormModal;
use shared::appointments::AppointmentResponse;
use shared::finance::{FinanceQuery, Transaction};
use shared::patients::{Patient, PatientTreatment};
use shared::treatments::PatientTreatmentPlan;
use dioxus::prelude::*;

pub use components::*;

const STYLE: Asset = asset!("/src/pages/patients/style.css");

#[component]
pub fn PatientsView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut patients_list = use_signal(Vec::<Patient>::new);
    let mut appointments = use_signal(Vec::<AppointmentResponse>::new);
    let mut transactions = use_signal(Vec::<Transaction>::new);
    let mut patient_treatments = use_signal(Vec::<PatientTreatment>::new);
    let mut treatment_plans = use_signal(Vec::<PatientTreatmentPlan>::new);

    let mut search_query = use_signal(String::new);
    let mut show_modal = use_signal(|| false);
    let mut selected_patient_id = use_signal(|| None::<String>);
    let mut active_tab = use_signal(|| PatientDetailTab::About);
    let mut reload_trigger = use_signal(|| 0);

    let cid_effect = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_effect.clone();
        let q = search_query.read().trim().to_string();
        let query_opt = if q.is_empty() { None } else { Some(q) };

        spawn(async move {
            if let Ok(resp) = PatientsApi::list_patients(query_opt.as_deref()).await {
                patients_list.set(resp.items);
            }
            if let Ok(apps) = AppointmentsApi::list_appointments(&cid, None).await {
                appointments.set(apps);
            }
            let f_query = FinanceQuery {
                clinic_id: cid.clone(),
                month: None,
                year: None,
                start_date: None,
                end_date: None,
            };
            if let Ok(txs) = FinanceApi::list_transactions(f_query).await {
                transactions.set(txs.transactions);
            }
            if let Ok(db) = DB.lock() {
                patient_treatments.set(db.patient_treatments.clone());
                treatment_plans.set(db.treatment_plans.clone());
            }
        });
    });

    let selected_patient = selected_patient_id.read().as_ref().and_then(|pid| {
        patients_list.read().iter().find(|p| p.id == *pid).cloned()
    });

    let filtered_patients: Vec<Patient> = patients_list.read().iter().filter(|p| {
        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() { return true; }
        p.full_name.to_lowercase().contains(&q)
            || p.phone.contains(&q)
            || p.document_cpf.as_deref().unwrap_or("").contains(&q)
    }).cloned().collect();

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "patients-page",
            if let Some(patient) = selected_patient {
                // Modo Ficha / Prontuário Completo do Paciente (Screenshots 1, 2, 4)
                PatientProfileHeader {
                    patient: patient.clone(),
                    active_tab,
                    on_back: move |_| selected_patient_id.set(None),
                    on_edit: move |_| show_modal.set(true),
                }

                match *active_tab.read() {
                    PatientDetailTab::About => rsx! {
                        TabAbout { patient, appointments: appointments() }
                    },
                    PatientDetailTab::Treatments => rsx! {
                        TabTreatments {
                            patient: patient.clone(),
                            treatments: patient_treatments(),
                            on_add_treatment: move |(_plan, t_name, teeth, cost): (String, String, String, i64)| {
                                if let Ok(mut db) = DB.lock() {
                                    let new_id = format!("pt:{}", db.patient_treatments.len() + 1);
                                    db.patient_treatments.push(PatientTreatment {
                                        id: new_id,
                                        patient_id: patient.id.clone(),
                                        clinic_id: patient.clinic_id.clone(),
                                        dentist_user_id: Some("usr:dr_lucas".to_string()),
                                        dentist_user_name: Some("Dr. Lucas Mendes".to_string()),
                                        appointment_id: None,
                                        appointment_date: None,
                                        document_id: None,
                                        exam_id: None,
                                        treatment_plan_id: None,
                                        treatment_plan_item_id: None,
                                        transaction_id: None,
                                        financial_status: Some("unpaid".to_string()),
                                        procedure_category: Some("Clínico".to_string()),
                                        procedure_name: t_name,
                                        tooth_number: if teeth.is_empty() { None } else { Some(teeth) },
                                        surfaces: None,
                                        materials_used: None,
                                        status: "Em Andamento".to_string(),
                                        cost_cents: cost,
                                        post_care_instructions: None,
                                        clinical_notes: None,
                                        performed_at: Some("2026-08-26T10:00:00Z".to_string()),
                                        created_at: "2026-08-26T10:00:00Z".to_string(),
                                    });
                                }
                                reload_trigger.set(reload_trigger() + 1);
                            },
                        }
                    },
                    PatientDetailTab::Debits => rsx! {
                        TabDebits {
                            patient: patient.clone(),
                            transactions: transactions(),
                            on_new_debit: move |_| {
                                active_tab.set(PatientDetailTab::Treatments);
                            },
                        }
                    },
                    PatientDetailTab::Budgets => rsx! {
                        TabBudgets {
                            patient,
                            plans: treatment_plans(),
                            on_new_budget: move |_| {
                                active_tab.set(PatientDetailTab::Treatments);
                            },
                        }
                    },
                    PatientDetailTab::Anamnesis => rsx! {
                        TabAnamnesis { patient }
                    },
                    PatientDetailTab::Images => rsx! {
                        TabImages { patient }
                    },
                    PatientDetailTab::Documents => rsx! {
                        TabDocuments { patient }
                    },
                }
            } else {
                // Modo Listagem Oficial dos Pacientes (Screenshot 3)
                PatientListToolbar {
                    search_query,
                    on_new_patient: move |_| show_modal.set(true),
                    on_export: move |_| {
                        let _ = web_sys::window().map(|w| w.print());
                    },
                }

                PatientListTable {
                    patients: filtered_patients,
                    on_open_profile: move |pid| {
                        selected_patient_id.set(Some(pid));
                        active_tab.set(PatientDetailTab::About);
                    },
                    on_edit_patient: move |pid| {
                        selected_patient_id.set(Some(pid));
                        show_modal.set(true);
                    },
                }
            }

            if show_modal() {
                PatientFormModal {
                    on_close: move |_| show_modal.set(false),
                    on_save: move |_| {
                        show_modal.set(false);
                        reload_trigger.set(reload_trigger() + 1);
                    },
                }
            }
        }
    }
}
