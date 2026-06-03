use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::course::delete::CourseDeleteUseCase;
use crate::presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications};
use crate::presentation::table::{self, Column};

use super::{CoursesState, Mode, clear_course_form, format_price, make_course_repo};

enum Action { Open, Edit, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Cursos");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("+ Nuevo").clicked() {
            clear_course_form(state);
            if let Ok(ts) = crate::application::teacher::get_all::TeacherGetAllUseCase::new(
                super::make_teacher_repo(client)
            ).execute() {
                state.teachers = ts;
            }
            state.mode = Mode::CreateCourse;
        }
        });
    });
    ui.separator();

    let mut action: Option<(Action, Uuid)> = None;

    let f = state.list_filter.to_lowercase();
    let visible: Vec<_> = state.courses.iter()
        .filter(|c| f.is_empty() || c.name.to_lowercase().contains(&f))
        .cloned()
        .collect();

    table::builder(ui)
        .column(Column::remainder().at_least(120.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(60.0))
        .column(Column::exact(60.0))
        .column(Column::exact(70.0))
        .column(Column::exact(80.0))
        .column(Column::auto())
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head_filter(ui, "Nombre", &mut state.list_filter));
            h.col(|ui| table::head(ui, "Profesor"));
            h.col(|ui| table::head(ui, "Grupo"));
            h.col(|ui| table::head(ui, "Cap."));
            h.col(|ui| table::head(ui, "Mensual"));
            h.col(|ui| table::head(ui, "Por clase"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for c in &visible {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&c.name); });
                    row.col(|ui| { ui.label(&c.teacher_name); });
                    row.col(|ui| { ui.label(c.age_group.label()); });
                    row.col(|ui| { ui.label(c.capacity.to_string()); });
                    row.col(|ui| { ui.label(format_price(c.price_cents)); });
                    row.col(|ui| { ui.label(format_price(c.class_price_cents)); });
                    row.col(|ui| {
                        if ui.small_button("Ver").clicked()      { action = Some((Action::Open,   c.id)); }
                        if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   c.id)); }
                        if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, c.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Open => {
                if let Some(c) = state.courses.iter().find(|c| c.id == id) {
                    state.selected_course      = Some(c.clone());
                    state.needs_reload_periods = true;
                    state.mode                 = Mode::Detail;
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
                    state.class_price  = format_price(c.class_price_cents);
                    state.course_notes = c.notes.clone().unwrap_or_default();
                    state.created_at   = fmt_dt(c.created_at);
                    state.updated_at   = fmt_dt(c.updated_at);
                    if let Ok(ts) = crate::application::teacher::get_all::TeacherGetAllUseCase::new(
                        super::make_teacher_repo(client)
                    ).execute() {
                        state.teachers = ts;
                    }
                    state.mode = Mode::EditCourse;
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match CourseDeleteUseCase::new(make_course_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Curso eliminado"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
