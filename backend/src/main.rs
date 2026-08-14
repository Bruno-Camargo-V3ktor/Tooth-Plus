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
            .unwrap_or_else(|_| "http://localhost:8080/uploads".into()),
    };
    let storage_provider = storage::build_storage_provider(storage_config).await;

    let evolution_url =
        env::var("EVOLUTION_API_URL").unwrap_or_else(|_| "http://localhost:8080".into());
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
            .wrap(cors) // Injeta o middleware de CORS
            .app_data(db_data.clone())
            .app_data(storage_data.clone())
            .app_data(evolution_data.clone())
            .service(
                web::scope("/api")
                    .service(routes::auth::login)
                    .service(routes::users::create_user)
                    .service(routes::users::list_users)
                    .service(routes::users::update_user)
                    .service(routes::users::toggle_status)
                    .service(routes::users::delete_user),
            )
    })
    .bind(("127.0.0.1", port.parse::<u16>().unwrap_or(4000)))?
    .run()
    .await
}
