use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;

use crate::application::teacher::{
    create::{TeacherCreateInput, TeacherCreateUseCase},
    update::{TeacherUpdateInput, TeacherUpdateUseCase},
};

use super::{Mode, TeachersState, clear_form, make_repo};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut TeachersState) {
    let title = if state.mode == Mode::Create { "Nuevo Profesor" } else { "Editar Profesor" };

    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            state.mode = Mode::List;
            clear_form(state);
        }
        ui.heading(title);
    });
    ui.separator();

    egui::Grid::new("teacher_form").num_columns(2).show(ui, |ui| {
        ui.label("Nombre");   ui.text_edit_singleline(&mut state.first_name); ui.end_row();
        ui.label("Apellido"); ui.text_edit_singleline(&mut state.last_name);  ui.end_row();
        ui.label("Email");    ui.text_edit_singleline(&mut state.email);      ui.end_row();
        ui.label("Teléfono"); ui.text_edit_singleline(&mut state.phone);      ui.end_row();
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
    }

    if ui.button("Guardar").clicked() {
        let result = match state.mode {
            Mode::Create => TeacherCreateUseCase::new(make_repo(client)).execute(TeacherCreateInput {
                email:      state.email.clone(),
                first_name: state.first_name.clone(),
                last_name:  state.last_name.clone(),
                phone:      state.phone.clone(),
            }),
            Mode::Edit => TeacherUpdateUseCase::new(make_repo(client)).execute(TeacherUpdateInput {
                id:         state.editing_id.unwrap(),
                email:      state.email.clone(),
                first_name: state.first_name.clone(),
                last_name:  state.last_name.clone(),
                phone:      state.phone.clone(),
            }),
            Mode::List => unreachable!(),
        };

        match result {
            Ok(_)  => { state.needs_reload = true; state.mode = Mode::List; clear_form(state); }
            Err(e) => { state.error = Some(e.to_string()); }
        }
    }
}
