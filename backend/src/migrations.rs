use crate::db::Db;
use std::env;
use std::fs;
use std::path::Path;

pub async fn run_migrations(db: &Db) {
    let migrations_dir = match env::var("MIGRATIONS_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            println!("MIGRATIONS_DIR not set. Skipping automatic migrations.");
            return;
        }
    };

    let path = Path::new(&migrations_dir);

    if !path.exists() || !path.is_dir() {
        println!("Migration directory not found: {}", migrations_dir);
        return;
    }

    println!("Starting migrations from: {}", migrations_dir);

    let mut files: Vec<_> = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read migration directory"))
        .filter_map(Result::ok)
        .collect();

    files.sort_by_key(|dir| dir.path());

    for file in files {
        let file_path = file.path();

        if file_path.extension().and_then(|s| s.to_str()) == Some("surql") {
            let file_name = file_path.file_name().unwrap().to_string_lossy();
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
