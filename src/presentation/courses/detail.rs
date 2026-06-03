use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;

use crate::presentation::Notifications;

use super::{CoursesState, Mode, format_price};

pub fn show(ui: &mut egui::Ui, _client: &Arc<Mutex<Client>>, state: &mut CoursesState, _notifs: &mut Notifications) {
    let course = match &state.selected_course {
        Some(c) => c.clone(),
        None    => { state.mode = Mode::List; return; }
    };

    ui.horizontal(|ui| {
        if ui.button("← Cursos").clicked() {
            state.mode            = Mode::List;
            state.selected_course = None;
        }
        ui.heading(format!("{} — {}", course.name, course.age_group.label()));
    });
    ui.separator();

    egui::Grid::new("course_detail").num_columns(2).show(ui, |ui| {
        ui.label("Profesor");     ui.label(&course.teacher_name);               ui.end_row();
        ui.label("Capacidad");    ui.label(course.capacity.to_string());         ui.end_row();
        ui.label("Precio mensual"); ui.label(format!("${}", format_price(course.price_cents))); ui.end_row();
        ui.label("Precio clase"); ui.label(format!("${}", format_price(course.class_price_cents))); ui.end_row();
        if let Some(n) = &course.notes {
            ui.label("Notas"); ui.label(n); ui.end_row();
        }
    });

    ui.add_space(12.0);
    ui.label("Períodos: próximamente");
}
