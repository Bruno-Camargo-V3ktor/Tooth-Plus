use std::env;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws, Wss};
use surrealdb::opt::auth::Root;

pub type Db = Surreal<Client>;

pub async fn init_db() -> surrealdb::Result<Db> {
    let host = env::var("DB_HOST").expect("DB_HOST not set");
    let user = env::var("DB_USER").expect("DB_USER not set");
    let pass = env::var("DB_PASS").expect("DB_PASS not set");

    let namespace = env::var("DB_NS").expect("DB_NS not set");
    let database = env::var("DB_DB").expect("DB_DB not set");

    let is_secure = host.starts_with("wss://") || host.starts_with("https://") || host.contains("surreal.cloud");

    let db = if is_secure {
        let raw = host.trim_start_matches("https://").trim_start_matches("wss://").trim_end_matches('/');
        let endpoint = if raw.ends_with("/rpc") { raw.to_string() } else { format!("{}/rpc", raw) };
        println!("Connecting to SurrealDB Cloud at: {}", endpoint);
        Surreal::new::<Wss>(endpoint).await?
    } else {
        let raw = host.trim_start_matches("http://").trim_start_matches("ws://").trim_end_matches('/');
        Surreal::new::<Ws>(raw).await?
    };

    db.signin(Root {
        username: user.clone(),
        password: pass.clone(),
    })
    .await?;

    db.use_ns(namespace).use_db(database).await?;

    Ok(db)
}
