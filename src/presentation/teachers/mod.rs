mod form;
mod list;

use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::teacher::{dto::TeacherDto, get_all::TeacherGetAllUseCase},
    infrastructure::teacher::TeacherPgRepo,
};

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, Create, Edit }

pub struct TeachersState {
    pub mode:         Mode,
    pub teachers:     Vec<TeacherDto>,
    pub needs_reload: bool,
    pub editing_id:   Option<Uuid>,
    pub first_name:   String,
    pub last_name:    String,
    pub email:        String,
    pub phone:        String,
    pub notes:        String,
    pub created_at:   String,
    pub updated_at:   String,
    pub error:          Option<String>,
    pub confirm_delete: Option<Uuid>,
}

impl Default for TeachersState {
    fn default() -> Self {
        Self {
            mode:           Mode::List,
            teachers:       Vec::new(),
            needs_reload:   true,
            editing_id:     None,
            first_name:     String::new(),
            last_name:      String::new(),
            email:          String::new(),
            phone:          String::new(),
            notes:          String::new(),
            created_at:     String::new(),
            updated_at:     String::new(),
            error:          None,
            confirm_delete: None,
        }
    }
}

pub fn make_repo(client: &Arc<Mutex<Client>>) -> Arc<TeacherPgRepo> {
    Arc::new(TeacherPgRepo::new(Arc::clone(client)))
}

pub fn clear_form(state: &mut TeachersState) {
    state.editing_id = None;
    state.first_name = String::new();
    state.last_name  = String::new();
    state.email      = String::new();
    state.phone      = String::new();
    state.notes      = String::new();
    state.created_at = String::new();
    state.updated_at = String::new();
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
        Mode::List              => list::show(ui, client, state),
        Mode::Create | Mode::Edit => form::show(ui, client, state),
    }
}
