use eframe::egui;
use sqlx::PgPool;

pub fn show(ui: &mut egui::Ui, _pool: &PgPool) {
    ui.heading("Profesores");
}
