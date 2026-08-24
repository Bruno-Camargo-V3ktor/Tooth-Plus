//! # Mini Banco de Dados Mock em Memória (Tooth Plus V2)
//!
//! Fornece entidades, listagens e métodos simulados para desenvolvimento da UI
//! antes da integração direta com os endpoints reais do backend.

use shared::appointments::{AppointmentResponse, AppointmentStatus, AppointmentType, AssignedUserDto};
use shared::auth::{LoginRequest, LoginResponse};
use shared::models::ClinicAccess;
use shared::patients::Patient;

/// Sessão ativa do usuário autenticado no frontend.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionState {
    pub token: String,
    pub user_id: String,
    pub full_name: String,
    pub clinics: Vec<ClinicAccess>,
}

/// Clínica atualmente selecionada para o escopo de trabalho.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveClinicState {
    pub clinic_id: String,
    pub trading_name: String,
    pub theme_color: String,
    pub logo_url: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
}

/// Base mock de clínicas cadastradas.
pub fn get_mock_clinics() -> Vec<ClinicAccess> {
    vec![
        ClinicAccess {
            clinic_id: "clinic:smile_plus".to_string(),
            trading_name: "Smile Plus - Matriz".to_string(),
            theme_color: "#0284c7".to_string(),
            logo_url: None,
            role: "admin".to_string(),
            permissions: vec![
                "patients:read".into(),
                "patients:write".into(),
                "agenda:read".into(),
                "agenda:write".into(),
                "finance:read".into(),
                "finance:write".into(),
                "stock:read".into(),
                "stock:write".into(),
                "treatments:read".into(),
                "treatments:write".into(),
            ],
        },
        ClinicAccess {
            clinic_id: "clinic:luria_dent".to_string(),
            trading_name: "Luria Odontologia Integrada".to_string(),
            theme_color: "#0d9488".to_string(),
            logo_url: None,
            role: "dentist".to_string(),
            permissions: vec![
                "patients:read".into(),
                "patients:write".into(),
                "agenda:read".into(),
                "agenda:write".into(),
                "treatments:read".into(),
                "treatments:write".into(),
            ],
        },
    ]
}

/// Base mock de pacientes.
pub fn get_mock_patients() -> Vec<Patient> {
    vec![
        Patient {
            id: "patient:mariana_castro".to_string(),
            clinic_id: "clinic:smile_plus".to_string(),
            full_name: "Mariana Castro Fernandes".to_string(),
            document_cpf: Some("123.456.789-00".to_string()),
            document_rg: Some("MG-12.345.678".to_string()),
            phone: "(11) 98765-4321".to_string(),
            email: Some("mariana.castro@exemplo.com".to_string()),
            birth_date: Some("1994-05-18".to_string()),
            gender: Some("female".to_string()),
            marital_status: Some("single".to_string()),
            profession: Some("Arquiteta".to_string()),
            emergency_contact_name: Some("Carlos Fernandes (Pai)".to_string()),
            emergency_contact_phone: Some("(11) 98111-2233".to_string()),
            address_street: Some("Av. Paulista".to_string()),
            address_number: Some("1000".to_string()),
            address_complement: Some("Apto 42".to_string()),
            address_neighborhood: Some("Bela Vista".to_string()),
            address_city: Some("São Paulo".to_string()),
            address_state: Some("SP".to_string()),
            address_zip: Some("01310-100".to_string()),
            insurance_plan: Some("Unimed Odonto".to_string()),
            insurance_number: Some("88492019".to_string()),
            legal_guardians: vec![],
            legal_guardian_name: None,
            legal_guardian_cpf: None,
            has_signature_password: false,
            created_at: "2026-08-01T10:00:00Z".to_string(),
            updated_at: "2026-08-20T14:30:00Z".to_string(),
        },
        Patient {
            id: "patient:lucas_souza".to_string(),
            clinic_id: "clinic:smile_plus".to_string(),
            full_name: "Lucas Gabriel de Souza".to_string(),
            document_cpf: Some("987.654.321-99".to_string()),
            document_rg: None,
            phone: "(11) 97654-3210".to_string(),
            email: Some("lucas.souza@exemplo.com".to_string()),
            birth_date: Some("1988-11-25".to_string()),
            gender: Some("male".to_string()),
            marital_status: Some("married".to_string()),
            profession: Some("Engenheiro".to_string()),
            emergency_contact_name: None,
            emergency_contact_phone: None,
            address_street: Some("Rua Augusta".to_string()),
            address_number: Some("450".to_string()),
            address_complement: None,
            address_neighborhood: Some("Consolação".to_string()),
            address_city: Some("São Paulo".to_string()),
            address_state: Some("SP".to_string()),
            address_zip: Some("01305-000".to_string()),
            insurance_plan: None,
            insurance_number: None,
            legal_guardians: vec![],
            legal_guardian_name: None,
            legal_guardian_cpf: None,
            has_signature_password: false,
            created_at: "2026-08-05T09:15:00Z".to_string(),
            updated_at: "2026-08-18T16:20:00Z".to_string(),
        },
    ]
}

/// Base mock de agendamentos para exibição na Agenda.
pub fn get_mock_appointments() -> Vec<AppointmentResponse> {
    vec![
        AppointmentResponse {
            id: "app:1".to_string(),
            clinic_id: "clinic:smile_plus".to_string(),
            patient_id: Some("patient:mariana_castro".to_string()),
            patient_name: Some("Mariana Castro Fernandes".to_string()),
            treatment_id: None,
            treatment_plan_id: None,
            title: "Restauração Estética Resina".to_string(),
            scheduled_for: "2026-08-24T14:00:00Z".to_string(),
            duration_minutes: 30,
            status: AppointmentStatus::Confirmed,
            appointment_type: AppointmentType::Treatment,
            financial_amount_cents: Some(18000),
            financial_type: Some("particular".to_string()),
            notes: Some("Dente 21 - Faces V, M".to_string()),
            cancellation_reason: None,
            assigned_users: vec![
                AssignedUserDto {
                    user_id: "user:dr_lucas".to_string(),
                    user_name: Some("Dr. Lucas Mendes".to_string()),
                    role_in_appointment: "Cirurgião Dentista".to_string(),
                    split_percentage: 50,
                }
            ],
            consumed_items: vec![],
            assigned_equipment: vec![],
        },
        AppointmentResponse {
            id: "app:2".to_string(),
            clinic_id: "clinic:smile_plus".to_string(),
            patient_id: Some("patient:lucas_souza".to_string()),
            patient_name: Some("Lucas Gabriel de Souza".to_string()),
            treatment_id: None,
            treatment_plan_id: None,
            title: "Extração de Siso 38".to_string(),
            scheduled_for: "2026-08-25T14:00:00Z".to_string(),
            duration_minutes: 60,
            status: AppointmentStatus::Confirmed,
            appointment_type: AppointmentType::Surgery,
            financial_amount_cents: Some(45000),
            financial_type: Some("particular".to_string()),
            notes: Some("Kit cirúrgico preparado".to_string()),
            cancellation_reason: None,
            assigned_users: vec![
                AssignedUserDto {
                    user_id: "user:dr_lucas".to_string(),
                    user_name: Some("Dr. Lucas Mendes".to_string()),
                    role_in_appointment: "Cirurgião Dentista".to_string(),
                    split_percentage: 50,
                }
            ],
            consumed_items: vec![],
            assigned_equipment: vec![],
        },
    ]
}

/// Simula autenticação com login mockado.
pub async fn mock_login_call(req: LoginRequest) -> Result<LoginResponse, String> {
    // Delay simulado de 300ms
    gloo_timers::future::TimeoutFuture::new(300).await;

    if req.username.trim().is_empty() || req.password_plain.trim().is_empty() {
        return Err("Informe o usuário e a senha.".to_string());
    }

    if req.username == "erro" {
        return Err("Credenciais inválidas. Verifique usuário e senha.".to_string());
    }

    Ok(LoginResponse {
        token: "mock_jwt_token_toothplus_v2".to_string(),
        user_id: "user:admin_principal".to_string(),
        full_name: "Dr. Roberto Alencar".to_string(),
        clinics: get_mock_clinics(),
    })
}
