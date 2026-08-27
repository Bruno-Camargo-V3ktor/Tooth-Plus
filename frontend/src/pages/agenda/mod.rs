pub mod components;

use crate::api::appointments::AppointmentsApi;
use crate::api::mock_db::DB;
use crate::api::patients::PatientsApi;
use crate::api::ActiveClinicState;
use crate::components::toast::{ToastState, ToastVariant};
use shared::appointments::{
    AppointmentResponse, AppointmentStatus, AppointmentType, AssignedUserDto,
    CreateAppointmentRequest, UpdateAppointmentRequest, UpdateAppointmentStatusRequest,
};
use shared::patients::Patient;
use dioxus::prelude::*;

pub use components::*;

const STYLE: Asset = asset!("/src/pages/agenda/style.css");

#[derive(Clone, PartialEq)]
pub struct PopoverPos {
    pub x: f64,
    pub y: f64,
    pub appointment_id: String,
}

#[component]
pub fn AgendaView() -> Element {
    let active_clinic = consume_context::<Signal<Option<ActiveClinicState>>>();
    let toast = consume_context::<ToastState>();

    let clinic_id = active_clinic
        .read()
        .as_ref()
        .map(|c| c.clinic_id.clone())
        .unwrap_or_default();

    let mut open_hour = use_signal(|| 8u32);
    let mut close_hour = use_signal(|| 19u32);
    let mut clinic_labels = use_signal(Vec::<String>::new);

    let mut appointments = use_signal(Vec::<AppointmentResponse>::new);
    let mut patients_list = use_signal(Vec::<Patient>::new);
    let mut selected_patient_id = use_signal(String::new);
    let mut selected_label = use_signal(String::new);
    let mut reload_trigger = use_signal(|| 0);

    let mut view_mode = use_signal(|| "week".to_string());
    let mut current_date_str = use_signal(|| "2026-08-26".to_string());
    let mut dentist_filter = use_signal(|| "all".to_string());
    let mut show_new_modal = use_signal(|| false);
    let mut editing_appointment_id = use_signal(|| None::<String>);
    let mut popover = use_signal(|| None::<PopoverPos>);

    let mut is_compromisso = use_signal(|| false);
    let mut patient_query = use_signal(String::new);
    let mut appt_date = use_signal(|| "2026-08-26".to_string());
    let mut appt_time = use_signal(|| "09:00".to_string());
    let mut duration = use_signal(|| 30u32);
    let mut procedure_name = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut assigned_user_id = use_signal(|| "usr:dr_lucas".to_string());

    let cid_eff = clinic_id.clone();
    use_effect(move || {
        let _ = reload_trigger.read();
        let cid = cid_eff.clone();

        if let Ok(db) = DB.lock() {
            if let Some(c) = db.clinics.iter().find(|c| c.id == cid) {
                open_hour.set(c.opening_hour);
                close_hour.set(c.closing_hour);
                clinic_labels.set(c.appointment_labels.clone());
            }
        }

        spawn(async move {
            if let Ok(resps) = AppointmentsApi::list_appointments(&cid, None).await {
                appointments.set(resps);
            }
            if let Ok(pats) = PatientsApi::list_patients(None).await {
                patients_list.set(pats.items);
            }
        });
    });

    let days = vec![
        DayColumn { name: "Seg".to_string(), num: "24".to_string(), date_str: "2026-08-24".to_string(), is_today: false },
        DayColumn { name: "Ter".to_string(), num: "25".to_string(), date_str: "2026-08-25".to_string(), is_today: false },
        DayColumn { name: "Qua".to_string(), num: "26".to_string(), date_str: "2026-08-26".to_string(), is_today: true },
        DayColumn { name: "Qui".to_string(), num: "27".to_string(), date_str: "2026-08-27".to_string(), is_today: false },
        DayColumn { name: "Sex".to_string(), num: "28".to_string(), date_str: "2026-08-28".to_string(), is_today: false },
        DayColumn { name: "Sáb".to_string(), num: "29".to_string(), date_str: "2026-08-29".to_string(), is_today: false },
        DayColumn { name: "Dom".to_string(), num: "30".to_string(), date_str: "2026-08-30".to_string(), is_today: false },
    ];

    let handle_submit = {
        let cid = clinic_id.clone();
        let mut toast_c = toast.clone();
        let mut modal_sig = show_new_modal;
        let mut reload_sig = reload_trigger;
        let mut edit_id_sig = editing_appointment_id;

        let p_query = patient_query.clone();
        let p_id_sig = selected_patient_id.clone();
        let a_date = appt_date.clone();
        let a_time = appt_time.clone();
        let dur = duration.clone();
        let obs = notes.clone();
        let usr_id = assigned_user_id.clone();

        move |_| {
            let pat = p_query.read().trim().to_string();
            if pat.is_empty() {
                toast_c.show("Informe o nome do paciente ou selecione um cadastrado.", ToastVariant::Error);
                return;
            }
            let scheduled_for = format!("{}T{}:00Z", a_date.read(), a_time.read());
            let assigned_users = vec![AssignedUserDto {
                user_id: usr_id.read().clone(),
                user_name: Some("Dr. Lucas Mendes".to_string()),
                role_in_appointment: "Dentista Principal".to_string(),
                split_percentage: 100,
            }];

            let pid_val = if p_id_sig.read().is_empty() { None } else { Some(p_id_sig.read().clone()) };
            let edit_opt = edit_id_sig.read().clone();

            let mut toast_resp = toast_c.clone();
            let mut reload_c = reload_sig;
            let mut modal_c = modal_sig;
            let mut edit_id_reset = edit_id_sig;
            let cid_call = cid.clone();

            spawn(async move {
                if let Some(ref edit_id) = edit_opt {
                    let req = UpdateAppointmentRequest {
                        title: Some(pat.clone()),
                        scheduled_for: Some(scheduled_for),
                        duration_minutes: Some(*dur.read() as i32),
                        appointment_type: Some(AppointmentType::Consultation),
                        patient_id: pid_val,
                        patient_name: Some(pat),
                        treatment_id: None,
                        treatment_plan_id: None,
                        financial_amount_cents: None,
                        financial_type: None,
                        notes: Some(obs.read().clone()),
                        assigned_users: Some(assigned_users),
                        consumed_items: None,
                        assigned_equipment: None,
                    };
                    match AppointmentsApi::update_appointment(edit_id, req).await {
                        Ok(_) => {
                            toast_resp.show("Agendamento atualizado com sucesso!", ToastVariant::Success);
                            modal_c.set(false);
                            edit_id_reset.set(None);
                            reload_c.set(reload_c() + 1);
                        }
                        Err(err) => toast_resp.show(err, ToastVariant::Error),
                    }
                } else {
                    let req = CreateAppointmentRequest {
                        clinic_id: cid_call,
                        patient_id: pid_val,
                        patient_name: Some(pat.clone()),
                        treatment_id: None,
                        treatment_plan_id: None,
                        title: pat,
                        scheduled_for,
                        duration_minutes: *dur.read() as i32,
                        appointment_type: AppointmentType::Consultation,
                        financial_amount_cents: None,
                        financial_type: None,
                        notes: Some(obs.read().clone()),
                        assigned_users,
                        consumed_items: vec![],
                        assigned_equipment: None,
                    };
                    match AppointmentsApi::create_appointment(req).await {
                        Ok(_) => {
                            toast_resp.show("Agendamento criado com sucesso!", ToastVariant::Success);
                            modal_c.set(false);
                            reload_c.set(reload_c() + 1);
                        }
                        Err(err) => toast_resp.show(err, ToastVariant::Error),
                    }
                }
            });
        }
    };

    let selected_app = popover.read().as_ref().and_then(|p| {
        appointments.read().iter().find(|a| a.id == p.appointment_id).cloned()
    });

    let mut reload_status = reload_trigger;
    let mut reload_cancel = reload_trigger;

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE }

        div { class: "agenda-page",
            AgendaToolbar {
                dentist_filter,
                view_mode,
                current_date_str,
                month_label: "Ago 2026".to_string(),
                on_prev: move |_| current_date_str.set("2026-08-19".to_string()),
                on_today: move |_| current_date_str.set("2026-08-26".to_string()),
                on_next: move |_| current_date_str.set("2026-09-02".to_string()),
                on_open_new: move |_| {
                    editing_appointment_id.set(None);
                    selected_patient_id.set(String::new());
                    patient_query.set(String::new());
                    show_new_modal.set(true);
                },
            }

            AgendaGrid {
                view_mode: view_mode(),
                days: days.clone(),
                open_hour: open_hour(),
                close_hour: close_hour(),
                appointments: appointments(),
                dentist_filter: dentist_filter(),
                on_slot_click: move |(date_val, hour_val)| {
                    editing_appointment_id.set(None);
                    appt_date.set(date_val);
                    appt_time.set(format!("{:02}:00", hour_val));
                    selected_patient_id.set(String::new());
                    patient_query.set(String::new());
                    show_new_modal.set(true);
                },
                on_event_click: move |(x, y, app_id)| {
                    popover.set(Some(PopoverPos { x, y, appointment_id: app_id }));
                },
            }

            if let (Some(pos), Some(app)) = (popover(), selected_app) {
                {
                    let app_id_for_status = app.id.clone();
                    let app_id_for_cancel = app.id.clone();

                    rsx! {
                        AppointmentPopover {
                            app: app.clone(),
                            x: pos.x,
                            y: pos.y,
                            on_close: move |_| popover.set(None),
                            on_edit: move |app_to_edit: AppointmentResponse| {
                                popover.set(None);
                                editing_appointment_id.set(Some(app_to_edit.id.clone()));
                                selected_patient_id.set(app_to_edit.patient_id.clone().unwrap_or_default());
                                patient_query.set(app_to_edit.patient_name.clone().unwrap_or(app_to_edit.title.clone()));
                                let (h, m) = components::event_card::extract_hhmm(&app_to_edit.scheduled_for);
                                let date_only = app_to_edit.scheduled_for.split('T').next().unwrap_or("2026-08-26").to_string();
                                appt_date.set(date_only);
                                appt_time.set(format!("{:02}:{:02}", h, m));
                                duration.set(app_to_edit.duration_minutes.max(15) as u32);
                                notes.set(app_to_edit.notes.clone().unwrap_or_default());
                                if let Some(u) = app_to_edit.assigned_users.first() {
                                    assigned_user_id.set(u.user_id.clone());
                                }
                                show_new_modal.set(true);
                            },
                            on_change_status: move |new_st_str: String| {
                                let aid = app_id_for_status.clone();
                                let mut pop_sig = popover;
                                let mut reload_c = reload_status;

                                let new_status = match new_st_str.as_str() {
                                    "confirmed" => AppointmentStatus::Confirmed,
                                    "waiting" => AppointmentStatus::Waiting,
                                    "in_progress" => AppointmentStatus::InProgress,
                                    "completed" => AppointmentStatus::Completed,
                                    "no_show" => AppointmentStatus::NoShow,
                                    "canceled_pat" => AppointmentStatus::CanceledByPatient,
                                    "canceled_doc" => AppointmentStatus::CanceledByDoctor,
                                    "canceled" => AppointmentStatus::Canceled,
                                    _ => AppointmentStatus::Pending,
                                };

                                spawn(async move {
                                    let req = UpdateAppointmentStatusRequest {
                                        status: new_status,
                                        cancellation_reason: None,
                                        consumed_items: None,
                                    };
                                    let _ = AppointmentsApi::update_appointment_status(&aid, req).await;
                                    pop_sig.set(None);
                                    reload_c.set(reload_c() + 1);
                                });
                            },
                            on_cancel: move |_| {
                                let aid = app_id_for_cancel.clone();
                                let mut pop_sig = popover;
                                let mut reload_c = reload_cancel;
                                spawn(async move {
                                    let req = UpdateAppointmentStatusRequest {
                                        status: AppointmentStatus::Canceled,
                                        cancellation_reason: Some("Cancelado via Popover".to_string()),
                                        consumed_items: None,
                                    };
                                    let _ = AppointmentsApi::update_appointment_status(&aid, req).await;
                                    pop_sig.set(None);
                                    reload_c.set(reload_c() + 1);
                                });
                            },
                        }
                    }
                }
            }

            ModalAppointment {
                is_open: show_new_modal(),
                patients: patients_list(),
                labels: clinic_labels(),
                selected_patient_id,
                selected_label,
                on_close: move |_| {
                    editing_appointment_id.set(None);
                    show_new_modal.set(false);
                },
                on_submit: handle_submit,
                is_compromisso,
                patient_query,
                appt_date,
                appt_time,
                duration,
                procedure_name,
                notes,
                assigned_user_id,
            }
        }
    }
}
