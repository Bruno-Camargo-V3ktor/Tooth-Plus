use shared::appointments::{AppointmentResponse, AppointmentStatus};
use dioxus::prelude::*;

pub fn appointment_status_css(status: &AppointmentStatus) -> &'static str {
    match status {
        AppointmentStatus::Confirmed => "status-confirmed",
        AppointmentStatus::Completed => "status-finalized",
        AppointmentStatus::InProgress => "status-waiting",
        AppointmentStatus::Pending => "status-pending",
        AppointmentStatus::Canceled | AppointmentStatus::CanceledByDoctor | AppointmentStatus::CanceledByPatient => "status-cancelled",
        _ => "status-pending",
    }
}

pub fn extract_hhmm(date_str: &str) -> (u32, u32) {
    if let Some(time_part) = date_str.split('T').nth(1) {
        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() >= 2 {
            let h = parts[0].parse().unwrap_or(8);
            let m = parts[1].parse().unwrap_or(0);
            return (h, m);
        }
    }
    (8, 0)
}

#[component]
pub fn EventCard(
    app: AppointmentResponse,
    idx: usize,
    on_click: EventHandler<(f64, f64, String)>,
) -> Element {
    let app_id = app.id.clone();
    let (_, app_min) = extract_hhmm(&app.scheduled_for);
    let dur: u32 = app.duration_minutes as u32;
    let top_offset = (app_min as f64 / 60.0) * 100.0;
    let height = (dur as f64 / 60.0 * 100.0_f64).max(24.0) - 4.0;
    let left_pct = idx as f64 * 50.0;
    let status_css = appointment_status_css(&app.status);

    let patient_name = app.patient_name.clone().unwrap_or_else(|| app.title.clone());
    let (ah, am) = extract_hhmm(&app.scheduled_for);
    let end_min = am + dur;
    let end_h = ah + end_min / 60;
    let end_m = end_min % 60;
    let time_str = format!("{:02}h{:02} - {:02}h{:02}", ah, am, end_h, end_m);

    rsx! {
        div {
            class: "event-card {status_css}",
            style: format!(
                "top: {}px; height: {}px; left: calc({}% + 2px); right: 2px; z-index: {};",
                top_offset, height, left_pct, 5 + idx
            ),
            onclick: move |e| {
                e.stop_propagation();
                let rect = e.client_coordinates();
                on_click.call((rect.x as f64, rect.y as f64, app_id.clone()));
            },
            div { class: "event-time", "{time_str}" }
            div { class: "event-name", "{patient_name}" }
        }
    }
}
