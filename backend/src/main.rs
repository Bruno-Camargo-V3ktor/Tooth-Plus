use crate::{evolution::EvolutionClient, storage::StorageConfig};
use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header, web};
use std::env;

mod db;
mod error;
mod evolution;
mod migrations;
mod routes;
mod security;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    println!("Database connection initiated...");
    let db_client = db::init_db().await.expect("Failed to connect to SurrealDB");
    migrations::run_migrations(&db_client).await;

    let storage_config = StorageConfig {
        provider_type: env::var("STORAGE_PROVIDER").unwrap_or_else(|_| "local".into()),
        bucket_name: env::var("STORAGE_BUCKET").unwrap_or_else(|_| "./uploads".into()),
        region: env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".into()),
        endpoint_url: env::var("STORAGE_ENDPOINT").unwrap_or_default(),
        access_key: env::var("STORAGE_ACCESS_KEY").unwrap_or_default(),
        secret_key: env::var("STORAGE_SECRET_KEY").unwrap_or_default(),
        public_url: env::var("STORAGE_PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:4000/uploads".into()),
    };
    let storage_provider = storage::build_storage_provider(storage_config).await;

    let evolution_url =
        env::var("EVOLUTION_API_URL").unwrap_or_else(|_| "http://localhost:8081".into());
    let evolution_client = EvolutionClient::new(evolution_url);

    let db_data = web::Data::new(db_client.clone());
    let storage_data = web::Data::from(storage_provider);
    let evolution_data = web::Data::new(evolution_client);

    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "4000".to_string());
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
                    .service(routes::patients::delete_patient)
                    .service(routes::patients::save_anamnesis)
                    .service(routes::patients::create_exam)
                    .service(routes::patients::create_treatment)
                    .service(routes::documents::list_documents)
                    .service(routes::documents::create_patient_document)
                    .service(routes::documents::delete_patient_document)
                    .service(routes::documents::list_templates)
                    .service(routes::documents::create_template)
                    .service(routes::documents::update_template)
                    .service(routes::documents::delete_template)
                    .service(routes::documents::upload_document_pdf)
                    .service(routes::documents::get_public_signing_document)
                    .service(routes::documents::auth_patient_signing)
                    .service(routes::documents::auth_doctor_signing)
                    .service(routes::documents::request_signing_otp)
                    .service(routes::documents::submit_digital_signature),
            )
    })
    .bind(("127.0.0.1", port.parse::<u16>().unwrap_or(4000)))?
    .run()
    .await
}
