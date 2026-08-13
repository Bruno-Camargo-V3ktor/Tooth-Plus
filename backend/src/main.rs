use std::env;

use actix_web::{App, HttpServer, web};

use crate::storage::StorageConfig;

mod auth_guard;
mod crypto;
mod db;
mod migrations;
mod routes;
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

    let db_data = web::Data::new(db_client.clone());
    let storage_data = web::Data::from(storage_provider);

    println!("Server run in http://127.0.0.1:4000");
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .app_data(storage_data.clone())
            .service(
                web::scope("/api")
                    .service(routes::auth::login)
                    .service(routes::users::create_user)
                    .service(routes::users::list_users)
                    .service(routes::users::update_user)
                    .service(routes::users::toggle_status),
            )
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
