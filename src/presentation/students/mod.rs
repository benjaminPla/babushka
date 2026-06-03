mod detail;
mod form;
mod list;

use std::sync::{Arc, Mutex};

use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        course::dto::CourseDto,
        course_period::dto::CoursePeriodDto,
        student::{dto::StudentDto, get_all::StudentGetAllUseCase},
        student_ledger::LedgerEntry,
    },
    domain::student::AgeGroup,
    infrastructure::{
        course::CoursePgRepo,
        course_period::CoursePeriodPgRepo,
        enrollment::EnrollmentPgRepo,
        payment::PaymentPgRepo,
        student::StudentPgRepo,
    },
    presentation::{push_error, Notifications},
};

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, Create, Edit, Detail }

pub struct StudentsState {
    pub mode:         Mode,
    pub students:     Vec<StudentDto>,
    pub needs_reload: bool,

    // form
    pub editing_id: Option<Uuid>,
    pub age_group:  AgeGroup,
    pub first_name: String,
    pub last_name:  String,
    pub email:      String,
    pub phone:      String,
    pub notes:      String,
    pub created_at: String,
    pub updated_at: String,

    // detail
    pub selected_student:    Option<StudentDto>,
    pub ledger:              Vec<LedgerEntry>,
    pub balance_cents:       i32,
    pub needs_reload_ledger: bool,

    // enroll form
    pub show_enroll_form:  bool,
    pub enroll_courses:    Vec<CourseDto>,
    pub enroll_sel_course: Option<Uuid>,
    pub enroll_periods:    Vec<CoursePeriodDto>,
    pub enroll_sel_period: Option<Uuid>,

    // payment form
    pub show_payment_form: bool,
    pub payment_amount:    String,
    pub payment_due_date:  NaiveDate,

    pub confirm_delete: Option<Uuid>,
}

fn today() -> NaiveDate {
    let n = Local::now();
    NaiveDate::from_ymd_opt(n.year(), n.month(), n.day()).unwrap()
}

impl Default for StudentsState {
    fn default() -> Self {
        Self {
            mode:                   Mode::List,
            students:               Vec::new(),
            needs_reload:           true,
            editing_id:             None,
            age_group:              AgeGroup::default(),
            first_name:             String::new(),
            last_name:              String::new(),
            email:                  String::new(),
            phone:                  String::new(),
            notes:                  String::new(),
            created_at:             String::new(),
            updated_at:             String::new(),
            selected_student:       None,
            ledger:                 Vec::new(),
            balance_cents:          0,
            needs_reload_ledger:    false,
            show_enroll_form:       false,
            enroll_courses:         Vec::new(),
            enroll_sel_course:      None,
            enroll_periods:         Vec::new(),
            enroll_sel_period:      None,
            show_payment_form: false,
            payment_amount:    String::new(),
            payment_due_date:  today(),
            confirm_delete:         None,
        }
    }
}

pub fn make_repo(client: &Arc<Mutex<Client>>) -> Arc<StudentPgRepo> {
    Arc::new(StudentPgRepo::new(Arc::clone(client)))
}
pub fn make_enrollment_repo(client: &Arc<Mutex<Client>>) -> Arc<EnrollmentPgRepo> {
    Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
}
pub fn make_payment_repo(client: &Arc<Mutex<Client>>) -> Arc<PaymentPgRepo> {
    Arc::new(PaymentPgRepo::new(Arc::clone(client)))
}
pub fn make_course_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePgRepo> {
    Arc::new(CoursePgRepo::new(Arc::clone(client)))
}
pub fn make_course_period_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePeriodPgRepo> {
    Arc::new(CoursePeriodPgRepo::new(Arc::clone(client)))
}

pub fn clear_detail_state(state: &mut StudentsState) {
    state.selected_student       = None;
    state.ledger                 = Vec::new();
    state.balance_cents          = 0;
    state.needs_reload_ledger    = false;
    state.show_enroll_form       = false;
    state.enroll_courses         = Vec::new();
    state.enroll_sel_course      = None;
    state.enroll_periods         = Vec::new();
    state.enroll_sel_period      = None;
    state.show_payment_form = false;
    state.payment_amount    = String::new();
    state.payment_due_date  = today();
    state.confirm_delete         = None;
}

pub fn clear_form(state: &mut StudentsState) {
    state.editing_id = None;
    state.age_group  = AgeGroup::default();
    state.first_name = String::new();
    state.last_name  = String::new();
    state.email      = String::new();
    state.phone      = String::new();
    state.notes      = String::new();
    state.created_at = String::new();
    state.updated_at = String::new();
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState, notifs: &mut Notifications) {
    if state.needs_reload {
        match StudentGetAllUseCase::new(make_repo(client)).execute() {
            Ok(students) => { state.students = students; state.needs_reload = false; }
            Err(e)       => push_error(notifs, e.to_string()),
        }
    }

    match state.mode {
        Mode::List                => list::show(ui, client, state, notifs),
        Mode::Create | Mode::Edit => form::show(ui, client, state, notifs),
        Mode::Detail              => detail::show(ui, client, state, notifs),
    }
}
