use shared::clinics::{ClinicAddress, ClinicResponse, UpdateClinicRequest};
use shared::files::FileUploadRequest;

pub async fn fetch_clinic(clinic_id: &str) -> Result<ClinicResponse, String> {
    Ok(ClinicResponse {
        id: clinic_id.to_string(),
        corporate_name: "Tooth Plus Odontologia LTDA".into(),
        trading_name: "Tooth Plus".into(),
        document_cnpj: "12.345.678/0001-90".into(),
        theme_color: "#0f172a".into(),
        logo_url: None,
        whatsapp_instance: Some("tooth_plus_matriz".into()),
        address: ClinicAddress {
            street: "Av. Paulista".into(),
            number: "1000".into(),
            complement: Some("Sala 402".into()),
            neighborhood: "Bela Vista".into(),
            city: "São Paulo".into(),
            state: "SP".into(),
            zip_code: "01310-100".into(),
        },
    })
}

pub async fn update_clinic(_id: &str, _req: UpdateClinicRequest) -> Result<(), String> {
    Ok(())
}

pub async fn upload_clinic_logo(_id: &str, _req: FileUploadRequest) -> Result<String, String> {
    Ok("https://via.placeholder.com/150".into())
}

pub async fn fetch_whatsapp_qr_code(_instance: &str) -> Result<String, String> {
    Ok("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".into())
}
