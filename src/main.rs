mod app;
mod views;

async fn run_migrations(pool: &sqlx::PgPool) {
    let mut entries: Vec<_> = std::fs::read_dir("database/migrations")
        .expect("database/migrations not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let sql = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("failed to read {:?}", path));
        let sql: &'static str = Box::leak(sql.into_boxed_str());

        sqlx::raw_sql(sql)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("failed to run {:?}: {e}", path));
    }
}

use postgresql_embedded::PostgreSQL;
use sqlx::postgres::PgPoolOptions;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (pool, pg) = rt.block_on(async {
        let mut pg = PostgreSQL::default();
        pg.setup().await.expect("failed to setup postgres");
        pg.start().await.expect("failed to start postgres");

        if !pg.database_exists("aries").await.unwrap_or(false) {
            pg.create_database("aries").await.expect("failed to create database");
        }

        let url = pg.settings().url("aries");

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("failed to connect to postgres");

        run_migrations(&pool).await;

        (pool, pg)
    });

    eframe::run_native(
        "Aries",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(app::App::new(pool)))),
    )
    .expect("failed to start app");

    rt.block_on(async {
        pg.stop().await.expect("failed to stop postgres");
    });
}
