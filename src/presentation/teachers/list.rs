use std::sync::{Arc, Mutex};

use crate::presentation::fmt_dt;
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::teacher::delete::TeacherDeleteUseCase;

use super::{Mode, TeachersState, clear_form, make_repo};

enum Action { Edit, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut TeachersState) {
    ui.horizontal(|ui| {
        ui.heading("Profesores");
        if ui.button("+ Nuevo").clicked() {
            clear_form(state);
            state.mode = Mode::Create;
        }
    });
    ui.separator();

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
        ui.separator();
    }

    let mut action: Option<(Action, Uuid)> = None;

    egui::Grid::new("teachers_grid")
        .num_columns(5)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Nombre");
            ui.strong("Apellido");
            ui.strong("Email");
            ui.strong("Teléfono");
            ui.strong("");
            ui.end_row();

            for t in &state.teachers {
                ui.label(&t.first_name);
                ui.label(&t.last_name);
                ui.label(&t.email);
                ui.label(&t.phone);
                ui.horizontal(|ui| {
                    if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   t.id)); }
                    if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, t.id)); }
                });
                ui.end_row();
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
                    state.error      = None;
                    state.mode       = Mode::Edit;
                }
            }
            Action::Delete => {
                match TeacherDeleteUseCase::new(make_repo(client)).execute(id) {
                    Ok(_)  => state.needs_reload = true,
                    Err(e) => state.error = Some(e.to_string()),
                }
            }
        }
    }
}
