use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::student::delete::StudentDeleteUseCase;
use crate::presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications};
use crate::presentation::table::{self, Column};

use super::{Mode, StudentsState, clear_form, make_repo};

enum Action { Open, Edit, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Alumnos");
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
        .column(Column::auto().at_least(60.0))
        .column(Column::auto())
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Nombre"));
            h.col(|ui| table::head(ui, "Apellido"));
            h.col(|ui| table::head(ui, "Email"));
            h.col(|ui| table::head(ui, "Teléfono"));
            h.col(|ui| table::head(ui, "Tipo"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for s in &state.students {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&s.first_name); });
                    row.col(|ui| { ui.label(&s.last_name); });
                    row.col(|ui| { ui.label(&s.email); });
                    row.col(|ui| { ui.label(&s.phone); });
                    row.col(|ui| { ui.label(s.age_group.label()); });
                    row.col(|ui| {
                        if ui.small_button("Ver").clicked()      { action = Some((Action::Open,   s.id)); }
                        if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   s.id)); }
                        if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, s.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Open => {
                if let Some(s) = state.students.iter().find(|s| s.id == id).cloned() {
                    super::clear_detail_state(state);
                    state.selected_student    = Some(s);
                    state.needs_reload_ledger = true;
                    state.mode               = super::Mode::Detail;
                }
            }
            Action::Edit => {
                if let Some(s) = state.students.iter().find(|s| s.id == id) {
                    state.age_group  = s.age_group.clone();
                    state.first_name = s.first_name.clone();
                    state.last_name  = s.last_name.clone();
                    state.email      = s.email.clone();
                    state.phone      = s.phone.clone();
                    state.notes      = s.notes.clone().unwrap_or_default();
                    state.created_at = fmt_dt(s.created_at);
                    state.updated_at = fmt_dt(s.updated_at);
                    state.editing_id = Some(id);
                    state.mode       = Mode::Edit;
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match StudentDeleteUseCase::new(make_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Alumno eliminado"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
