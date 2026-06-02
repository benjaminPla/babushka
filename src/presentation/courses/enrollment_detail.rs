use std::sync::{Arc, Mutex};

use chrono::NaiveDate;

use crate::presentation::fmt_dt;
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

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState) {
    let enrollment = match &state.selected_enrollment {
        Some(e) => e.clone(),
        None    => { state.mode = Mode::Detail; return; }
    };
    let course_name = state.selected_course.as_ref().map(|c| c.name.clone()).unwrap_or_default();

    if state.needs_reload_payments {
        match PaymentGetByEnrollmentUseCase::new(make_payment_repo(client)).execute(enrollment.id) {
            Ok(payments) => { state.payments = payments; state.needs_reload_payments = false; }
            Err(e)       => { state.error = Some(e.to_string()); }
        }
    }

    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            state.mode                = Mode::Detail;
            state.selected_enrollment = None;
            state.payments            = Vec::new();
            state.error               = None;
        }
        ui.heading(format!("{} — {}", enrollment.student_name, course_name));
        ui.label(format!("Estado: {}", enrollment.status.label()));
    });
    ui.separator();

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
        ui.separator();
    }

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
                                state.needs_reload_payments = true;
                                state.mode                 = Mode::EnrollmentDetail;
                                state.payment_due_date     = String::new();
                                state.payment_notes        = String::new();
                                state.error                = None;
                            }
                            Err(e) => state.error = Some(e.to_string()),
                        }
                    }
                    _ => state.error = Some("período o monto inválido (formato: YYYY-MM)".into()),
                }
            }
            if ui.button("Cancelar").clicked() {
                state.mode = Mode::EnrollmentDetail;
                state.error = None;
            }
        });
        ui.separator();
    } else if ui.button("+ Registrar pago").clicked() {
        state.mode = Mode::AddPayment;
        state.error = None;
    }

    // Payment history
    let mut mark_id: Option<Uuid> = None;

    egui::Grid::new("payments_grid")
        .num_columns(5)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Período");
            ui.strong("Monto");
            ui.strong("Estado");
            ui.strong("Fecha de pago");
            ui.strong("");
            ui.end_row();

            for p in &state.payments {
                ui.label(p.due_date.format("%b %Y").to_string());
                ui.label(format!("${}", super::format_price(p.amount_cents)));

                let (text, color) = match p.status {
                    PaymentStatus::Paid    => ("✓ Pagado",    egui::Color32::GREEN),
                    PaymentStatus::Overdue => ("✗ Vencido",   egui::Color32::RED),
                    PaymentStatus::Pending => ("⚠ Pendiente", egui::Color32::YELLOW),
                };
                ui.colored_label(color, text);

                match p.paid_at {
                    Some(dt) => ui.label(fmt_dt(dt)),
                    None     => ui.label("—"),
                };

                if p.status != PaymentStatus::Paid {
                    if ui.small_button("Marcar pagado").clicked() {
                        mark_id = Some(p.id);
                    }
                } else {
                    ui.label("");
                }
                ui.end_row();
            }
        });

    if let Some(id) = mark_id {
        match PaymentMarkPaidUseCase::new(make_payment_repo(client)).execute(id) {
            Ok(_)  => state.needs_reload_payments = true,
            Err(e) => state.error = Some(e.to_string()),
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
