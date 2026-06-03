use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::teacher::delete::TeacherDeleteUseCase;
use crate::presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications};
use crate::presentation::table::{self, Column};

use super::{Mode, TeachersState, clear_form, make_repo};

enum Action { Edit, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut TeachersState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Profesores");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Nuevo").clicked() {
                clear_form(state);
                state.mode = Mode::Create;
            }
        });
    });
    ui.separator();

    let mut action: Option<(Action, Uuid)> = None;

    table::builder(ui)
        .column(Column::auto().at_least(90.0))
        .column(Column::auto().at_least(90.0))
        .column(Column::remainder().at_least(120.0))
        .column(Column::auto().at_least(110.0))
        .column(Column::auto())
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Nombre"));
            h.col(|ui| table::head(ui, "Apellido"));
            h.col(|ui| table::head(ui, "Email"));
            h.col(|ui| table::head(ui, "Teléfono"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for t in &state.teachers {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&t.first_name); });
                    row.col(|ui| { ui.label(&t.last_name); });
                    row.col(|ui| { ui.label(&t.email); });
                    row.col(|ui| { ui.label(&t.phone); });
                    row.col(|ui| {
                        if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   t.id)); }
                        if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, t.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Edit => {
                if let Some(t) = state.teachers.iter().find(|t| t.id == id) {
                    state.first_name = t.first_name.clone();
                    state.last_name  = t.last_name.clone();
                    state.email      = t.email.clone();
                    state.phone      = t.phone.clone();
                    state.notes      = t.notes.clone().unwrap_or_default();
                    state.created_at = fmt_dt(t.created_at);
                    state.updated_at = fmt_dt(t.updated_at);
                    state.editing_id = Some(id);
                    state.mode       = Mode::Edit;
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match TeacherDeleteUseCase::new(make_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Profesor eliminado"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
