use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    presentation::fmt_ars,
    application::{
        course::get_all::CourseGetAllUseCase,
        course_period::get_by_course::CoursePeriodGetByCourseUseCase,
        enrollment::{
            create::{EnrollmentCreateInput, EnrollmentCreateUseCase},
            delete::EnrollmentDeleteUseCase,
        },
        payment::{
            create::{PaymentCreateInput, PaymentCreateUseCase},
            delete::PaymentDeleteUseCase,
            get_by_student::PaymentGetByStudentUseCase,
            mark_paid::PaymentMarkPaidUseCase,
        },
        student_ledger::{LedgerKind, StudentLedgerUseCase},
    },
    presentation::{
        confirm_delete_modal, date_selector, fmt_dt, push_error, push_success, section_header,
        Notifications,
    },
    presentation::table::{self, Column},
};

use super::{
    Mode, StudentsState,
    make_course_period_repo, make_course_repo, make_enrollment_repo, make_payment_repo,
};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState, notifs: &mut Notifications) {
    let student = match &state.selected_student {
        Some(s) => s.clone(),
        None    => { state.mode = Mode::List; return; }
    };

    if state.needs_reload_ledger {
        state.needs_reload_ledger = false;
        let ledger_uc = StudentLedgerUseCase::new(make_enrollment_repo(client), make_payment_repo(client));
        match ledger_uc.execute(student.id) {
            Ok((entries, balance)) => {
                state.ledger        = entries;
                state.balance_cents = balance;
            }
            Err(e) => push_error(notifs, e.to_string()),
        }
        match PaymentGetByStudentUseCase::new(make_payment_repo(client)).execute(student.id) {
            Ok(payments) => {
                state.pending_payments = payments.into_iter()
                    .filter(|p| p.status != crate::domain::payment::PaymentStatus::Paid)
                    .collect();
            }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }

    // ── Navigation ────────────────────────────────────────────────────────────
    if ui.button("← Volver").clicked() {
        super::clear_detail_state(state);
        state.mode = Mode::List;
        return;
    }
    ui.separator();

    // ── Información ───────────────────────────────────────────────────────────
    section_header(ui, "Información");
    ui.heading(format!("{} {}", student.first_name, student.last_name));
    egui::Grid::new("student_detail_info").num_columns(2).spacing([16.0, 2.0]).show(ui, |ui| {
        ui.label(egui::RichText::new("Grupo").color(crate::theme::colors::TEXT_MUTED));
        ui.label(student.age_group.label());
        ui.end_row();
        ui.label(egui::RichText::new("Email").color(crate::theme::colors::TEXT_MUTED));
        ui.label(&student.email);
        ui.end_row();
        ui.label(egui::RichText::new("Teléfono").color(crate::theme::colors::TEXT_MUTED));
        ui.label(&student.phone);
        ui.end_row();
        if let Some(n) = &student.notes {
            ui.label(egui::RichText::new("Notas").color(crate::theme::colors::TEXT_MUTED));
            ui.label(n.as_str());
            ui.end_row();
        }
    });
    ui.add_space(4.0);

    // ── Actions + balance ─────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        let (bal_color, bal_text) = if state.balance_cents >= 0 {
            (crate::theme::colors::SUCCESS, format!("Balance: +{}", fmt_ars(state.balance_cents)))
        } else {
            (crate::theme::colors::ERROR,   format!("Balance: {}", fmt_ars(state.balance_cents)))
        };
        ui.colored_label(bal_color, bal_text);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Pago").clicked() && !state.show_payment_form {
                state.payment_amount    = String::new();
                state.show_payment_form = true;
                state.show_enroll_form  = false;
            }
            if ui.button("+ Inscribir").clicked() && !state.show_enroll_form {
                if let Ok(courses) = CourseGetAllUseCase::new(make_course_repo(client)).execute() {
                    state.enroll_courses       = courses.into_iter().filter(|c| c.age_group == student.age_group).collect();
                    state.enroll_sel_course    = None;
                    state.enroll_course_filter = String::new();
                    state.enroll_sel_period    = None;
                    state.enroll_period_filter = String::new();
                    state.enroll_periods       = Vec::new();
                    state.show_enroll_form     = true;
                    state.show_payment_form    = false;
                }
            }
        });
    });
    ui.separator();

    // ── Enroll form ───────────────────────────────────────────────────────────
    if state.show_enroll_form {
        ui.horizontal(|ui| {
            ui.label("Curso");
            egui::ComboBox::from_id_salt("enroll_course")
                .selected_text(
                    state.enroll_sel_course
                        .and_then(|id| state.enroll_courses.iter().find(|c| c.id == id))
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    ui.text_edit_singleline(&mut state.enroll_course_filter);
                    let cf = state.enroll_course_filter.to_lowercase();
                    let courses: Vec<_> = state.enroll_courses.iter()
                        .filter(|c| cf.is_empty() || c.name.to_lowercase().contains(&cf))
                        .cloned()
                        .collect();
                    for c in &courses {
                        if ui.selectable_value(&mut state.enroll_sel_course, Some(c.id), &c.name).clicked() {
                            if let Ok(ps) = CoursePeriodGetByCourseUseCase::new(make_course_period_repo(client)).execute(c.id) {
                                state.enroll_periods       = ps;
                                state.enroll_sel_period    = None;
                                state.enroll_period_filter = String::new();
                            }
                        }
                    }
                });

            ui.label("Período");
            egui::ComboBox::from_id_salt("enroll_period")
                .selected_text(
                    state.enroll_sel_period
                        .and_then(|id| state.enroll_periods.iter().find(|p| p.id == id))
                        .map(|p| p.label.clone())
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    ui.text_edit_singleline(&mut state.enroll_period_filter);
                    let pf = state.enroll_period_filter.to_lowercase();
                    let periods: Vec<_> = state.enroll_periods.iter()
                        .filter(|p| pf.is_empty() || p.label.to_lowercase().contains(&pf))
                        .cloned()
                        .collect();
                    for p in &periods {
                        ui.selectable_value(&mut state.enroll_sel_period, Some(p.id), &p.label);
                    }
                });

            // Show price for selected course
            if let Some(price) = state.enroll_sel_course
                .and_then(|id| state.enroll_courses.iter().find(|c| c.id == id))
                .map(|c| c.price_cents)
            {
                ui.colored_label(crate::theme::colors::TEXT_MUTED, fmt_ars(price));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancelar").clicked() {
                    state.show_enroll_form     = false;
                    state.enroll_sel_course    = None;
                    state.enroll_course_filter = String::new();
                    state.enroll_sel_period    = None;
                    state.enroll_period_filter = String::new();
                    state.enroll_periods       = Vec::new();
                }
                if ui.button("Inscribir").clicked() {
                    match state.enroll_sel_period {
                        Some(period_id) => {
                            match EnrollmentCreateUseCase::new(
                                make_enrollment_repo(client),
                                make_course_period_repo(client),
                                make_course_repo(client),
                            ).execute(EnrollmentCreateInput { student_id: student.id, course_period_id: period_id }) {
                                Ok(_) => {
                                    push_success(notifs, "Alumno inscrito");
                                    state.show_enroll_form     = false;
                                    state.enroll_sel_course    = None;
                                    state.enroll_course_filter = String::new();
                                    state.enroll_sel_period    = None;
                                    state.enroll_period_filter = String::new();
                                    state.enroll_periods       = Vec::new();
                                    state.needs_reload_ledger  = true;
                                }
                                Err(e) => push_error(notifs, e.to_string()),
                            }
                        }
                        None => push_error(notifs, "Seleccionar un período"),
                    }
                }
            });
        });
        ui.separator();
    }

    // ── Payment form ──────────────────────────────────────────────────────────
    if state.show_payment_form {
        ui.horizontal(|ui| {
            ui.label("Monto");
            ui.add(egui::TextEdit::singleline(&mut state.payment_amount).desired_width(70.0));

            ui.label("Fecha");
            date_selector(ui, "pay_date", &mut state.payment_due_date);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancelar").clicked() {
                    state.show_payment_form = false;
                    state.payment_amount    = String::new();
                }
                if ui.button("Guardar").clicked() {
                    let amount_cents = match parse_cents(&state.payment_amount) {
                        Some(v) => v,
                        None    => { push_error(notifs, "Monto inválido"); return; }
                    };
                    match PaymentCreateUseCase::new(make_payment_repo(client))
                        .execute(PaymentCreateInput { student_id: student.id, amount_cents, due_date: state.payment_due_date, notes: None }) {
                        Ok(_) => {
                            push_success(notifs, "Pago registrado");
                            state.show_payment_form   = false;
                            state.payment_amount      = String::new();
                            state.needs_reload_ledger = true;
                        }
                        Err(e) => push_error(notifs, e.to_string()),
                    }
                }
            });
        });
        ui.separator();
    }

    // ── Ledger table ──────────────────────────────────────────────────────────
    let mut action: Option<(LedgerAction, Uuid)> = None;

    table::builder(ui)
        .column(Column::auto().at_least(110.0))
        .column(Column::remainder().at_least(150.0))
        .column(Column::exact(80.0))
        .column(Column::exact(90.0))
        .column(Column::auto().at_least(60.0))
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Fecha"));
            h.col(|ui| table::head(ui, "Descripción"));
            h.col(|ui| table::head(ui, "Monto"));
            h.col(|ui| table::head(ui, "Balance"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for entry in &state.ledger {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(fmt_dt(entry.date)); });
                    row.col(|ui| { ui.label(&entry.description); });
                    row.col(|ui| {
                        match entry.kind {
                            LedgerKind::Debt => ui.colored_label(
                                crate::theme::colors::ERROR,
                                format!("-{}", fmt_ars(entry.amount_cents)),
                            ),
                            LedgerKind::Credit => ui.colored_label(
                                crate::theme::colors::SUCCESS,
                                format!("+{}", fmt_ars(entry.amount_cents)),
                            ),
                        };
                    });
                    row.col(|ui| {
                        let color = if entry.running_balance < 0 {
                            crate::theme::colors::ERROR
                        } else {
                            crate::theme::colors::SUCCESS
                        };
                        let sign = if entry.running_balance < 0 { "-" } else { "+" };
                        ui.colored_label(color, format!("{sign}{}", fmt_ars(entry.running_balance.abs())));
                    });
                    row.col(|ui| {
                        // Debt entries: mark paid or delete enrollment
                        // Credit entries: delete payment
                        match entry.kind {
                            LedgerKind::Debt => {
                                if ui.small_button("🗑").clicked() {
                                    action = Some((LedgerAction::DeleteEnrollment, entry.id));
                                }
                            }
                            LedgerKind::Credit => {
                                if ui.small_button("🗑").clicked() {
                                    action = Some((LedgerAction::DeletePayment, entry.id));
                                }
                            }
                        }
                    });
                });
            }
        });

    // Pending payments (shown below ledger, not counted in balance)
    let pending = {
        let enrollment_repo = make_enrollment_repo(client);
        let payment_repo    = make_payment_repo(client);
        // quick inline: get pending payments
        let _ = (enrollment_repo, payment_repo); // repos available if needed
        Vec::<(String, i32)>::new() // placeholder — pending payments shown via mark_paid action
    };
    let _ = pending;

    // Mark-paid shortcuts: show pending payments separately
    show_pending_payments(ui, client, state, notifs);

    if let Some((act, id)) = action {
        match act {
            LedgerAction::DeleteEnrollment => { state.confirm_delete = Some(id); }
            LedgerAction::DeletePayment    => { state.confirm_delete = Some(id); }
        }
        state.confirm_delete = Some(id);
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        // Try delete as enrollment first, then as payment
        let del_enroll = EnrollmentDeleteUseCase::new(make_enrollment_repo(client)).execute(id);
        if del_enroll.is_ok() {
            push_success(notifs, "Inscripción eliminada");
            state.needs_reload_ledger = true;
        } else {
            match PaymentDeleteUseCase::new(make_payment_repo(client)).execute(id) {
                Ok(_)  => { push_success(notifs, "Pago eliminado"); state.needs_reload_ledger = true; }
                Err(e) => push_error(notifs, e.to_string()),
            }
        }
    }
}

enum LedgerAction { DeleteEnrollment, DeletePayment }

fn show_pending_payments(
    ui: &mut egui::Ui,
    client: &Arc<Mutex<Client>>,
    state: &mut StudentsState,
    notifs: &mut Notifications,
) {
    use crate::domain::payment::PaymentStatus;

    if state.pending_payments.is_empty() { return; }

    ui.add_space(4.0);
    section_header(ui, "Pagos pendientes");
    let mut mark_id: Option<Uuid> = None;
    for p in &state.pending_payments {
        ui.horizontal(|ui| {
            let status_color = if p.status == PaymentStatus::Overdue {
                crate::theme::colors::ERROR
            } else {
                crate::theme::colors::WARNING
            };
            ui.colored_label(status_color, p.status.label());
            ui.label(format!("{} — vence {}", fmt_ars(p.amount_cents), p.due_date.format("%d/%m/%Y")));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✓ Marcar pagado").clicked() { mark_id = Some(p.id); }
            });
        });
    }
    if let Some(id) = mark_id {
        match PaymentMarkPaidUseCase::new(make_payment_repo(client)).execute(id) {
            Ok(_)  => { push_success(notifs, "Pago marcado como pagado"); state.needs_reload_ledger = true; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

fn parse_cents(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32).filter(|&c| c > 0)
}
