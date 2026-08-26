pub mod components;

use crate::api::patients::PatientsApi;
use crate::components::patient_form_modal::PatientFormModal;
use shared::patients::Patient;
use dioxus::prelude::*;

pub use components::{PatientDetailsModal, PatientKpis, PatientTable, PatientToolbar};

const STYLE: Asset = asset!("/src/pages/patients/style.css");

#[component]
pub fn PatientsView() -> Element {
    let mut patients_list = use_signal(Vec::<Patient>::new);
    let mut search_query = use_signal(String::new);
    let mut show_new_modal = use_signal(|| false);
    let mut selected_patient_id = use_signal(|| None::<String>);
    let mut modal_tab = use_signal(|| "info".to_string());

    let load_patients = {
        let query_sig = search_query.clone();
        let mut list_sig = patients_list;

        move || {
            let q = query_sig.read().trim().to_string();
            let query_opt = if q.is_empty() { None } else { Some(q) };

            spawn(async move {
                if let Ok(resp) = PatientsApi::list_patients(query_opt.as_deref()).await {
                    list_sig.set(resp.items);
                }
            });
        }
    };

    use_effect({
        let mut loader = load_patients.clone();
        move || loader()
    });

    let selected_patient = selected_patient_id.read().as_ref().and_then(|pid| {
        patients_list.read().iter().find(|p| p.id == *pid).cloned()
    });

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "patients-page",
            PatientKpis { patients: patients_list() }

            PatientToolbar {
                search_query,
                on_search: move |_| {
                    let mut loader = load_patients.clone();
                    loader();
                },
                on_open_modal: move |_| show_new_modal.set(true),
            }

            PatientTable {
                patients: patients_list(),
                on_select: move |pid| selected_patient_id.set(Some(pid)),
            }

            if show_new_modal() {
                PatientFormModal {
                    on_close: move |_| show_new_modal.set(false),
                    on_save: move |_| {
                        show_new_modal.set(false);
                        let mut loader = load_patients.clone();
                        loader();
                    },
                }
            }

            if let Some(patient) = selected_patient {
                PatientDetailsModal {
                    patient,
                    active_tab: modal_tab,
                    on_close: move |_| selected_patient_id.set(None),
                }
            }
        }
    }
}
