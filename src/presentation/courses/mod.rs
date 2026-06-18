mod detail;
mod list;
mod form;

use std::sync::{Arc, Mutex};

use chrono::Datelike;
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::course::dto::CourseDto;
use crate::application::course::get_all::CourseGetAllUseCase;
use crate::application::course_period::dto::CoursePeriodDto;
use crate::application::teacher::dto::TeacherDto;
use crate::domain::course::repository::CourseRepo;
use crate::domain::shared::value_objects::age_group::AgeGroup;
use crate::infrastructure::course_period::CoursePeriodPgRepo;
use crate::infrastructure::teacher::TeacherPgRepo;
use crate::presentation::push_error;
use crate::presentation::Notifications;

#[derive(Default, PartialEq)]
pub enum Mode {
    #[default] List,
    Detail,
}

pub struct CoursesState {
    pub mode:         Mode,

    // list
    pub courses:      Vec<CourseDto>,
    pub needs_reload: bool,
    pub filter_name:  String,

    // create/edit modal
    pub show_modal:   bool,
    pub editing_id:   Option<Uuid>,
    pub name:         String,
    pub teacher_id:   Option<Uuid>,
    pub teachers:     Vec<TeacherDto>,
    pub age_group:    AgeGroup,
    pub capacity:               String,
    pub month_price_cash:       String,
    pub month_price_transfer:   String,
    pub class_price_cash:       String,
    pub class_price_transfer:   String,
    pub course_notes: String,

    // detail
    pub selected_course:      Option<CourseDto>,
    pub periods:              Vec<CoursePeriodDto>,
    pub needs_reload_periods: bool,

    // period modal
    pub period_year:      i32,
    pub period_month:     u32,
    pub show_period_form: bool,

    // timestamps (edit only)
    pub created_at: String,
    pub updated_at: String,

    pub confirm_delete:        Option<Uuid>,
    pub confirm_delete_period: Option<Uuid>,
}

impl Default for CoursesState {
    fn default() -> Self {
        let now = chrono::Local::now();
        Self {
            mode:         Mode::List,
            courses:      Vec::new(),
            needs_reload: true,
            filter_name:  String::new(),
            show_modal:   false,
            editing_id:   None,
            name:         String::new(),
            teacher_id:   None,
            teachers:     Vec::new(),
            age_group:    AgeGroup::Adult,
            capacity:             String::new(),
            month_price_cash:     String::new(),
            month_price_transfer: String::new(),
            class_price_cash:     String::new(),
            class_price_transfer: String::new(),
            course_notes: String::new(),
            selected_course:      None,
            periods:              Vec::new(),
            needs_reload_periods: false,
            period_year:      now.year(),
            period_month:     now.month(),
            show_period_form: false,
            created_at: String::new(),
            updated_at: String::new(),
            confirm_delete:        None,
            confirm_delete_period: None,
        }
    }
}

pub fn make_course_period_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePeriodPgRepo> {
    Arc::new(CoursePeriodPgRepo::new(Arc::clone(client)))
}
pub fn make_teacher_repo(client: &Arc<Mutex<Client>>) -> Arc<TeacherPgRepo> {
    Arc::new(TeacherPgRepo::new(Arc::clone(client)))
}

pub fn format_price(cents: i32) -> String { crate::presentation::fmt_ars(cents) }

pub fn parse_price(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32)
}

pub fn clear_course_form(state: &mut CoursesState) {
    state.show_modal   = false;
    state.editing_id   = None;
    state.name         = String::new();
    state.teacher_id   = None;
    state.age_group    = AgeGroup::Adult;
    state.capacity             = String::new();
    state.month_price_cash     = String::new();
    state.month_price_transfer = String::new();
    state.class_price_cash     = String::new();
    state.class_price_transfer = String::new();
    state.course_notes = String::new();
    state.created_at   = String::new();
    state.updated_at   = String::new();
}

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn CourseRepo>, client: &Arc<Mutex<Client>>, state: &mut CoursesState, notifs: &mut Notifications) {
    if state.needs_reload {
        state.needs_reload = false;
        match CourseGetAllUseCase::new(Arc::clone(repo)).execute() {
            Ok(courses) => { state.courses = courses; }
            Err(e)      => push_error(notifs, e.to_string()),
        }
    }

    match state.mode {
        Mode::List   => list::show(ui, repo, client, state, notifs),
        Mode::Detail => detail::show(ui, client, state, notifs),
    }
    form::show(ui.ctx(), repo, state, notifs);
}
