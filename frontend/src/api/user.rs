use shared::users::{CreateUserRequest, ToggleStatusRequest, UpdateUserRequest, UserResponse};

pub async fn fetch_users(_clinic_id: &str) -> Result<Vec<UserResponse>, String> {
    Ok(vec![
        UserResponse {
            id: "1".into(),
            username: "admin".into(),
            full_name: "Dr. Admin User".into(),
            document_cpf: "00000000000".into(),
            professional_registry: Some("CRO-SP 12345".into()),
            is_active: true,
            role: "admin".into(),
            permissions: vec!["admin:all".into()],
            clinic_ids: vec!["clinic:1".into(), "clinic:2".into()],
        },
        UserResponse {
            id: "2".into(),
            username: "fernanda.a".into(),
            full_name: "Fernanda Alves".into(),
            document_cpf: "11111111111".into(),
            professional_registry: Some("CRO-RJ 54321".into()),
            is_active: true,
            role: "dentist".into(),
            permissions: vec![
                "patients:read".into(),
                "patients:write".into(),
                "agenda:read".into(),
            ],
            clinic_ids: vec!["clinic:1".into()],
        },
        UserResponse {
            id: "3".into(),
            username: "carlos.rec".into(),
            full_name: "Carlos Recepcionista".into(),
            document_cpf: "22222222222".into(),
            professional_registry: None,
            is_active: false,
            role: "receptionist".into(),
            permissions: vec!["agenda:read".into(), "agenda:write".into()],
            clinic_ids: vec!["clinic:1".into()],
        },
    ])
}

pub async fn create_user(_req: CreateUserRequest) -> Result<(), String> {
    Ok(())
}

pub async fn update_user(_id: &str, _req: UpdateUserRequest) -> Result<(), String> {
    Ok(())
}

pub async fn toggle_user_status(_id: &str, _req: ToggleStatusRequest) -> Result<(), String> {
    Ok(())
}

pub async fn delete_user(_id: &str) -> Result<(), String> {
    Ok(())
}
