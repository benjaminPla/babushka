use std::sync::{Arc, Mutex};

use crate::presentation::fmt_dt;
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::student::delete::StudentDeleteUseCase;

use super::{Mode, StudentsState, clear_form, make_repo};

enum Action { Edit, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState) {
    ui.horizontal(|ui| {
        ui.heading("Alumnos");
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

    egui::Grid::new("students_grid")
        .num_columns(6)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Nombre");
            ui.strong("Apellido");
            ui.strong("Email");
            ui.strong("Teléfono");
            ui.strong("Tipo");
            ui.strong("");
            ui.end_row();

            for s in &state.students {
                ui.label(&s.first_name);
                ui.label(&s.last_name);
                ui.label(&s.email);
                ui.label(&s.phone);
                ui.label(s.age_group.label());
                ui.horizontal(|ui| {
                    if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   s.id)); }
                    if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, s.id)); }
                });
                ui.end_row();
            }
        });

    if let Some((act, id)) = action {
        match act {
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
                    state.error      = None;
                    state.mode       = Mode::Edit;
                }
            }
            Action::Delete => {
                match StudentDeleteUseCase::new(make_repo(client)).execute(id) {
                    Ok(_)  => state.needs_reload = true,
                    Err(e) => state.error = Some(e.to_string()),
                }
            }
        }
    }
}
