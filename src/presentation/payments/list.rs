use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        enrollment::get_all::EnrollmentGetAllUseCase,
        payment::{
            create::{PaymentCreateInput, PaymentCreateUseCase},
            delete::PaymentDeleteUseCase,
            mark_paid::PaymentMarkPaidUseCase,
        },
    },
    domain::payment::PaymentStatus,
    presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications},
    presentation::table::{self, Column},
};

use super::{Mode, PaymentsState, make_enrollment_repo, make_payment_repo};

enum Action { MarkPaid, Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut PaymentsState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Pagos");
        if state.mode == Mode::List && ui.button("+ Registrar").clicked() {
            match EnrollmentGetAllUseCase::new(make_enrollment_repo(client)).execute() {
                Ok(e)  => state.enrollments = e,
                Err(e) => push_error(notifs, e.to_string()),
            }
            state.sel_enrollment = None;
            state.amount         = String::new();
            state.due_date       = String::new();
            state.notes          = String::new();
            state.mode           = Mode::Create;
        }
    });
    ui.separator();

    if state.mode == Mode::Create {
        egui::Grid::new("payment_create").num_columns(2).show(ui, |ui| {
            ui.label("Inscripción");
            egui::ComboBox::from_id_salt("pay_enrollment")
                .selected_text(
                    state.sel_enrollment
                        .and_then(|id| state.enrollments.iter().find(|e| e.id == id))
                        .map(|e| format!("{} — {}", e.student_name, e.course_name))
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    for e in &state.enrollments {
                        let label = format!("{} — {}", e.student_name, e.course_name);
                        ui.selectable_value(&mut state.sel_enrollment, Some(e.id), label);
                    }
                });
            ui.end_row();
            ui.label("Período (YYYY-MM)"); ui.text_edit_singleline(&mut state.due_date);   ui.end_row();
            ui.label("Monto");             ui.text_edit_singleline(&mut state.amount);      ui.end_row();
            ui.label("Notas");             ui.text_edit_singleline(&mut state.notes);       ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.button("Guardar").clicked() {
                let enrollment_id = match state.sel_enrollment {
                    Some(id) => id,
                    None     => { push_error(notifs, "Seleccionar una inscripción"); return; }
                };
                let due_date = match parse_due_date(&state.due_date) {
                    Some(d) => d,
                    None    => { push_error(notifs, "Período inválido (formato: YYYY-MM)"); return; }
                };
                let amount_cents = match parse_cents(&state.amount) {
                    Some(c) => c,
                    None    => { push_error(notifs, "Monto inválido"); return; }
                };
                let notes = if state.notes.trim().is_empty() { None } else { Some(state.notes.clone()) };
                match PaymentCreateUseCase::new(make_payment_repo(client)).execute(PaymentCreateInput {
                    enrollment_id, amount_cents, due_date, notes,
                }) {
                    Ok(_)  => { push_success(notifs, "Pago registrado"); state.needs_reload = true; state.mode = Mode::List; }
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            if ui.button("Cancelar").clicked() { state.mode = Mode::List; }
        });
        ui.separator();
    }

    let mut action: Option<(Action, Uuid)> = None;

    table::builder(ui)
        .column(Column::auto().at_least(100.0))
        .column(Column::remainder().at_least(100.0))
        .column(Column::exact(70.0))
        .column(Column::exact(75.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(110.0))
        .column(Column::auto())
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Alumno"));
            h.col(|ui| table::head(ui, "Curso"));
            h.col(|ui| table::head(ui, "Período"));
            h.col(|ui| table::head(ui, "Monto"));
            h.col(|ui| table::head(ui, "Estado"));
            h.col(|ui| table::head(ui, "Pagado"));
            h.col(|ui| table::head(ui, ""));
        })
        .body(|mut body| {
            for p in &state.payments {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&p.student_name); });
                    row.col(|ui| { ui.label(&p.course_name); });
                    row.col(|ui| { ui.label(p.due_date.format("%b %Y").to_string()); });
                    row.col(|ui| { ui.label(format!("${:.2}", p.amount_cents as f64 / 100.0)); });
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
                            if ui.small_button("✓ Pagar").clicked() { action = Some((Action::MarkPaid, p.id)); }
                        }
                        if ui.small_button("🗑").clicked() { action = Some((Action::Delete, p.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::MarkPaid => {
                match PaymentMarkPaidUseCase::new(make_payment_repo(client)).execute(id) {
                    Ok(_)  => { state.needs_reload = true; push_success(notifs, "Pago marcado como pagado"); }
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match PaymentDeleteUseCase::new(make_payment_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Pago eliminado"); }
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

fn parse_cents(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32).filter(|&c| c > 0)
}
