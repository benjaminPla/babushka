mod list;

use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::enrollment::{dto::EnrollmentDto, get_all::EnrollmentGetAllUseCase},
    domain::enrollment::EffectiveStatus,
    infrastructure::enrollment::EnrollmentPgRepo,
    presentation::{push_error, Notifications},
};

pub struct EnrollmentsState {
    pub enrollments:    Vec<EnrollmentDto>,
    pub needs_reload:   bool,
    pub confirm_delete: Option<Uuid>,
}

impl Default for EnrollmentsState {
    fn default() -> Self {
        Self {
            enrollments:    Vec::new(),
            needs_reload:   true,
            confirm_delete: None,
        }
    }
}

pub fn make_enrollment_repo(client: &Arc<Mutex<Client>>) -> Arc<EnrollmentPgRepo> {
    Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
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

pub fn status_color(s: &EffectiveStatus) -> egui::Color32 {
    match s {
        EffectiveStatus::Active    => crate::theme::colors::SUCCESS,
        EffectiveStatus::Dropped   => crate::theme::colors::ERROR,
        EffectiveStatus::Completed => crate::theme::colors::TEXT_MUTED,
    }
}
