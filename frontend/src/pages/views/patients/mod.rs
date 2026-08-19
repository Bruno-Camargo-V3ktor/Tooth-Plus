//! # Módulo de Visualização de Pacientes e Prontuário Clínico (Frontend)
//!
//! Orquestra a navegação entre a listagem de pacientes com KPIs e a visualização detalhada
//! do prontuário integrado (Visão Geral, Anamnese, Tratamentos, Exames e Documentos Digitais).

pub mod anamnese_tab;
pub mod anamnese_templates_modal;
pub mod documents_tab;
pub mod edit_patient_modal;
pub mod odontogram_tab;
pub mod overview_tab;
pub mod patient_form;
pub mod patient_list;
pub mod photos_tab;

pub use anamnese_tab::*;
pub use anamnese_templates_modal::*;
pub use documents_tab::*;
pub use edit_patient_modal::*;
pub use odontogram_tab::*;
pub use overview_tab::*;
pub use patient_form::*;
pub use patient_list::*;
pub use photos_tab::*;

use crate::api::{fetch_patient_details, fetch_patients, fetch_templates};
use crate::components::icons::{
    IconCalendar, IconChevronLeft, IconEdit, IconEye, IconFile, IconFolder, IconHeartPulse,
    IconLock, IconMail, IconPhone, IconSignature, IconTooth, IconUsers, 
};
use crate::permissions;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::patients::{PatientDetailsResponse, PatientKpis};

/// Formata a data ISO para o padrão brasileiro DD/MM/YYYY.
fn format_br_date_short(date_str: &str) -> String {
    if date_str.len() >= 10 {
        let parts: Vec<&str> = date_str[0..10].split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
        }
    }
    date_str.to_string()
}

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

    let can_read_anamnese = permissions::has_permission(&sess, &clinic, "anamnese:read");
    let can_write_anamnese = permissions::has_permission(&sess, &clinic, "anamnese:write");
    let can_manage_templates = permissions::has_permission(&sess, &clinic, "anamnese:manage_templates");

    let can_read_exams = permissions::has_permission(&sess, &clinic, "exams:read");
    let can_upload_exams = permissions::has_permission(&sess, &clinic, "exams:upload");
    let can_delete_exams = permissions::has_permission(&sess, &clinic, "exams:delete");

    let can_read_treatments = permissions::has_permission(&sess, &clinic, "treatments:read");
    let can_write_treatments = permissions::has_permission(&sess, &clinic, "treatments:write");
    let can_delete_treatments = permissions::has_permission(&sess, &clinic, "treatments:delete");

    let can_read_documents = permissions::has_permission(&sess, &clinic, "documents:read");
    let can_write_documents = permissions::has_permission(&sess, &clinic, "documents:write");
    let can_delete_documents = permissions::has_permission(&sess, &clinic, "documents:delete");

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
        let _ = reload_trigger();
        async move {
            if t.is_empty() || cid.is_empty() {
                return Ok(vec![]);
            }
            fetch_templates(&t, &cid).await
        }
    });

    let (patients_list, kpis, is_patients_loading) = match &*patients_resource.read() {
        Some(Ok(resp)) => (resp.items.clone(), resp.kpis.clone(), false),
        Some(Err(_e)) => (vec![], PatientKpis::default(), false),
        None => (vec![], PatientKpis::default(), true),
    };

    let templates_list = match &*templates_resource.read() {
        Some(Ok(tpls)) => tpls.clone(),
        _ => vec![],
    };

    // Estado do Prontuário Detalhado
    let mut selected_patient_id = use_signal(|| None::<String>);
    let mut patient_details = use_signal(|| None::<PatientDetailsResponse>);
    let mut details_loading = use_signal(|| false);
    let mut active_patient_tab = use_signal(|| "overview".to_string());
    let mut is_create_patient_open = use_signal(|| false);
    let mut is_edit_patient_open = use_signal(|| false);
    let mut is_templates_modal_open = use_signal(|| false);
    let mut is_emit_contract_open = use_signal(|| false);

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

    let on_select_loader = load_patient_details.clone();
    let on_edit_loader = load_patient_details.clone();

    rsx! {
        div { class: "patients-view-container",
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
                    div { class: "loading-card",
                        div { class: "loading-spinner" }
                        p { "Carregando prontuário completo..." }
                    }
                } else if let Some(ref det) = *patient_details.read() {
                    {
                        let reload_fn = {
                            let p_id = sel_id.clone();
                            let mut loader = load_patient_details.clone();
                            move |()| loader(p_id.clone())
                        };

                        let initial = det.patient.full_name.chars().next().unwrap_or('P');
                        let plan_name = det.patient.insurance_plan.as_deref().unwrap_or("Particular");
                        let is_particular = plan_name.eq_ignore_ascii_case("Particular");

                        let doc_str = if let Some(ref cpf) = det.patient.document_cpf {
                            format!("CPF: {}", cpf)
                        } else if let Some(ref rg) = det.patient.document_rg {
                            format!("RG: {}", rg)
                        } else {
                            "Doc: Não informado".to_string()
                        };

                        rsx! {
                            div { class: "patient-details-page",
                                // Top Action Row
                                div { class: "prontuario-top-nav-row",
                                    button {
                                        class: "btn-back-to-list",
                                        onclick: move |_| {
                                            selected_patient_id.set(None);
                                            patient_details.set(None);
                                            reload_trigger.set(reload_trigger() + 1);
                                        },
                                        span { "← Voltar para Lista de Pacientes" }
                                    }
                                }

                                // Patient Profile Card (Single Row)
                                div { class: "patient-profile-card-single-row",
                                    div { class: "patient-profile-avatar",
                                        "{initial}"
                                    }
                                    h2 { class: "patient-hero-name", "{det.patient.full_name}" }
                                    if is_particular {
                                        span { class: "badge-insurance-particular", "Particular" }
                                    } else {
                                        span { class: "badge-insurance-plan", "{plan_name}" }
                                    }
                                    span { class: "patient-profile-meta-item",
                                        IconLock { size: 13, color: "#64748b".to_string() }
                                        span { "{doc_str}" }
                                    }
                                    span { class: "patient-profile-meta-item",
                                        IconPhone { size: 13, color: "#64748b".to_string() }
                                        span { "{det.patient.phone}" }
                                    }
                                    if let Some(ref email) = det.patient.email {
                                        span { class: "patient-profile-meta-item",
                                            IconMail { size: 13, color: "#64748b".to_string() }
                                            span { "{email}" }
                                        }
                                    }
                                    if let Some(ref bdate) = det.patient.birth_date {
                                        span { class: "patient-profile-meta-item",
                                            IconCalendar { size: 13, color: "#64748b".to_string() }
                                            span { "Nasc: {format_br_date_short(bdate)}" }
                                        }
                                    }
                                    if can_write {
                                        button {
                                            r#type: "button",
                                            class: "btn-secondary",
                                            style: "margin-left: auto; font-size: 12px; padding: 4px 10px; display: inline-flex; align-items: center; gap: 6px; border-radius: 6px;",
                                            onclick: move |_| is_edit_patient_open.set(true),
                                            IconEdit { size: 13, color: "#0052cc".to_string() }
                                            span { "Editar Cadastro" }
                                        }
                                    }
                                }

                                // Subtabs Switcher
                                div { class: "patient-subtabs-bar",
                                    button {
                                        class: if active_patient_tab() == "overview" { "patient-subtab-btn active" } else { "patient-subtab-btn" },
                                        onclick: move |_| active_patient_tab.set("overview".to_string()),
                                        IconUsers { size: 15, color: "currentColor".to_string() }
                                        span { " Visão Geral" }
                                    }
                                    if can_read_anamnese {
                                        button {
                                            class: if active_patient_tab() == "anamnese" { "patient-subtab-btn active" } else { "patient-subtab-btn" },
                                            onclick: move |_| active_patient_tab.set("anamnese".to_string()),
                                            IconHeartPulse { size: 15, color: "currentColor".to_string() }
                                            span { " Anamnese & Ficha Médica" }
                                        }
                                    }
                                    if can_read_exams {
                                        button {
                                            class: if active_patient_tab() == "photos" { "patient-subtab-btn active" } else { "patient-subtab-btn" },
                                            onclick: move |_| active_patient_tab.set("photos".to_string()),
                                            IconEye { size: 15, color: "currentColor".to_string() }
                                            span { " Exames & Laudos ({det.exams.len()})" }
                                        }
                                    }
                                    if can_read_treatments {
                                        button {
                                            class: if active_patient_tab() == "odontogram" { "patient-subtab-btn active" } else { "patient-subtab-btn" },
                                            onclick: move |_| active_patient_tab.set("odontogram".to_string()),
                                            IconTooth { size: 15, color: "currentColor".to_string() }
                                            span { " Histórico de Tratamentos ({det.treatments.len()})" }
                                        }
                                    }
                                    if can_read_documents {
                                        button {
                                            class: if active_patient_tab() == "documents" { "patient-subtab-btn active" } else { "patient-subtab-btn" },
                                            onclick: move |_| active_patient_tab.set("documents".to_string()),
                                            IconSignature { size: 15, color: "currentColor".to_string() }
                                            span { " Contratos & Documentos ({det.documents.len()})" }
                                        }
                                    }
                                }

                                // Active Subtab Content
                                match active_patient_tab().as_str() {
                                    "anamnese" => {
                                        if can_read_anamnese {
                                            rsx! {
                                                PatientAnamneseTab {
                                                    patient_id: det.patient.id.clone(),
                                                    clinic_id: clinic_id.clone(),
                                                    token: token.clone(),
                                                    anamnesis: det.anamnesis.clone(),
                                                    can_write: can_write_anamnese,
                                                    reload_patient_details: reload_fn.clone(),
                                                    toast_msg,
                                                    error_toast,
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div { class: "permission-denied-state", style: "margin: 40px auto; max-width: 500px;",
                                                    div { class: "permission-denied-icon", "🔒" }
                                                    h3 { class: "permission-denied-title", "Acesso Restrito à Anamnese" }
                                                    p { class: "permission-denied-desc", "Você não possui a permissão 'anamnese:read' para visualizar a ficha médica." }
                                                }
                                            }
                                        }
                                    },
                                    "odontogram" => {
                                        if can_read_treatments {
                                            rsx! {
                                                PatientOdontogramTab {
                                                    patient_id: det.patient.id.clone(),
                                                    clinic_id: clinic_id.clone(),
                                                    token: token.clone(),
                                                    treatments: det.treatments.clone(),
                                                    can_write: can_write_treatments,
                                                    can_delete: can_delete_treatments,
                                                    reload_patient_details: reload_fn.clone(),
                                                    toast_msg,
                                                    error_toast,
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div { class: "permission-denied-state", style: "margin: 40px auto; max-width: 500px;",
                                                    div { class: "permission-denied-icon", "🔒" }
                                                    h3 { class: "permission-denied-title", "Acesso Restrito aos Tratamentos" }
                                                    p { class: "permission-denied-desc", "Você não possui a permissão 'treatments:read' para visualizar os procedimentos." }
                                                }
                                            }
                                        }
                                    },
                                    "photos" => {
                                        if can_read_exams {
                                            rsx! {
                                                PatientPhotosTab {
                                                    patient_id: det.patient.id.clone(),
                                                    clinic_id: clinic_id.clone(),
                                                    token: token.clone(),
                                                    exams: det.exams.clone(),
                                                    can_write: can_upload_exams,
                                                    can_delete: can_delete_exams,
                                                    reload_patient_details: reload_fn.clone(),
                                                    toast_msg,
                                                    error_toast,
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div { class: "permission-denied-state", style: "margin: 40px auto; max-width: 500px;",
                                                    div { class: "permission-denied-icon", "🔒" }
                                                    h3 { class: "permission-denied-title", "Acesso Restrito aos Exames" }
                                                    p { class: "permission-denied-desc", "Você não possui a permissão 'exams:read' para visualizar os exames e laudos." }
                                                }
                                            }
                                        }
                                    },
                                    "documents" => {
                                        if can_read_documents {
                                            rsx! {
                                                PatientDocumentsTab {
                                                    patient_id: det.patient.id.clone(),
                                                    patient_name: det.patient.full_name.clone(),
                                                    patient_cpf: det.patient.document_cpf.clone(),
                                                    patient_phone: Some(det.patient.phone.clone()),
                                                    patient_insurance: det.patient.insurance_plan.clone(),
                                                    clinic_id: clinic_id.clone(),
                                                    token: token.clone(),
                                                    documents: det.documents.clone(),
                                                    templates: templates_list.clone(),
                                                    can_write: can_write_documents,
                                                    reload_patient_details: reload_fn.clone(),
                                                    toast_msg,
                                                    error_toast,
                                                    is_emit_modal_open: is_emit_contract_open,
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div { class: "permission-denied-state", style: "margin: 40px auto; max-width: 500px;",
                                                    div { class: "permission-denied-icon", "🔒" }
                                                    h3 { class: "permission-denied-title", "Acesso Restrito aos Documentos" }
                                                    p { class: "permission-denied-desc", "Você não possui a permissão 'documents:read' para visualizar os contratos e termos." }
                                                }
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        PatientOverviewTab {
                                            patient: det.patient.clone(),
                                            clinic_id: clinic_id.clone(),
                                            token: token.clone(),
                                            can_write,
                                            reload_patient_details: reload_fn.clone(),
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
                    kpis,
                    patients: patients_list,
                    is_loading: is_patients_loading,
                    search_query,
                    reload_trigger,
                    can_write,
                    can_delete,
                    can_manage_templates,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    on_select_patient: move |p_id: String| {
                        let mut loader = on_select_loader.clone();
                        selected_patient_id.set(Some(p_id.clone()));
                        loader(p_id);
                    },
                    on_open_create_modal: move |()| {
                        is_create_patient_open.set(true);
                    },
                    on_open_templates_modal: move |()| {
                        is_templates_modal_open.set(true);
                    },
                    toast_msg,
                    error_toast,
                }
            }


            if is_create_patient_open() {
                PatientFormModal {
                    is_open: is_create_patient_open,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    reload_trigger,
                    toast_msg,
                    error_toast,
                }
            }

            if is_templates_modal_open() {
                AnamneseTemplatesModal {
                    is_open: is_templates_modal_open,
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    toast_msg,
                    error_toast,
                }
            }

            if is_edit_patient_open() {
                if let Some(ref det) = patient_details() {
                    {
                        let reload_loader = on_edit_loader.clone();
                        let sel_p_id = selected_patient_id().unwrap_or_default();
                        rsx! {
                            EditPatientModal {
                                is_open: is_edit_patient_open,
                                patient: det.patient.clone(),
                                clinic_id: clinic_id.clone(),
                                token: token.clone(),
                                reload_patient_details: move |()| {
                                    let mut l = reload_loader.clone();
                                    if !sel_p_id.is_empty() {
                                        l(sel_p_id.clone());
                                    }
                                },
                                toast_msg,
                                error_toast,
                            }
                        }
                    }
                }
            }
        }
    }
}

