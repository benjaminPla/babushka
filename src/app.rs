use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;

use crate::presentation::{
    students::{self, StudentsState},
    teachers::{self, TeachersState},
};

#[derive(PartialEq)]
enum View {
    Teachers,
    Students,
}

pub struct App {
    client:         Arc<Mutex<Client>>,
    current_view:   View,
    students_state: StudentsState,
    teachers_state: TeachersState,
}

impl App {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        Self {
            client,
            current_view:   View::Teachers,
            students_state: StudentsState::default(),
            teachers_state: TeachersState::default(),
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
            View::Teachers => teachers::show(ui, &self.client, &mut self.teachers_state),
            View::Students => students::show(ui, &self.client, &mut self.students_state),
        });
    }
}
