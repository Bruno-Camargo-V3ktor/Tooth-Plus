use actix_web::{App, HttpServer, web};

mod auth_guard;
mod crypto;
mod db;
mod migrations;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    println!("Database connection initiated...");

    let db_client = db::init_db().await.expect("Failed to connect to SurrealDB");
    let db_data = web::Data::new(db_client.clone());

    println!("Server run in http://127.0.0.1:4000");

    migrations::run_migrations(&db_client).await;

    HttpServer::new(move || {
        App::new().app_data(db_data.clone()).service(
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
