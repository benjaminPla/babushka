mod detail;
mod list;
mod form;

use std::sync::{Arc, Mutex};

use chrono::{Datelike, Local, NaiveDate};
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::application::course::dto::CourseDto;
use crate::application::course_period::dto::CoursePeriodDto;
use crate::application::student::dto::StudentDto;
use crate::application::student::get_all::StudentGetAllUseCase;
use crate::domain::enrollment::Enrollment;
use crate::domain::student::repository::StudentRepo;
use crate::domain::student::AgeGroup;
use crate::infrastructure::course::CoursePgRepo;
use crate::infrastructure::course_period::CoursePeriodPgRepo;
use crate::infrastructure::enrollment::EnrollmentPgRepo;
use crate::presentation::push_error;
use crate::presentation::Notifications;

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, Detail }

pub struct StudentsState {
    pub mode:              Mode,
    pub students:          Vec<StudentDto>,
    pub needs_reload:      bool,
    pub filter_first_name: String,
    pub filter_last_name:  String,
    pub filter_email:      String,

    // create/edit modal
    pub show_modal: bool,
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
    pub selected_student:         Option<StudentDto>,
    pub enrollments:              Vec<Enrollment>,
    pub needs_reload_enrollments: bool,

    // enroll modal
    pub show_enroll_form:     bool,
    pub enroll_courses:       Vec<CourseDto>,
    pub enroll_sel_course:    Option<Uuid>,
    pub enroll_course_filter: String,
    pub enroll_periods:       Vec<CoursePeriodDto>,
    pub enroll_sel_period:    Option<Uuid>,
    pub enroll_period_filter: String,
    pub enroll_pricing_type:  String,

    // payment modal (triggered per enrollment)
    pub show_payment_form:   bool,
    pub pay_enrollment_id:   Option<Uuid>,
    pub pay_amount:          String,
    pub pay_method:          String,
    pub pay_date:            NaiveDate,
    pub pay_notes:           String,

    pub confirm_delete:         Option<Uuid>,
    pub confirm_delete_payment: Option<Uuid>,
}

fn today() -> NaiveDate {
    let n = Local::now();
    NaiveDate::from_ymd_opt(n.year(), n.month(), n.day()).unwrap()
}

impl Default for StudentsState {
    fn default() -> Self {
        Self {
            mode:              Mode::List,
            students:          Vec::new(),
            needs_reload:      true,
            filter_first_name: String::new(),
            filter_last_name:  String::new(),
            filter_email:      String::new(),
            show_modal:        false,
            editing_id:        None,
            age_group:         AgeGroup::Adult,
            first_name:        String::new(),
            last_name:         String::new(),
            email:             String::new(),
            phone:             String::new(),
            notes:             String::new(),
            created_at:        String::new(),
            updated_at:        String::new(),
            selected_student:         None,
            enrollments:              Vec::new(),
            needs_reload_enrollments: false,
            show_enroll_form:     false,
            enroll_courses:       Vec::new(),
            enroll_sel_course:    None,
            enroll_course_filter: String::new(),
            enroll_periods:       Vec::new(),
            enroll_sel_period:    None,
            enroll_period_filter: String::new(),
            enroll_pricing_type:  "monthly".into(),
            show_payment_form:   false,
            pay_enrollment_id:   None,
            pay_amount:          String::new(),
            pay_method:          "cash".into(),
            pay_date:            today(),
            pay_notes:           String::new(),
            confirm_delete:         None,
            confirm_delete_payment: None,
        }
    }
}

pub fn make_enrollment_repo(client: &Arc<Mutex<Client>>) -> Arc<EnrollmentPgRepo> {
    Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
}
pub fn make_course_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePgRepo> {
    Arc::new(CoursePgRepo::new(Arc::clone(client)))
}
pub fn make_course_period_repo(client: &Arc<Mutex<Client>>) -> Arc<CoursePeriodPgRepo> {
    Arc::new(CoursePeriodPgRepo::new(Arc::clone(client)))
}

pub fn clear_detail_state(state: &mut StudentsState) {
    state.selected_student         = None;
    state.enrollments              = Vec::new();
    state.needs_reload_enrollments = false;
    state.show_enroll_form         = false;
    state.enroll_courses           = Vec::new();
    state.enroll_sel_course        = None;
    state.enroll_course_filter     = String::new();
    state.enroll_periods           = Vec::new();
    state.enroll_sel_period        = None;
    state.enroll_period_filter     = String::new();
    state.enroll_pricing_type      = "monthly".into();
    state.show_payment_form        = false;
    state.pay_enrollment_id        = None;
    state.pay_amount               = String::new();
    state.pay_method               = "cash".into();
    state.pay_date                 = today();
    state.pay_notes                = String::new();
    state.confirm_delete           = None;
    state.confirm_delete_payment   = None;
}

pub fn clear_form(state: &mut StudentsState) {
    state.show_modal = false;
    state.editing_id = None;
    state.age_group  = AgeGroup::Adult;
    state.first_name = String::new();
    state.last_name  = String::new();
    state.email      = String::new();
    state.phone      = String::new();
    state.notes      = String::new();
    state.created_at = String::new();
    state.updated_at = String::new();
}

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn StudentRepo>, client: &Arc<Mutex<Client>>, state: &mut StudentsState, notifs: &mut Notifications) {
    if state.needs_reload {
        state.needs_reload = false;
        match StudentGetAllUseCase::new(Arc::clone(repo)).execute() {
            Ok(students) => { state.students = students; }
            Err(e)       => push_error(notifs, e.to_string()),
        }
    }

    match state.mode {
        Mode::List   => list::show(ui, repo, state, notifs),
        Mode::Detail => detail::show(ui, client, state, notifs),
    }
    form::show(ui.ctx(), repo, state, notifs);
}
