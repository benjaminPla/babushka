use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::teacher::{
        create::{TeacherCreateInput, TeacherCreateUseCase},
        delete::TeacherDeleteUseCase,
        dto::TeacherDto,
        get_all::TeacherGetAllUseCase,
        update::{TeacherUpdateInput, TeacherUpdateUseCase},
    },
    infrastructure::teacher::TeacherPgRepo,
};

#[derive(Default, PartialEq)]
enum Mode { #[default] List, Create, Edit }

pub struct TeachersState {
    mode:         Mode,
    teachers:     Vec<TeacherDto>,
    needs_reload: bool,
    editing_id:   Option<Uuid>,
    first_name:   String,
    last_name:    String,
    email:        String,
    phone:        String,
    error:        Option<String>,
}

impl Default for TeachersState {
    fn default() -> Self {
        Self {
            mode:         Mode::List,
            teachers:     Vec::new(),
            needs_reload: true,
            editing_id:   None,
            first_name:   String::new(),
            last_name:    String::new(),
            email:        String::new(),
            phone:        String::new(),
            error:        None,
        }
    }
}

fn make_repo(client: &Arc<Mutex<Client>>) -> Arc<TeacherPgRepo> {
    Arc::new(TeacherPgRepo::new(Arc::clone(client)))
}

fn clear_form(state: &mut TeachersState) {
    state.editing_id = None;
    state.first_name = String::new();
    state.last_name  = String::new();
    state.email      = String::new();
    state.phone      = String::new();
    state.error      = None;
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut TeachersState) {
    if state.needs_reload {
        match TeacherGetAllUseCase::new(make_repo(client)).execute() {
            Ok(teachers) => { state.teachers = teachers; state.needs_reload = false; }
            Err(e)       => { state.error = Some(e.to_string()); }
        }
    }

    match state.mode {
        Mode::List          => show_list(ui, client, state),
        Mode::Create | Mode::Edit => show_form(ui, client, state),
    }
}

fn show_list(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut TeachersState) {
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

fn show_form(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut TeachersState) {
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

enum Action { Edit, Delete }
