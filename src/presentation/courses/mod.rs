mod detail;
mod enrollment_detail;
mod form;
mod list;

use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        course::{dto::CourseDto, get_all::CourseGetAllUseCase},
        enrollment::dto::EnrollmentDto,
        payment::dto::PaymentDto,
        teacher::dto::TeacherDto,
        student::dto::StudentDto,
    },
    domain::shared::value_objects::age_group::AgeGroup,
    infrastructure::{
        course::CoursePgRepo, enrollment::EnrollmentPgRepo,
        payment::PaymentPgRepo, teacher::TeacherPgRepo, student::StudentPgRepo,
    },
};

#[derive(Default, PartialEq)]
pub enum Mode {
    #[default] List,
    CreateCourse,
    EditCourse,
    Detail,
    AddEnrollment,
    EnrollmentDetail,
    AddPayment,
}

pub struct CoursesState {
    pub mode:                   Mode,

    // list
    pub courses:                Vec<CourseDto>,
    pub needs_reload:           bool,

    // course form
    pub editing_id:             Option<Uuid>,
    pub name:                   String,
    pub teacher_id:             Option<Uuid>,
    pub teachers:               Vec<TeacherDto>,
    pub age_group:              AgeGroup,
    pub capacity:               String,
    pub price:                  String,
    pub course_notes:           String,

    // course detail
    pub selected_course:        Option<CourseDto>,
    pub enrollments:            Vec<EnrollmentDto>,
    pub needs_reload_enrollments: bool,
    pub available_students:     Vec<StudentDto>,
    pub selected_student_id:    Option<Uuid>,

    // enrollment detail
    pub selected_enrollment:    Option<EnrollmentDto>,
    pub payments:               Vec<PaymentDto>,
    pub needs_reload_payments:  bool,

    // payment form
    pub payment_amount:         String,
    pub payment_due_date:       String,
    pub payment_notes:          String,

    // read-only timestamps
    pub created_at:             String,
    pub updated_at:             String,

    pub error:                  Option<String>,
    pub confirm_delete:         Option<Uuid>,
}

impl Default for CoursesState {
    fn default() -> Self {
        Self {
            mode:                     Mode::List,
            courses:                  Vec::new(),
            needs_reload:             true,
            editing_id:               None,
            name:                     String::new(),
            teacher_id:               None,
            teachers:                 Vec::new(),
            age_group:                AgeGroup::default(),
            capacity:                 String::new(),
            price:                    String::new(),
            course_notes:             String::new(),
            selected_course:          None,
            enrollments:              Vec::new(),
            needs_reload_enrollments: false,
            available_students:       Vec::new(),
            selected_student_id:      None,
            selected_enrollment:      None,
            payments:                 Vec::new(),
            needs_reload_payments:    false,
            payment_amount:           String::new(),
            payment_due_date:         String::new(),
            payment_notes:            String::new(),
            created_at:               String::new(),
            updated_at:               String::new(),
            error:                    None,
            confirm_delete:           None,
        }
    }
}

pub fn make_course_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePgRepo> {
    Arc::new(CoursePgRepo::new(Arc::clone(client)))
}
pub fn make_enrollment_repo(client: &Arc<Mutex<Client>>) -> Arc<EnrollmentPgRepo> {
    Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
}
pub fn make_payment_repo(client: &Arc<Mutex<Client>>) -> Arc<PaymentPgRepo> {
    Arc::new(PaymentPgRepo::new(Arc::clone(client)))
}
pub fn make_teacher_repo(client: &Arc<Mutex<Client>>) -> Arc<TeacherPgRepo> {
    Arc::new(TeacherPgRepo::new(Arc::clone(client)))
}
pub fn make_student_repo(client: &Arc<Mutex<Client>>) -> Arc<StudentPgRepo> {
    Arc::new(StudentPgRepo::new(Arc::clone(client)))
}

pub fn format_price(cents: i32) -> String { format!("{:.2}", cents as f64 / 100.0) }

pub fn parse_price(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32)
}

pub fn clear_course_form(state: &mut CoursesState) {
    state.editing_id   = None;
    state.name         = String::new();
    state.teacher_id   = None;
    state.age_group    = AgeGroup::default();
    state.capacity     = String::new();
    state.price        = String::new();
    state.course_notes = String::new();
    state.created_at   = String::new();
    state.updated_at   = String::new();
    state.error        = None;
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState) {
    if state.needs_reload {
        match CourseGetAllUseCase::new(make_course_repo(client)).execute() {
            Ok(courses) => { state.courses = courses; state.needs_reload = false; }
            Err(e)      => { state.error = Some(e.to_string()); }
        }
    }

    match state.mode {
        Mode::List                             => list::show(ui, client, state),
        Mode::CreateCourse | Mode::EditCourse  => form::show(ui, client, state),
        Mode::Detail | Mode::AddEnrollment     => detail::show(ui, client, state),
        Mode::EnrollmentDetail | Mode::AddPayment => enrollment_detail::show(ui, client, state),
    }
}
