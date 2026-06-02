mod list;

use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        course::dto::CourseDto,
        enrollment::{dto::EnrollmentDto, get_all::EnrollmentGetAllUseCase},
        student::dto::StudentDto,
    },
    domain::enrollment::EnrollmentStatus,
    infrastructure::{
        course::CoursePgRepo, enrollment::EnrollmentPgRepo,
        student::StudentPgRepo,
    },
    presentation::{push_error, Notifications},
};

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, Create }

pub struct EnrollmentsState {
    pub mode:           Mode,
    pub enrollments:    Vec<EnrollmentDto>,
    pub needs_reload:   bool,
    pub confirm_delete: Option<Uuid>,
    // create form
    pub students:       Vec<StudentDto>,
    pub courses:        Vec<CourseDto>,
    pub sel_student:    Option<Uuid>,
    pub sel_course:     Option<Uuid>,
}

impl Default for EnrollmentsState {
    fn default() -> Self {
        Self {
            mode:           Mode::List,
            enrollments:    Vec::new(),
            needs_reload:   true,
            confirm_delete: None,
            students:       Vec::new(),
            courses:        Vec::new(),
            sel_student:    None,
            sel_course:     None,
        }
    }
}

pub fn make_enrollment_repo(client: &Arc<Mutex<Client>>) -> Arc<EnrollmentPgRepo> {
    Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
}
pub fn make_course_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePgRepo> {
    Arc::new(CoursePgRepo::new(Arc::clone(client)))
}
pub fn make_student_repo(client: &Arc<Mutex<Client>>) -> Arc<StudentPgRepo> {
    Arc::new(StudentPgRepo::new(Arc::clone(client)))
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut EnrollmentsState, notifs: &mut Notifications) {
    if state.needs_reload {
        match EnrollmentGetAllUseCase::new(make_enrollment_repo(client)).execute() {
            Ok(e)  => { state.enrollments = e; state.needs_reload = false; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
    list::show(ui, client, state, notifs);
}

pub fn status_label_color(s: &EnrollmentStatus) -> egui::Color32 {
    match s {
        EnrollmentStatus::Active    => egui::Color32::GREEN,
        EnrollmentStatus::Dropped   => egui::Color32::RED,
        EnrollmentStatus::Completed => egui::Color32::GRAY,
    }
}
