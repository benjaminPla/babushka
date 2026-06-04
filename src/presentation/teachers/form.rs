use std::sync::Arc;

use eframe::egui;

use crate::application::teacher::{
    create::{TeacherCreateInput, TeacherCreateUseCase},
    update::{TeacherUpdateInput, TeacherUpdateUseCase},
};
use crate::domain::teacher::repository::TeacherRepo;
use crate::presentation::{push_error, push_success, Notifications};

use super::{Mode, TeachersState, clear_form};

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn TeacherRepo>, state: &mut TeachersState, notifs: &mut Notifications) {
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
        ui.label("Notas");    ui.text_edit_multiline(&mut state.notes);       ui.end_row();

        if state.mode == Mode::Edit {
            ui.label("Creado");  ui.label(&state.created_at); ui.end_row();
            ui.label("Editado"); ui.label(&state.updated_at); ui.end_row();
        }
    });

    if ui.button("Guardar").clicked() {
        let notes = if state.notes.trim().is_empty() { None } else { Some(state.notes.clone()) };

        let result = match state.mode {
            Mode::Create => TeacherCreateUseCase::new(Arc::clone(repo)).execute(TeacherCreateInput {
                email:      state.email.clone(),
                first_name: state.first_name.clone(),
                last_name:  state.last_name.clone(),
                notes,
                phone:      state.phone.clone(),
            }),
            Mode::Edit => TeacherUpdateUseCase::new(Arc::clone(repo)).execute(TeacherUpdateInput {
                id:         state.editing_id.unwrap(),
                email:      state.email.clone(),
                first_name: state.first_name.clone(),
                last_name:  state.last_name.clone(),
                notes,
                phone:      state.phone.clone(),
            }),
            Mode::List => unreachable!(),
        };

        match result {
            Ok(_)  => {
                push_success(notifs, "Profesor guardado");
                state.needs_reload = true;
                state.mode = Mode::List;
                clear_form(state);
            }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
