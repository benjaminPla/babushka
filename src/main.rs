mod app;
mod application;
mod domain;
mod infrastructure;
mod presentation;

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
use postgresql_embedded::PostgreSQL;

use app::{AppWrapper, InitResult, LoadingStatus};

fn init_logger() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("aries")
        .join("logs");

    std::fs::create_dir_all(&log_dir).expect("failed to create log directory");

    let log_file    = log_dir.join("aries.log");
    let archive_pat = log_dir.join("aries.{}.log").to_string_lossy().into_owned();

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

// NOTE: only dev; prod must use a proper migration tool
fn run_migrations(client: &mut postgres::Client) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir("database/migrations")
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let sql  = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        client.batch_execute(&sql).map_err(|e| format!("{path:?}: {e}"))?;
    }
    Ok(())
}

fn main() {
    init_logger();
    log::info!("starting aries");

    let status = Arc::new(Mutex::new(LoadingStatus {
        message:  "Initializing…".into(),
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

        set("Setting up database…", 0.1);
        let mut pg = PostgreSQL::default();
        if let Err(e) = rt.block_on(pg.setup()) { fail(e.to_string()); return; }

        set("Starting database…", 0.3);
        if let Err(e) = rt.block_on(pg.start()) { fail(e.to_string()); return; }

        set("Preparing database…", 0.55);
        if !rt.block_on(pg.database_exists("aries")).unwrap_or(false) {
            if let Err(e) = rt.block_on(pg.create_database("aries")) { fail(e.to_string()); return; }
        }

        let url = pg.settings().url("aries");

        set("Connecting…", 0.7);
        let mut client = match postgres::Client::connect(&url, postgres::NoTls) {
            Ok(c)  => c,
            Err(e) => { fail(e.to_string()); return; }
        };

        set("Running migrations…", 0.85);
        if let Err(e) = run_migrations(&mut client) { fail(e); return; }

        let mut s  = status_bg.lock().unwrap();
        s.progress = 1.0;
        s.result   = Some(Ok(InitResult { pg, client, rt }));
    });

    eframe::run_native(
        "Aries",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(AppWrapper::new(status)))),
    )
    .expect("failed to start app");

    log::info!("shutting down aries");
}
