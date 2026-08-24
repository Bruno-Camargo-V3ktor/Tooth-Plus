use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "online",
        "service": "Tooth Plus Backend API",
        "version": "0.1.0"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "4000".to_string())
        .parse::<u16>()
        .unwrap_or(4000);

    println!("🚀 Tooth Plus Backend Server iniciando na porta {}...", port);

    HttpServer::new(|| {
        App::new()
            .service(health_check)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
