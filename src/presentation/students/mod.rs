mod form;
mod list;

use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::student::{dto::StudentDto, get_all::StudentGetAllUseCase},
    domain::student::AgeGroup,
    infrastructure::student::StudentPgRepo,
};

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, Create, Edit }

pub struct StudentsState {
    pub mode:         Mode,
    pub students:     Vec<StudentDto>,
    pub needs_reload: bool,
    pub editing_id:   Option<Uuid>,
    pub age_group:    AgeGroup,
    pub first_name:   String,
    pub last_name:    String,
    pub email:        String,
    pub phone:        String,
    pub notes:        String,
    pub error:        Option<String>,
}

impl Default for StudentsState {
    fn default() -> Self {
        Self {
            mode:         Mode::List,
            students:     Vec::new(),
            needs_reload: true,
            editing_id:   None,
            age_group:    AgeGroup::default(),
            first_name:   String::new(),
            last_name:    String::new(),
            email:        String::new(),
            phone:        String::new(),
            notes:        String::new(),
            error:        None,
        }
    }
}

pub fn make_repo(client: &Arc<Mutex<Client>>) -> Arc<StudentPgRepo> {
    Arc::new(StudentPgRepo::new(Arc::clone(client)))
}

pub fn clear_form(state: &mut StudentsState) {
    state.editing_id = None;
    state.age_group  = AgeGroup::default();
    state.first_name = String::new();
    state.last_name  = String::new();
    state.email      = String::new();
    state.phone      = String::new();
    state.notes      = String::new();
    state.error      = None;
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState) {
    if state.needs_reload {
        match StudentGetAllUseCase::new(make_repo(client)).execute() {
            Ok(students) => { state.students = students; state.needs_reload = false; }
            Err(e)       => { state.error = Some(e.to_string()); }
        }
    }

    match state.mode {
        Mode::List              => list::show(ui, client, state),
        Mode::Create | Mode::Edit => form::show(ui, client, state),
    }
}
