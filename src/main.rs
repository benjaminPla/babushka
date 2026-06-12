mod app;
mod application;
mod domain;
mod infrastructure;
mod presentation;
mod theme;
mod updater;

use std::sync::{Arc, Mutex};

use log::LevelFilter;
use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller,
            trigger::size::SizeTrigger,
            CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};

use app::{AppWrapper, InitResult, LoadingStatus, UpdateState};

fn init_logger() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("babushka")
        .join("logs");

    std::fs::create_dir_all(&log_dir).expect("failed to create log directory");

    let log_file    = log_dir.join("babushka.log");
    let archive_pat = log_dir.join("babushka.{}.log").to_string_lossy().into_owned();

    let roller   = FixedWindowRoller::builder().build(&archive_pat, 5).unwrap();
    let trigger  = SizeTrigger::new(5 * 1024 * 1024);
    let policy   = CompoundPolicy::new(Box::new(trigger), Box::new(roller));

    let appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}\n")))
        .build(log_file, Box::new(policy))
        .expect("failed to build log appender");

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(appender)))
        .build(Root::builder().appender("file").build(LevelFilter::Info))
        .expect("failed to build log config");

    log4rs::init_config(config).expect("failed to init logger");
}

fn run_migrations(client: &mut postgres::Client) -> Result<(), String> {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS _migrations (filename TEXT PRIMARY KEY, applied_at TIMESTAMPTZ DEFAULT now())",
        )
        .map_err(|e| e.to_string())?;

    let migrations: &[(&str, &str)] = &[
        ("001_shared.sql",      include_str!("../database/migrations/001_shared.sql")),
        ("002_teachers.sql",    include_str!("../database/migrations/002_teachers.sql")),
        ("003_students.sql",    include_str!("../database/migrations/003_students.sql")),
        ("004_courses.sql",     include_str!("../database/migrations/004_courses.sql")),
        ("005_enrollments.sql", include_str!("../database/migrations/005_enrollments.sql")),
        ("006_payments.sql",    include_str!("../database/migrations/006_payments.sql")),
    ];

    for (name, sql) in migrations {
        let already_applied: bool = client
            .query_one("SELECT EXISTS(SELECT 1 FROM _migrations WHERE filename = $1)", &[name])
            .map_err(|e| e.to_string())?
            .get(0);

        if !already_applied {
            client.batch_execute(sql).map_err(|e| format!("{name}: {e}"))?;
            client
                .execute("INSERT INTO _migrations (filename) VALUES ($1)", &[name])
                .map_err(|e| e.to_string())?;
            log::info!("applied migration: {name}");
        }
    }
    Ok(())
}

fn main() {
    init_logger();
    log::info!("starting babushka");

    let status = Arc::new(Mutex::new(LoadingStatus {
        message:  "Iniciando…".into(),
        progress: 0.0,
        result:   None,
    }));

    let status_bg = Arc::clone(&status);
    std::thread::spawn(move || {
        let set = |msg: &str, prog: f32| {
            let mut s = status_bg.lock().unwrap();
            s.message  = msg.into();
            s.progress = prog;
        };
        let fail = |e: String| {
            status_bg.lock().unwrap().result = Some(Err(e));
        };

        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => { fail(e.to_string()); return; }
        };

        set("Configurando base de datos…", 0.1);
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("babushka")
            .join("data");
        let settings = SettingsBuilder::new()
            .version(VersionReq::parse("=17.5.0").unwrap())
            .data_dir(data_dir)
            .temporary(false)
            .password("babushka")
            .build();
        let mut pg = PostgreSQL::new(settings);
        if let Err(e) = rt.block_on(pg.setup()) { fail(e.to_string()); return; }

        set("Arrancando base de datos…", 0.3);
        if let Err(e) = rt.block_on(pg.start()) { fail(e.to_string()); return; }

        set("Preparando base de datos…", 0.55);
        if !rt.block_on(pg.database_exists("babushka")).unwrap_or(false) {
            if let Err(e) = rt.block_on(pg.create_database("babushka")) { fail(e.to_string()); return; }
        }

        let url = pg.settings().url("babushka");
        log::info!("psql \"{url}\"");

        set("Conectando…", 0.7);
        let mut client = match postgres::Client::connect(&url, postgres::NoTls) {
            Ok(c)  => c,
            Err(e) => { fail(e.to_string()); return; }
        };

        set("Aplicando migraciones…", 0.85);
        if let Err(e) = run_migrations(&mut client) { fail(e); return; }

        set("Verificando actualizaciones…", 0.95);
        let update = updater::check();

        let mut s  = status_bg.lock().unwrap();
        s.progress = 1.0;
        s.result   = Some(Ok(InitResult {
            pg,
            client,
            rt,
            update_available: update.map(|v| UpdateState::Available(v)),
        }));
    });

    eframe::run_native(
        "Babushka",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(AppWrapper::new(status)))
        }),
    )
    .expect("failed to start app");

    log::info!("shutting down babushka");
}
