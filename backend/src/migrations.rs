use crate::db::Db;
use std::env;
use std::fs;
use std::path::Path;

pub async fn run_migrations(db: &Db) {
    let raw_dir = env::var("MIGRATIONS_DIR").unwrap_or_else(|_| "migrations".to_string());
    let path = if Path::new(&raw_dir).is_dir() {
        Path::new(&raw_dir)
    } else if Path::new("migrations").is_dir() {
        Path::new("migrations")
    } else if Path::new("../migrations").is_dir() {
        Path::new("../migrations")
    } else {
        println!("Migration directory not found: {}", raw_dir);
        return;
    };

    println!("Starting migrations from: {:?}", path);

    // Ensure _migrations tracking table exists
    let _ = db.query("DEFINE TABLE IF NOT EXISTS _migrations SCHEMALESS;").await;

    let mut files: Vec<_> = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read migration directory"))
        .filter_map(Result::ok)
        .collect();

    files.sort_by_key(|dir| dir.path());

    for file in files {
        let file_path = file.path();

        if file_path.extension().and_then(|s| s.to_str()) == Some("surql") {
            let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();

            // Check if migration has already been executed
            let check_res = db
                .query("SELECT * FROM _migrations WHERE name = $name")
                .bind(("name", file_name.clone()))
                .await;

            if let Ok(mut res) = check_res {
                let existing: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
                if !existing.is_empty() {
                    println!("Migration {} already executed. Skipping.", file_name);
                    continue;
                }
            }

            println!("Executing migration: {}", file_name);

            let content = match fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    println!("Failed to read {}: {}", file_name, e);
                    continue;
                }
            };

            match db.query(&content).await {
                Ok(mut response) => {
                    let errors = response.take_errors();
                    if !errors.is_empty() {
                        println!("Error in migration {}: {:?}", file_name, errors);
                    } else {
                        println!("Migration {} executed successfully.", file_name);
                        let _ = db
                            .query("CREATE _migrations CONTENT { name: $name, executed_at: time::now() }")
                            .bind(("name", file_name))
                            .await;
                    }
                }
                Err(e) => {
                    println!("Failed to execute migration {}: {}", file_name, e);
                }
            }
        }
    }

    println!("Migrations finished.");
}
