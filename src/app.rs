use eframe::egui;
use sqlx::PgPool;

use crate::views::{students, teachers};

#[derive(PartialEq)]
enum View {
    Teachers,
    Students,
}

pub struct App {
    pool: PgPool,
    current_view: View,
}

impl App {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            current_view: View::Teachers,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("menu").show_inside(ui, |ui| {
            ui.heading("Aries");
            ui.separator();
            ui.selectable_value(&mut self.current_view, View::Teachers, "Profesores");
            ui.selectable_value(&mut self.current_view, View::Students, "Alumnos");
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.current_view {
            View::Teachers => teachers::show(ui, &self.pool),
            View::Students => students::show(ui, &self.pool),
        });
    }
}
