mod app;
mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::{Arc, Mutex};

use postgresql_embedded::PostgreSQL;

// NOTE: only dev; prod must use a proper migration tool
fn run_migrations(client: &mut postgres::Client) {
    let mut entries: Vec<_> = std::fs::read_dir("database/migrations")
        .expect("database/migrations not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let sql  = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("failed to read {:?}", path));
        client
            .batch_execute(&sql)
            .unwrap_or_else(|e| panic!("failed to run {:?}: {e}", path));
    }
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (pg, url) = rt.block_on(async {
        let mut pg = PostgreSQL::default();
        pg.setup().await.expect("failed to setup postgres");
        pg.start().await.expect("failed to start postgres");

        if !pg.database_exists("aries").await.unwrap_or(false) {
            pg.create_database("aries").await.expect("failed to create database");
        }

        let url = pg.settings().url("aries");
        (pg, url)
    });

    let mut client = postgres::Client::connect(&url, postgres::NoTls)
        .expect("failed to connect to postgres");

    run_migrations(&mut client);

    let client = Arc::new(Mutex::new(client));

    eframe::run_native(
        "Aries",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(app::App::new(client)))),
    )
    .expect("failed to start app");

    rt.block_on(async {
        pg.stop().await.expect("failed to stop postgres");
    });
}
