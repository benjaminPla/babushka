use std::sync::{Arc, Mutex};

use chrono::NaiveDate;

use crate::presentation::{fmt_dt, push_error, push_success, Notifications};
use crate::presentation::table::{self, Column};
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::payment::{
        create::{PaymentCreateInput, PaymentCreateUseCase},
        get_by_enrollment::PaymentGetByEnrollmentUseCase,
        mark_paid::PaymentMarkPaidUseCase,
    },
    domain::payment::PaymentStatus,
};

use super::{CoursesState, Mode, make_payment_repo, parse_price};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState, notifs: &mut Notifications) {
    let enrollment = match &state.selected_enrollment {
        Some(e) => e.clone(),
        None    => { state.mode = Mode::Detail; return; }
    };
    let course_name = state.selected_course.as_ref().map(|c| c.name.clone()).unwrap_or_default();

    if state.needs_reload_payments {
        match PaymentGetByEnrollmentUseCase::new(make_payment_repo(client)).execute(enrollment.id) {
            Ok(payments) => { state.payments = payments; state.needs_reload_payments = false; }
            Err(e)       => push_error(notifs, e.to_string()),
        }
    }

    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            state.mode                = Mode::Detail;
            state.selected_enrollment = None;
            state.payments            = Vec::new();
        }
        ui.heading(format!("{} — {}", enrollment.student_name, course_name));
        ui.label(format!("Estado: {}", enrollment.status.label()));
    });
    ui.separator();

    // Add payment form
    if state.mode == Mode::AddPayment {
        ui.heading("Registrar pago");
        egui::Grid::new("payment_form").num_columns(2).show(ui, |ui| {
            ui.label("Período (YYYY-MM)"); ui.text_edit_singleline(&mut state.payment_due_date); ui.end_row();
            ui.label("Monto");            ui.text_edit_singleline(&mut state.payment_amount);   ui.end_row();
            ui.label("Notas");            ui.text_edit_singleline(&mut state.payment_notes);    ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("Guardar").clicked() {
                let due_date = parse_due_date(&state.payment_due_date);
                let amount   = parse_price(&state.payment_amount);

                match (due_date, amount) {
                    (Some(due_date), Some(amount_cents)) => {
                        let notes = if state.payment_notes.trim().is_empty() { None }
                                    else { Some(state.payment_notes.clone()) };
                        match PaymentCreateUseCase::new(make_payment_repo(client)).execute(PaymentCreateInput {
                            enrollment_id: enrollment.id, amount_cents, due_date, notes,
                        }) {
                            Ok(_) => {
                                push_success(notifs, "Pago registrado");
                                state.needs_reload_payments = true;
                                state.mode                  = Mode::EnrollmentDetail;
                                state.payment_due_date      = String::new();
                                state.payment_notes         = String::new();
                            }
                            Err(e) => push_error(notifs, e.to_string()),
                        }
                    }
                    _ => push_error(notifs, "Período o monto inválido (formato: YYYY-MM)"),
                }
            }
            if ui.button("Cancelar").clicked() {
                state.mode = Mode::EnrollmentDetail;
            }
        });
        ui.separator();
    } else if ui.button("+ Registrar pago").clicked() {
        state.mode = Mode::AddPayment;
    }

    // Payment history
    let mut mark_id: Option<Uuid> = None;

    table::builder(ui)
        .column(Column::exact(70.0))
        .column(Column::exact(75.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::remainder().at_least(110.0))
        .column(Column::auto())
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Período"));
            h.col(|ui| table::head(ui, "Monto"));
            h.col(|ui| table::head(ui, "Estado"));
            h.col(|ui| table::head(ui, "Fecha de pago"));
            h.col(|ui| table::head(ui, ""));
        })
        .body(|mut body| {
            for p in &state.payments {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(p.due_date.format("%b %Y").to_string()); });
                    row.col(|ui| { ui.label(format!("${}", super::format_price(p.amount_cents))); });
                    row.col(|ui| {
                        let (text, color) = match p.status {
                            PaymentStatus::Paid    => ("✓ Pagado",    crate::theme::colors::SUCCESS),
                            PaymentStatus::Overdue => ("✗ Vencido",   crate::theme::colors::ERROR),
                            PaymentStatus::Pending => ("⚠ Pendiente", crate::theme::colors::WARNING),
                        };
                        ui.colored_label(color, text);
                    });
                    row.col(|ui| {
                        match p.paid_at {
                            Some(dt) => { ui.label(fmt_dt(dt)); }
                            None     => { ui.label("—"); }
                        }
                    });
                    row.col(|ui| {
                        if p.status != PaymentStatus::Paid {
                            if ui.small_button("Marcar pagado").clicked() { mark_id = Some(p.id); }
                        }
                    });
                });
            }
        });

    if let Some(id) = mark_id {
        match PaymentMarkPaidUseCase::new(make_payment_repo(client)).execute(id) {
            Ok(_)  => { push_success(notifs, "Pago marcado como pagado"); state.needs_reload_payments = true; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

fn parse_due_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.len() != 7 { return None; }
    let year:  i32 = s[..4].parse().ok()?;
    let month: u32 = s[5..].parse().ok()?;
    NaiveDate::from_ymd_opt(year, month, 1)
}
