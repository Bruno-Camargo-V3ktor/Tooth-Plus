//! # Módulo de Visualização de Pacientes e Prontuário Clínico (Frontend)
//!
//! Orquestra a navegação entre a listagem de pacientes com KPIs e a visualização detalhada
//! do prontuário integrado (Visão Geral, Anamnese, Tratamentos, Exames e Documentos Digitais).

pub mod anamnese_tab;
pub mod documents_tab;
pub mod odontogram_tab;
pub mod overview_tab;
pub mod patient_form;
pub mod patient_list;
pub mod photos_tab;

pub use anamnese_tab::*;
pub use documents_tab::*;
pub use odontogram_tab::*;
pub use overview_tab::*;
pub use patient_form::*;
pub use patient_list::*;
pub use photos_tab::*;

use crate::api::{fetch_patient_details, fetch_patients, fetch_templates};
use crate::components::icons::{
    IconChevronLeft, IconFile, IconFolder, IconHeartPulse, IconSignature, IconTooth,
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::patients::{PatientDetailsResponse, PatientKpis};

/// Componente principal da tela de Gestão de Pacientes e Prontuário Clínico.
#[component]
pub fn PatientsView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = permissions::has_permission(&sess, &clinic, "patients:read");
    let can_write = permissions::has_permission(&sess, &clinic, "patients:write");
    let can_delete = permissions::has_permission(&sess, &clinic, "patients:delete");

    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();
    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar os prontuários desta unidade." }
            }
        };
    }

    let mut search_query = use_signal(String::new);
    let mut reload_trigger = use_signal(|| 0usize);
    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();
    let patients_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        let search = search_query();
        let _ = reload_trigger();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(shared::patients::PatientListResponse {
                    items: vec![],
                    kpis: PatientKpis::default(),
                    total: 0,
                });
            }
            fetch_patients(
                &t,
                &cid,
                if search.is_empty() {
                    None
                } else {
                    Some(&search)
                },
            )
            .await
        }
    });

    let tok_tpl = token.clone();
    let cid_tpl = clinic_id.clone();
    let templates_resource = use_resource(move || {
        let t = tok_tpl.clone();
        let cid = cid_tpl.clone();
        async move {
            if t.is_empty() || cid.is_empty() || !can_read {
                return Ok(vec![]);
            }
            fetch_templates(&t, &cid).await
        }
    });

    let (patients_list, kpis, is_loading) = match &*patients_resource.read() {
        Some(Ok(resp)) => (resp.items.clone(), resp.kpis.clone(), false),
        Some(Err(_e)) => (vec![], PatientKpis::default(), false),
        None => (vec![], PatientKpis::default(), true),
    };

    let templates_list = match &*templates_resource.read() {
        Some(Ok(tpls)) => tpls.clone(),
        _ => vec![],
    };

    // Dedicated Full-Page Patient View State
    let mut selected_patient_id = use_signal(|| None::<String>);
    let mut patient_details = use_signal(|| None::<PatientDetailsResponse>);
    let mut details_loading = use_signal(|| false);
    let mut active_patient_tab = use_signal(|| "overview".to_string());
    let mut is_create_patient_open = use_signal(|| false);

    let load_patient_details = {
        let tok = token.clone();
        let cid = clinic_id.clone();
        move |p_id: String| {
            let t = tok.clone();
            let c = cid.clone();
            let mut det_sig = patient_details;
            let mut load_sig = details_loading;
            let mut err_sig = error_toast;

            load_sig.set(true);
            spawn(async move {
                match fetch_patient_details(&t, &p_id, &c).await {
                    Ok(resp) => {
                        det_sig.set(Some(resp));
                    }
                    Err(e) => {
                        err_sig.set(Some(format!("Erro ao carregar prontuário: {}", e)));
                    }
                }
                load_sig.set(false);
            });
        }
    };

    rsx! {
        div { class: "patients-main-wrapper",
            if let Some(ref toast) = *toast_msg.read() {
                div { class: "toast toast-success",
                    span { "{toast}" }
                    button { class: "toast-close", onclick: move |_| toast_msg.set(None), "✕" }
                }
            }
            if let Some(ref err) = *error_toast.read() {
                div { class: "toast toast-error",
                    span { "{err}" }
                    button { class: "toast-close", onclick: move |_| error_toast.set(None), "✕" }
                }
            }

            if let Some(ref sel_id) = *selected_patient_id.read() {
                if details_loading() {
                    div { class: "loading-state p-8", "Carregando prontuário completo..." }
                } else if let Some(ref det) = *patient_details.read() {
                    {
                        let reload_fn = {
                            let p_id = sel_id.clone();
                            let mut loader = load_patient_details.clone();
                            move |()| loader(p_id.clone())
                        };

                        rsx! {
                            div { class: "patient-details-view",
                                div { class: "details-topbar",
                                    button {
                                        class: "btn-back",
                                        onclick: move |_| {
                                            selected_patient_id.set(None);
                                            patient_details.set(None);
                                            reload_trigger.set(reload_trigger() + 1);
                                        },
                                        IconChevronLeft { size: 16, color: "currentColor".to_string() }
                                        span { "Voltar para Lista de Pacientes" }
                                    }
                                }

                                div { class: "patient-header-banner",
                                    div { class: "patient-banner-avatar",
                                        "{det.patient.full_name.chars().next().unwrap_or('P')}"
                                    }
                                    div { class: "patient-banner-info",
                                        div { class: "patient-banner-title-row",
                                            h2 { class: "patient-banner-name", "{det.patient.full_name}" }
                                            span { class: "badge-insurance",
                                                "{det.patient.insurance_plan.as_deref().unwrap_or(\"Particular\")}"
                                            }
                                        }
                                        div { class: "patient-banner-meta",
                                            span { "CPF: {det.patient.document_cpf}" }
                                            span { " • " }
                                            span { "Tel: {det.patient.phone}" }
                                            if let Some(ref bdate) = det.patient.birth_date {
                                                span { " • " }
                                                span { "Nascimento: {bdate}" }
                                            }
                                        }
                                    }
                                }

                                div { class: "patient-nav-tabs",
                                    button {
                                        class: if active_patient_tab() == "overview" { "nav-tab-btn active" } else { "nav-tab-btn" },
                                        onclick: move |_| active_patient_tab.set("overview".to_string()),
                                        "Visão Geral"
                                    }
                                    button {
                                        class: if active_patient_tab() == "anamnese" { "nav-tab-btn active" } else { "nav-tab-btn" },
                                        onclick: move |_| active_patient_tab.set("anamnese".to_string()),
                                        IconHeartPulse { size: 14, color: "currentColor".to_string() }
                                        span { class: "ml-1", "Anamnese" }
                                    }
                                    button {
                                        class: if active_patient_tab() == "odontogram" { "nav-tab-btn active" } else { "nav-tab-btn" },
                                        onclick: move |_| active_patient_tab.set("odontogram".to_string()),
                                        IconTooth { size: 14, color: "currentColor".to_string() }
                                        span { class: "ml-1", "Tratamentos ({det.treatments.len()})" }
                                    }
                                    button {
                                        class: if active_patient_tab() == "photos" { "nav-tab-btn active" } else { "nav-tab-btn" },
                                        onclick: move |_| active_patient_tab.set("photos".to_string()),
                                        IconFolder { size: 14, color: "currentColor".to_string() }
                                        span { class: "ml-1", "Exames & Fotos ({det.exams.len()})" }
                                    }
                                    button {
                                        class: if active_patient_tab() == "documents" { "nav-tab-btn active" } else { "nav-tab-btn" },
                                        onclick: move |_| active_patient_tab.set("documents".to_string()),
                                        IconSignature { size: 14, color: "currentColor".to_string() }
                                        span { class: "ml-1", "Contratos & Termos ({det.documents.len()})" }
                                    }
                                }

                                match active_patient_tab().as_str() {
                                    "anamnese" => rsx! {
                                        PatientAnamneseTab {
                                            patient_id: det.patient.id.clone(),
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            anamnesis: det.anamnesis.clone(),
                                            can_write,
                                            reload_patient_details: reload_fn,
                                            toast_msg,
                                            error_toast,
                                        }
                                    },
                                    "odontogram" => rsx! {
                                        PatientOdontogramTab {
                                            patient_id: det.patient.id.clone(),
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            treatments: det.treatments.clone(),
                                            can_write,
                                            reload_patient_details: reload_fn,
                                            toast_msg,
                                            error_toast,
                                        }
                                    },
                                    "photos" => rsx! {
                                        PatientPhotosTab {
                                            patient_id: det.patient.id.clone(),
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            exams: det.exams.clone(),
                                            can_write,
                                            reload_patient_details: reload_fn,
                                            toast_msg,
                                            error_toast,
                                        }
                                    },
                                    "documents" => rsx! {
                                        PatientDocumentsTab {
                                            patient_id: det.patient.id.clone(),
                                            patient_name: det.patient.full_name.clone(),
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            documents: det.documents.clone(),
                                            templates: templates_list.clone(),
                                            can_write,
                                            reload_patient_details: reload_fn,
                                            toast_msg,
                                            error_toast,
                                        }
                                    },
                                    _ => rsx! {
                                        PatientOverviewTab {
                                            patient: det.patient.clone(),
                                            token: token.clone(),
                                            clinic_id: clinic_id.clone(),
                                            can_write,
                                            reload_patient_details: reload_fn,
                                            toast_msg,
                                            error_toast,
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            } else {
                PatientListSection {
                    patients: patients_list,
                    kpis,
                    is_loading,
                    search_query,
                    can_write,
                    can_delete,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    on_open_create_modal: move |_| is_create_patient_open.set(true),
                    on_select_patient: move |p_id: String| {
                        selected_patient_id.set(Some(p_id.clone()));
                        let mut loader = load_patient_details.clone();
                        loader(p_id);
                    },
                    reload_trigger,
                    toast_msg,
                    error_toast,
                }
            }

            PatientFormModal {
                token: token.clone(),
                clinic_id: clinic_id.clone(),
                is_open: is_create_patient_open,
                reload_trigger,
                toast_msg,
                error_toast,
            }
        }
    }
}
