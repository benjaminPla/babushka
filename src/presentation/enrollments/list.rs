use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::enrollment::{
        delete::EnrollmentDeleteUseCase,
        set_dropped::{EnrollmentSetDroppedInput, EnrollmentSetDroppedUseCase},
    },
    domain::enrollment::EffectiveStatus,
    presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications},
    presentation::table::{self, Column},
};

use super::{EnrollmentsState, make_enrollment_repo, status_color};

enum Action { SetDropped(bool), Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut EnrollmentsState, notifs: &mut Notifications) {
    ui.horizontal(|ui| { ui.heading("Inscripciones"); });
    ui.separator();

    let mut action: Option<(Action, Uuid)> = None;

    table::builder(ui)
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(70.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(110.0))
        .column(Column::auto())
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Alumno"));
            h.col(|ui| table::head(ui, "Curso"));
            h.col(|ui| table::head(ui, "Período"));
            h.col(|ui| table::head(ui, "Estado"));
            h.col(|ui| table::head(ui, "Último pago"));
            h.col(|ui| table::head(ui, "Inscripto"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for e in &state.enrollments {
                let status = e.effective_status();
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&e.student_name); });
                    row.col(|ui| { ui.label(&e.course_name); });
                    row.col(|ui| { ui.label(&e.period_label); });
                    row.col(|ui| { ui.colored_label(status_color(&status), status.label()); });
                    row.col(|ui| {
                        match e.latest_payment.as_deref() {
                            None            => { ui.label("—"); }
                            Some("paid")    => { ui.colored_label(crate::theme::colors::SUCCESS, "✓ Pagado"); }
                            Some("overdue") => { ui.colored_label(crate::theme::colors::ERROR,   "✗ Vencido"); }
                            _               => { ui.colored_label(crate::theme::colors::WARNING, "⚠ Pendiente"); }
                        }
                    });
                    row.col(|ui| { ui.label(fmt_dt(e.enrolled_at)); });
                    row.col(|ui| {
                        match status {
                            EffectiveStatus::Dropped => {
                                if ui.small_button("Reactivar").clicked() {
                                    action = Some((Action::SetDropped(false), e.id));
                                }
                            }
                            EffectiveStatus::Active => {
                                if ui.small_button("Dar de baja").clicked() {
                                    action = Some((Action::SetDropped(true), e.id));
                                }
                            }
                            EffectiveStatus::Completed => {}
                        }
                        if ui.small_button("🗑").clicked() { action = Some((Action::Delete, e.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::SetDropped(dropped) => {
                match EnrollmentSetDroppedUseCase::new(make_enrollment_repo(client))
                    .execute(EnrollmentSetDroppedInput { id, dropped }) {
                    Ok(_)  => state.needs_reload = true,
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match EnrollmentDeleteUseCase::new(make_enrollment_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Inscripción eliminada"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
