//! # Módulo de Integração e Serviço de Agenda (AppointmentsApi)

use super::mock_db::DB;
use shared::appointments::{
    AgendaResourceOption, AgendaResourcesResponse, AppointmentResponse, AppointmentStatus,
    CreateAppointmentRequest, UpdateAppointmentRequest, UpdateAppointmentStatusRequest,
};

pub struct AppointmentsApi;

impl AppointmentsApi {
    /// Lista todos os agendamentos da clínica com filtro opcional de data.
    pub async fn list_appointments(clinic_id: &str, _date: Option<&str>) -> Result<Vec<AppointmentResponse>, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let filtered = db
            .appointments
            .iter()
            .filter(|a| a.clinic_id == clinic_id)
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Obtém um agendamento por ID.
    pub async fn get_appointment_by_id(appointment_id: &str) -> Result<AppointmentResponse, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        db.appointments
            .iter()
            .find(|a| a.id == appointment_id)
            .cloned()
            .ok_or_else(|| format!("Agendamento {} não encontrado.", appointment_id))
    }

    /// Cria um novo agendamento na agenda.
    pub async fn create_appointment(req: CreateAppointmentRequest) -> Result<AppointmentResponse, String> {
        gloo_timers::future::TimeoutFuture::new(200).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let app_id = format!("app:{}", db.appointments.len() + 1);

        let new_app = AppointmentResponse {
            id: app_id,
            clinic_id: req.clinic_id,
            patient_id: req.patient_id,
            patient_name: req.patient_name,
            treatment_id: req.treatment_id,
            treatment_plan_id: req.treatment_plan_id,
            title: req.title,
            scheduled_for: req.scheduled_for,
            duration_minutes: req.duration_minutes,
            status: AppointmentStatus::Confirmed,
            appointment_type: req.appointment_type,
            financial_amount_cents: req.financial_amount_cents,
            financial_type: req.financial_type,
            notes: req.notes,
            cancellation_reason: None,
            assigned_users: req.assigned_users,
            consumed_items: req.consumed_items,
            assigned_equipment: req.assigned_equipment.unwrap_or_default(),
        };

        db.appointments.push(new_app.clone());
        Ok(new_app)
    }

    /// Atualiza status e dados de atendimento de uma consulta.
    pub async fn update_appointment_status(
        appointment_id: &str,
        req: UpdateAppointmentStatusRequest,
    ) -> Result<AppointmentResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let app = db
            .appointments
            .iter_mut()
            .find(|a| a.id == appointment_id)
            .ok_or_else(|| format!("Agendamento {} não encontrado.", appointment_id))?;

        app.status = req.status;
        if let Some(reason) = req.cancellation_reason {
            app.cancellation_reason = Some(reason);
        }
        if let Some(items) = req.consumed_items {
            app.consumed_items = items;
        }

        Ok(app.clone())
    }

    /// Atualiza um agendamento existente.
    pub async fn update_appointment(
        appointment_id: &str,
        req: UpdateAppointmentRequest,
    ) -> Result<AppointmentResponse, String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let app = db
            .appointments
            .iter_mut()
            .find(|a| a.id == appointment_id)
            .ok_or_else(|| format!("Agendamento {} não encontrado.", appointment_id))?;

        if let Some(title) = req.title { app.title = title; }
        if let Some(sched) = req.scheduled_for { app.scheduled_for = sched; }
        if let Some(dur) = req.duration_minutes { app.duration_minutes = dur; }
        if let Some(typ) = req.appointment_type { app.appointment_type = typ; }
        if let Some(pid) = req.patient_id { app.patient_id = Some(pid); }
        if let Some(pname) = req.patient_name { app.patient_name = Some(pname); }
        if let Some(cents) = req.financial_amount_cents { app.financial_amount_cents = Some(cents); }
        if let Some(ftype) = req.financial_type { app.financial_type = Some(ftype); }
        if let Some(notes) = req.notes { app.notes = Some(notes); }
        if let Some(users) = req.assigned_users { app.assigned_users = users; }
        if let Some(items) = req.consumed_items { app.consumed_items = items; }
        if let Some(eq) = req.assigned_equipment { app.assigned_equipment = eq; }

        Ok(app.clone())
    }

    /// Remove um agendamento.
    pub async fn delete_appointment(appointment_id: &str) -> Result<(), String> {
        gloo_timers::future::TimeoutFuture::new(150).await;
        let mut db = DB.lock().map_err(|e| e.to_string())?;

        let initial = db.appointments.len();
        db.appointments.retain(|a| a.id != appointment_id);

        if db.appointments.len() == initial {
            return Err(format!("Agendamento {} não encontrado.", appointment_id));
        }

        Ok(())
    }

    /// Carrega recursos de apoio à agenda (profissionais, pacientes, insumos).
    pub async fn get_agenda_resources(_clinic_id: &str) -> Result<AgendaResourcesResponse, String> {
        gloo_timers::future::TimeoutFuture::new(100).await;
        let db = DB.lock().map_err(|e| e.to_string())?;

        let team_members = db
            .users
            .iter()
            .map(|u| AgendaResourceOption {
                id: u.id.clone(),
                name: u.full_name.clone(),
                extra_info: u.professional_registry.clone(),
            })
            .collect();

        let patients = db
            .patients
            .iter()
            .map(|p| AgendaResourceOption {
                id: p.id.clone(),
                name: p.full_name.clone(),
                extra_info: Some(p.phone.clone()),
            })
            .collect();

        let inventory_items = db
            .inventory_items
            .iter()
            .map(|i| AgendaResourceOption {
                id: i.id.clone(),
                name: i.name.clone(),
                extra_info: Some(format!("Estoque: {} {}", i.current_stock, i.unit_type)),
            })
            .collect();

        Ok(AgendaResourcesResponse {
            team_members,
            patients,
            inventory_items,
            equipment_items: vec![],
            pending_treatments: vec![],
        })
    }
}
