use std::sync::{Arc, Mutex};

use crate::presentation::fmt_dt;
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::course::delete::CourseDeleteUseCase;

use super::{CoursesState, Mode, clear_course_form, format_price, make_course_repo};

enum Action { Open, Edit, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState) {
    ui.horizontal(|ui| {
        ui.heading("Cursos");
        if ui.button("+ Nuevo").clicked() {
            clear_course_form(state);
            // preload teachers for the form
            if let Ok(ts) = crate::application::teacher::get_all::TeacherGetAllUseCase::new(
                super::make_teacher_repo(client)
            ).execute() {
                state.teachers = ts;
            }
            state.mode = Mode::CreateCourse;
        }
    });
    ui.separator();

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
        ui.separator();
    }

    let mut action: Option<(Action, Uuid)> = None;

    egui::Grid::new("courses_grid")
        .num_columns(7)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Nombre");
            ui.strong("Profesor");
            ui.strong("Grupo");
            ui.strong("Capacidad");
            ui.strong("Precio");
            ui.strong("Inscritos");
            ui.strong("");
            ui.end_row();

            for c in &state.courses {
                ui.label(&c.name);
                ui.label(&c.teacher_name);
                ui.label(c.age_group.label());
                ui.label(format!("{}", c.capacity));
                ui.label(format_price(c.price_cents));
                ui.label(format!("{}/{}", c.enrolled, c.capacity));
                ui.horizontal(|ui| {
                    if ui.small_button("Ver").clicked()      { action = Some((Action::Open,   c.id)); }
                    if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   c.id)); }
                    if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, c.id)); }
                });
                ui.end_row();
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Open => {
                if let Some(c) = state.courses.iter().find(|c| c.id == id) {
                    state.selected_course        = Some(c.clone());
                    state.needs_reload_enrollments = true;
                    state.error                  = None;
                    state.mode                   = Mode::Detail;
                }
            }
            Action::Edit => {
                if let Some(c) = state.courses.iter().find(|c| c.id == id) {
                    state.editing_id   = Some(id);
                    state.name         = c.name.clone();
                    state.teacher_id   = Some(c.teacher_id);
                    state.age_group    = c.age_group.clone();
                    state.capacity     = c.capacity.to_string();
                    state.price        = format_price(c.price_cents);
                    state.course_notes = c.notes.clone().unwrap_or_default();
                    state.created_at   = fmt_dt(c.created_at);
                    state.updated_at   = fmt_dt(c.updated_at);
                    state.error        = None;
                    if let Ok(ts) = crate::application::teacher::get_all::TeacherGetAllUseCase::new(
                        super::make_teacher_repo(client)
                    ).execute() {
                        state.teachers = ts;
                    }
                    state.mode = Mode::EditCourse;
                }
            }
            Action::Delete => {
                match CourseDeleteUseCase::new(make_course_repo(client)).execute(id) {
                    Ok(_)  => state.needs_reload = true,
                    Err(e) => state.error = Some(e.to_string()),
                }
            }
        }
    }
}
