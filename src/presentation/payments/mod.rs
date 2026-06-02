mod list;

use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        enrollment::dto::EnrollmentDto,
        payment::{dto::PaymentDto, get_all::PaymentGetAllUseCase},
    },
    infrastructure::{enrollment::EnrollmentPgRepo, payment::PaymentPgRepo},
    presentation::{push_error, Notifications},
};

#[derive(Default, PartialEq)]
pub enum Mode { #[default] List, Create }

pub struct PaymentsState {
    pub mode:               Mode,
    pub payments:           Vec<PaymentDto>,
    pub needs_reload:       bool,
    pub confirm_delete:     Option<Uuid>,
    // create form
    pub enrollments:        Vec<EnrollmentDto>,
    pub sel_enrollment:     Option<Uuid>,
    pub amount:             String,
    pub due_date:           String,
    pub notes:              String,
}

impl Default for PaymentsState {
    fn default() -> Self {
        Self {
            mode:           Mode::List,
            payments:       Vec::new(),
            needs_reload:   true,
            confirm_delete: None,
            enrollments:    Vec::new(),
            sel_enrollment: None,
            amount:         String::new(),
            due_date:       String::new(),
            notes:          String::new(),
        }
    }
}

pub fn make_payment_repo(client: &Arc<Mutex<Client>>) -> Arc<PaymentPgRepo> {
    Arc::new(PaymentPgRepo::new(Arc::clone(client)))
}
pub fn make_enrollment_repo(client: &Arc<Mutex<Client>>) -> Arc<EnrollmentPgRepo> {
    Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut PaymentsState, notifs: &mut Notifications) {
    if state.needs_reload {
        match PaymentGetAllUseCase::new(make_payment_repo(client)).execute() {
            Ok(p)  => { state.payments = p; state.needs_reload = false; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
    list::show(ui, client, state, notifs);
}
