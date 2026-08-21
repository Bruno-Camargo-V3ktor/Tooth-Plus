use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer};
use std::env;

mod db;
pub mod documents_pdf;
pub mod email;
mod error;
mod evolution;
mod migrations;
mod routes;
mod security;
mod storage;

use evolution::EvolutionClient;
use storage::{build_storage_provider, StorageConfig};

pub fn resolve_uploads_dir() -> String {
    if let Ok(bucket) = env::var("STORAGE_BUCKET") {
        if std::path::Path::new(&bucket).is_dir() {
            return bucket;
        }
        let alt = format!("../{}", bucket.trim_start_matches("./"));
        if std::path::Path::new(&alt).is_dir() {
            return alt;
        }
    }
    if std::path::Path::new("uploads").is_dir() {
        return "uploads".to_string();
    }
    if std::path::Path::new("../uploads").is_dir() {
        return "../uploads".to_string();
    }
    "uploads".to_string()
}

#[get("/uploads/{filename:.*}")]
pub async fn serve_uploads(
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let raw_path = path.into_inner();
    let upload_dir = resolve_uploads_dir();
    let sanitized = raw_path.replace("..", "");
    let full_path = std::path::Path::new(&upload_dir).join(&sanitized);

    if !full_path.exists() || !full_path.is_file() {
        return Ok(HttpResponse::NotFound().body("Arquivo não encontrado."));
    }

    let named_file = actix_files::NamedFile::open_async(&full_path).await?;
    let mut response = named_file.into_response(&req);

    let mime_str = if sanitized.ends_with(".pdf") {
        "application/pdf"
    } else if sanitized.ends_with(".png") {
        "image/png"
    } else if sanitized.ends_with(".jpg") || sanitized.ends_with(".jpeg") {
        "image/jpeg"
    } else if sanitized.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    };

    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(mime_str),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("inline"),
    );

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let db_instance = db::init_db().await.expect("Failed to connect to DB");
    let db_data = web::Data::new(db_instance);

    migrations::run_migrations(&db_data).await;

    let upload_dir = resolve_uploads_dir();
    let _ = std::fs::create_dir_all(&upload_dir);
    documents_pdf::ensure_sample_template_pdf(&upload_dir);
    let storage_config = StorageConfig {
        provider_type: env::var("STORAGE_PROVIDER").unwrap_or_else(|_| "local".into()),
        bucket_name: upload_dir.clone(),
        region: env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".into()),
        endpoint_url: env::var("STORAGE_ENDPOINT").unwrap_or_default(),
        access_key: env::var("STORAGE_ACCESS_KEY").unwrap_or_default(),
        secret_key: env::var("STORAGE_SECRET_KEY").unwrap_or_default(),
        public_url: env::var("STORAGE_PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:4000/uploads".into()),
    };

    let storage_provider = build_storage_provider(storage_config).await;
    let storage_data = web::Data::new(storage_provider);

    let evo_base = env::var("EVOLUTION_API_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let evolution_client = EvolutionClient::new(evo_base);
    let evolution_data = web::Data::new(evolution_client);

    let port: u16 = env::var("PORT")
        .or_else(|_| env::var("SERVER_PORT"))
        .unwrap_or_else(|_| "4000".into())
        .parse()
        .unwrap_or(4000);

    let frontend_url_env = env::var("FRONTEND_URL").unwrap_or_else(|_| "*".to_string());

    let allowed_origins: Vec<String> = frontend_url_env
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    println!("Server run in http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        if allowed_origins.contains(&"*".to_string()) {
            cors = cors.allow_any_origin();
        } else {
            for origin in &allowed_origins {
                cors = cors.allowed_origin(origin.as_str());
            }
        }

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .app_data(storage_data.clone())
            .app_data(evolution_data.clone())
            .service(serve_uploads)
            .service(
                web::scope("/api")
                    .service(routes::auth::login)
                    .service(routes::clinics::get_clinic)
                    .service(routes::clinics::update_clinic)
                    .service(routes::clinics::delete_clinic)
                    .service(routes::clinics::upload_logo)
                    .service(routes::clinics::get_whatsapp_qr)
                    .service(routes::users::create_user)
                    .service(routes::users::list_users)
                    .service(routes::users::update_user)
                    .service(routes::users::toggle_status)
                    .service(routes::users::delete_user)
                    .service(routes::appointments::list_appointments)
                    .service(routes::appointments::get_agenda_resources)
                    .service(routes::appointments::create_appointment)
                    .service(routes::appointments::update_appointment)
                    .service(routes::appointments::update_appointment_status)
                    .service(routes::appointments::delete_appointment)
                    .service(routes::finance::get_finance_data)
                    .service(routes::finance::create_transaction)
                    .service(routes::finance::update_transaction_status)
                    .service(routes::finance::delete_transaction)
                    .service(routes::stock::get_stock_data)
                    .service(routes::stock::create_stock_item)
                    .service(routes::stock::update_stock_item)
                    .service(routes::stock::delete_stock_item)
                    .service(routes::stock::create_stock_movement)
                    .service(routes::stock::upload_stock_document)
                    .service(routes::patients::list_patients)
                    .service(routes::patients::create_patient)
                    .service(routes::patients::get_patient_details)
                    .service(routes::patients::update_patient)
                    .service(routes::patients::reset_patient_password)
                    .service(routes::patients::delete_patient)
                    .service(routes::patients::save_anamnesis)
                    .service(routes::patients::get_anamnesis_templates)
                    .service(routes::patients::save_anamnesis_template)
                    .service(routes::patients::sync_patient_anamnesis)
                    .service(routes::patients::create_exam)
                    .service(routes::patients::update_exam)
                    .service(routes::patients::delete_exam)
                    .service(routes::patients::create_treatment)
                    .service(routes::patients::update_treatment)
                    .service(routes::patients::delete_treatment)
                    .service(routes::documents::list_documents)
                    .service(routes::documents::create_patient_document)
                    .service(routes::documents::delete_patient_document)
                    .service(routes::documents::list_templates)
                    .service(routes::documents::create_template)
                    .service(routes::documents::update_template)
                    .service(routes::documents::delete_template)
                    .service(routes::documents::upload_document_pdf)
                    .service(routes::documents::get_sample_template_pdf)
                    .service(routes::documents::get_public_signing_document)
                    .service(routes::documents::check_patient_signing)
                    .service(routes::documents::register_patient_password)
                    .service(routes::documents::auth_patient_signing)
                    .service(routes::documents::auth_doctor_signing)
                    .service(routes::documents::request_signing_otp)
                    .service(routes::documents::submit_digital_signature)
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
