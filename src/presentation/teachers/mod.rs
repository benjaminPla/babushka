mod form;
mod list;
mod view;

use std::sync::Arc;

use eframe::egui;
use uuid::Uuid;

use crate::{
    application::teacher::{dto::TeacherDto, get_all::TeacherGetAllUseCase},
    domain::teacher::repository::TeacherRepo,
    presentation::{push_error, Notifications},
};

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, View, Create, Edit }

pub struct TeachersState {
    pub mode:              Mode,
    pub teachers:          Vec<TeacherDto>,
    pub needs_reload:      bool,
    pub filter_first_name: String,
    pub filter_last_name:  String,
    pub filter_email:      String,
    pub viewing_id:        Option<Uuid>,
    pub editing_id:        Option<Uuid>,
    pub first_name:        String,
    pub last_name:         String,
    pub email:             String,
    pub phone:             String,
    pub notes:             String,
    pub created_at:        String,
    pub updated_at:        String,
    pub confirm_delete:    Option<Uuid>,
}

impl Default for TeachersState {
    fn default() -> Self {
        Self {
            mode:              Mode::List,
            teachers:          Vec::new(),
            needs_reload:      true,
            filter_first_name: String::new(),
            filter_last_name:  String::new(),
            filter_email:      String::new(),
            viewing_id:        None,
            editing_id:        None,
            first_name:        String::new(),
            last_name:         String::new(),
            email:             String::new(),
            phone:             String::new(),
            notes:             String::new(),
            created_at:        String::new(),
            updated_at:        String::new(),
            confirm_delete:    None,
        }
    }
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
}

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn TeacherRepo>, state: &mut TeachersState, notifs: &mut Notifications) {
    if state.needs_reload {
        state.needs_reload = false;
        match TeacherGetAllUseCase::new(Arc::clone(repo)).execute() {
            Ok(teachers) => { state.teachers = teachers; }
            Err(e)       => push_error(notifs, e.to_string()),
        }
    }

    match state.mode {
        Mode::List                => list::show(ui, repo, state, notifs),
        Mode::View                => view::show(ui, state),
        Mode::Create | Mode::Edit => form::show(ui, repo, state, notifs),
    }
}
