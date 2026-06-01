use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;

use crate::{
    application::student::{
        create::{StudentCreateInput, StudentCreateUseCase},
        update::{StudentUpdateInput, StudentUpdateUseCase},
    },
    domain::student::AgeGroup,
};

use super::{Mode, StudentsState, clear_form, make_repo};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState) {
    let title = if state.mode == Mode::Create { "Nuevo Alumno" } else { "Editar Alumno" };

    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            state.mode = Mode::List;
            clear_form(state);
        }
        ui.heading(title);
    });
    ui.separator();

    egui::Grid::new("student_form").num_columns(2).show(ui, |ui| {
        ui.label("Nombre");   ui.text_edit_singleline(&mut state.first_name); ui.end_row();
        ui.label("Apellido"); ui.text_edit_singleline(&mut state.last_name);  ui.end_row();
        ui.label("Email");    ui.text_edit_singleline(&mut state.email);      ui.end_row();
        ui.label("Teléfono"); ui.text_edit_singleline(&mut state.phone);      ui.end_row();

        ui.label("Tipo");
        ui.horizontal(|ui| {
            ui.radio_value(&mut state.age_group, AgeGroup::Adult, AgeGroup::Adult.label());
            ui.radio_value(&mut state.age_group, AgeGroup::Minor, AgeGroup::Minor.label());
        });
        ui.end_row();

        ui.label("Notas");
        ui.text_edit_multiline(&mut state.notes);
        ui.end_row();
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
    }

    if ui.button("Guardar").clicked() {
        let notes = if state.notes.trim().is_empty() { None } else { Some(state.notes.clone()) };

        let result = match state.mode {
            Mode::Create => StudentCreateUseCase::new(make_repo(client)).execute(StudentCreateInput {
                age_group:  state.age_group.clone(),
                email:      state.email.clone(),
                first_name: state.first_name.clone(),
                last_name:  state.last_name.clone(),
                notes,
                phone:      state.phone.clone(),
            }),
            Mode::Edit => StudentUpdateUseCase::new(make_repo(client)).execute(StudentUpdateInput {
                id:         state.editing_id.unwrap(),
                age_group:  state.age_group.clone(),
                email:      state.email.clone(),
                first_name: state.first_name.clone(),
                last_name:  state.last_name.clone(),
                notes,
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
