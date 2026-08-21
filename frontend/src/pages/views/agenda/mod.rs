//! # Módulo de Visualização da Agenda Clínica (Frontend)
//!
//! Controla o agendamento de consultas, cirurgias, retornos e procedimentos,
//! visualização em timeline diária por profissional e alocação de insumos/estoque.

pub mod appointment_modal;
pub mod calendar_views;
pub mod status_modal;

pub use appointment_modal::*;
pub use calendar_views::*;
pub use status_modal::*;

use crate::api::{delete_appointment, fetch_agenda_resources, fetch_appointments};
use crate::components::icons::{IconCalendar, IconCheck, IconClock, IconUsers};
use crate::permissions::has_permission;
use crate::{ActiveClinicState, SessionState};
use dioxus::prelude::*;
use shared::appointments::{
    AgendaResourcesResponse, AppointmentResponse, AppointmentStatus, AppointmentType,
};

/// Componente principal da tela de Agenda.
#[component]
pub fn AgendaView() -> Element {
    let session = consume_context::<Signal<SessionState>>();
    let active_clinic = consume_context::<Signal<ActiveClinicState>>();

    let sess = session();
    let clinic = active_clinic();

    let can_read = has_permission(&sess, &clinic, "appointments:read");
    let can_write = has_permission(&sess, &clinic, "appointments:write");
    let can_delete = has_permission(&sess, &clinic, "appointments:delete");
    let can_finance = has_permission(&sess, &clinic, "appointments:finance");


    if !can_read {
        return rsx! {
            div { class: "permission-denied-state",
                div { class: "permission-denied-icon", "🔒" }
                h2 { class: "permission-denied-title", "Acesso Restrito" }
                p { class: "permission-denied-desc", "Você não possui permissão para acessar a agenda desta clínica." }
            }
        };
    }

    let now_date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut selected_date = use_signal(|| now_date_str);
    let mut selected_time = use_signal(|| "09:00".to_string());
    let mut search_query = use_signal(String::new);
    let mut filter_member = use_signal(|| "all".to_string());
    let mut filter_status = use_signal(|| "all".to_string());
    let mut filter_type = use_signal(|| "all".to_string());

    let mut is_form_modal_open = use_signal(|| false);
    let mut is_status_modal_open = use_signal(|| false);
    let mut is_delete_modal_open = use_signal(|| false);
    let mut selected_appointment = use_signal(|| None::<AppointmentResponse>);
    let mut delete_target = use_signal(|| None::<AppointmentResponse>);
    let mut is_deleting = use_signal(|| false);
    let mut toast_msg = use_signal(|| None::<String>);
    let mut error_toast = use_signal(|| None::<String>);

    let clinic_id = clinic
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();
    let token = sess.as_ref().map(|s| s.token.clone()).unwrap_or_default();

    let tok_res = token.clone();
    let cid_res = clinic_id.clone();

    let mut appointments_resource = use_resource(move || {
        let t = tok_res.clone();
        let cid = cid_res.clone();
        let d = selected_date();
        async move {
            if t.is_empty() || cid.is_empty() {
                vec![]
            } else {
                fetch_appointments(&t, &cid, Some(&d), None, None)
                    .await
                    .unwrap_or_default()
            }
        }
    });

    let tok_res_meta = token.clone();
    let cid_res_meta = clinic_id.clone();
    let resources_resource = use_resource(move || {
        let t = tok_res_meta.clone();
        let cid = cid_res_meta.clone();
        async move {
            if t.is_empty() || cid.is_empty() {
                AgendaResourcesResponse {
                    team_members: vec![],
                    patients: vec![],
                    inventory_items: vec![],
                    equipment_items: vec![],
                }
            } else {
                fetch_agenda_resources(&t, &cid).await.unwrap_or(AgendaResourcesResponse {
                    team_members: vec![],
                    patients: vec![],
                    inventory_items: vec![],
                    equipment_items: vec![],
                })
            }
        }
    });

    let all_appointments = appointments_resource().unwrap_or_default();
    let agenda_resources = resources_resource().unwrap_or(AgendaResourcesResponse {
        team_members: vec![],
        patients: vec![],
        inventory_items: vec![],
        equipment_items: vec![],
    });

    let s_query = search_query().to_lowercase();
    let f_member = filter_member();
    let f_status = filter_status();
    let f_type = filter_type();

    let filtered_appointments: Vec<AppointmentResponse> = all_appointments
        .into_iter()
        .filter(|app| {
            if !s_query.is_empty() {
                let matches_title = app.title.to_lowercase().contains(&s_query);
                let matches_patient = app
                    .patient_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&s_query);
                if !matches_title && !matches_patient {
                    return false;
                }
            }

            if f_member != "all" {
                let has_member = app.assigned_users.iter().any(|u| {
                    u.user_id == f_member || u.user_id.ends_with(&format!(":{}", f_member))
                });
                if !has_member {
                    return false;
                }
            }

            if f_status != "all" {
                let status_matches = match f_status.as_str() {
                    "pending" => app.status == AppointmentStatus::Pending,
                    "confirmed" => app.status == AppointmentStatus::Confirmed,
                    "in_progress" => app.status == AppointmentStatus::InProgress,
                    "completed" => app.status == AppointmentStatus::Completed,
                    "canceled_by_doctor" => app.status == AppointmentStatus::CanceledByDoctor,
                    "canceled_by_patient" => app.status == AppointmentStatus::CanceledByPatient,
                    "canceled" => app.status.is_canceled(),
                    "no_show" => app.status == AppointmentStatus::NoShow,
                    _ => true,
                };
                if !status_matches {
                    return false;
                }
            }

            if f_type != "all" {
                let type_matches = match f_type.as_str() {
                    "consultation" => app.appointment_type == AppointmentType::Consultation,
                    "treatment" => app.appointment_type == AppointmentType::Treatment,
                    "surgery" => app.appointment_type == AppointmentType::Surgery,
                    "return" => app.appointment_type == AppointmentType::Return,
                    "meeting" => app.appointment_type == AppointmentType::Meeting,
                    "other" => app.appointment_type == AppointmentType::Other,
                    _ => true,
                };
                if !type_matches {
                    return false;
                }
            }

            true
        })
        .collect();

    let total_count = filtered_appointments.len();
    let completed_count = filtered_appointments
        .iter()
        .filter(|a| a.status == AppointmentStatus::Completed)
        .count();
    let in_progress_count = filtered_appointments
        .iter()
        .filter(|a| a.status == AppointmentStatus::InProgress)
        .count();
    let pending_count = filtered_appointments
        .iter()
        .filter(|a| {
            a.status == AppointmentStatus::Pending || a.status == AppointmentStatus::Confirmed
        })
        .count();

    let tok_del = token.clone();
    let cid_del = clinic_id.clone();
    let mut handle_confirm_delete = move |_| {
        let Some(ref app) = delete_target() else {
            return;
        };
        let a_id = app.id.clone();
        let t = tok_del.clone();
        let c = cid_del.clone();
        let mut del_sig = delete_target;
        let mut del_modal = is_delete_modal_open;
        let mut toast = toast_msg;
        let mut err_sig = error_toast;
        let mut is_del = is_deleting;

        is_del.set(true);
        spawn(async move {
            match delete_appointment(&t, &a_id, &c).await {
                Ok(_) => {
                    del_sig.set(None);
                    del_modal.set(false);
                    appointments_resource.restart();
                    toast.set(Some("Agendamento excluído!".into()));
                }
                Err(e) => {
                    err_sig.set(Some(format!("Erro ao excluir agendamento: {}", e)));
                }
            }
            is_del.set(false);
        });
    };

    rsx! {
        div { class: "agenda-page-container",
            if let Some(ref msg) = *toast_msg.read() {
                div { class: "toast toast-success",
                    span { "{msg}" }
                    button { class: "toast-close", onclick: move |_| toast_msg.set(None), "✕" }
                }
            }
            if let Some(ref err) = *error_toast.read() {
                div { class: "toast toast-error",
                    span { "{err}" }
                    button { class: "toast-close", onclick: move |_| error_toast.set(None), "✕" }
                }
            }

            div { class: "agenda-kpi-row",
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-total",
                        IconCalendar { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Total no Dia" }
                    }
                    div { class: "agenda-kpi-val", "{total_count}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-pending",
                        IconClock { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Pendentes" }
                    }
                    div { class: "agenda-kpi-val kpi-pending", "{pending_count}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-progress",
                        IconUsers { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Em Atendimento" }
                    }
                    div { class: "agenda-kpi-val kpi-progress", "{in_progress_count}" }
                }
                div { class: "agenda-kpi-card",
                    div { class: "agenda-kpi-icon-wrapper kpi-icon-completed",
                        IconCheck { size: 16, color: "currentColor".to_string() }
                    }
                    div { class: "agenda-kpi-text-col",
                        span { class: "agenda-kpi-lbl", "Concluídos" }
                    }
                    div { class: "agenda-kpi-val kpi-completed", "{completed_count}" }
                }
            }

            AgendaToolbar {
                selected_date,
                search_query,
                filter_member,
                filter_status,
                filter_type,
                resources: agenda_resources.clone(),
                can_write,
                on_new_appointment: move |_| {
                    selected_appointment.set(None);
                    selected_time.set("09:00".to_string());
                    is_form_modal_open.set(true);
                },
            }

            DayTimelineView {
                appointments: filtered_appointments,
                can_write,
                can_delete,
                can_finance,

                on_slot_click: move |h| {
                    if can_write {
                        selected_appointment.set(None);
                        selected_time.set(format!("{:02}:00", h));
                        is_form_modal_open.set(true);
                    }
                },
                on_edit: move |app: AppointmentResponse| {
                    selected_appointment.set(Some(app));
                    is_form_modal_open.set(true);
                },
                on_status_change: move |app: AppointmentResponse| {
                    selected_appointment.set(Some(app));
                    is_status_modal_open.set(true);
                },
                on_delete: move |app: AppointmentResponse| {
                    delete_target.set(Some(app));
                    is_delete_modal_open.set(true);
                },
            }

            if is_form_modal_open() {
                AppointmentModal {
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    editing_appointment: selected_appointment(),
                    default_date: selected_date(),
                    default_time: selected_time(),
                    resources: agenda_resources.clone(),
                    is_open: is_form_modal_open,
                    can_finance,
                    on_success: move |_| {
                        is_form_modal_open.set(false);
                        appointments_resource.restart();
                    },
                    toast_msg,
                    error_toast,
                }
            }


            if is_status_modal_open() {
                AppointmentStatusModal {
                    token: token.clone(),
                    clinic_id: clinic_id.clone(),
                    appointment: selected_appointment(),
                    is_open: is_status_modal_open,
                    on_success: move |_| {
                        is_status_modal_open.set(false);
                        appointments_resource.restart();
                    },
                    toast_msg,
                    error_toast,
                }
            }

            if is_delete_modal_open() {
                if let Some(ref app) = *delete_target.read() {
                    div { class: "modal-overlay",
                        div { class: "action-modal delete-modal-card",
                            div { class: "modal-header",
                                h2 { class: "modal-title text-danger", "Excluir Agendamento" }
                                button { class: "modal-close", onclick: move |_| is_delete_modal_open.set(false), "×" }
                            }
                            div { class: "modal-body",
                                p { "Deseja realmente excluir o agendamento ", strong { "{app.title}" }, "?" }
                                p { class: "text-muted font-xs mt-2", "Os vínculos de profissionais e insumos planejados serão removidos." }
                            }
                            div { class: "modal-footer-actions",
                                button { class: "btn-secondary", onclick: move |_| is_delete_modal_open.set(false), "Cancelar" }
                                button {
                                    class: "btn-danger",
                                    disabled: is_deleting(),
                                    onclick: move |e| handle_confirm_delete(e),
                                    if is_deleting() { "Excluindo..." } else { "Sim, Excluir" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
