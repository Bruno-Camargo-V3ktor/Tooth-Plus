use std::env;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;

pub type Db = Surreal<Client>;

pub async fn init_db() -> surrealdb::Result<Db> {
    let host = env::var("DB_HOST").expect("DB_HOST not set");
    let user = env::var("DB_USER").expect("DB_USER not set");
    let pass = env::var("DB_PASS").expect("DB_PASS not set");

    let namespace = env::var("DB_NS").expect("DB_PASS not set");
    let database = env::var("DB_DB").expect("DB_PASS not set");

    let db = Surreal::new::<Ws>(host).await?;

    db.signin(Root {
        username: user.clone(),
        password: pass.clone(),
    })
    .await?;

    db.use_ns(namespace).use_db(database).await?;

    Ok(db)
}
