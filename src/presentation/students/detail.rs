use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        course::get_all::CourseGetAllUseCase,
        course_period::get_by_course::CoursePeriodGetByCourseUseCase,
        enrollment::{
            create::{EnrollmentCreateInput, EnrollmentCreateUseCase},
            delete::EnrollmentDeleteUseCase,
            get_by_student::EnrollmentGetByStudentUseCase,
            set_dropped::{EnrollmentSetDroppedInput, EnrollmentSetDroppedUseCase},
        },
        payment::{
            create::{PaymentCreateInput, PaymentCreateUseCase},
            delete::PaymentDeleteUseCase,
            get_by_student::PaymentGetByStudentUseCase,
            mark_paid::PaymentMarkPaidUseCase,
        },
    },
    domain::{enrollment::EffectiveStatus, payment::PaymentStatus},
    presentation::{confirm_delete_modal, date_selector, fmt_dt, push_error, push_success, section_header, Notifications},
    presentation::table::{self, Column},
};

use super::{
    DetailTab, Mode, StudentsState,
    make_course_period_repo, make_course_repo, make_enrollment_repo, make_payment_repo,
};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState, notifs: &mut Notifications) {
    let student = match &state.selected_student {
        Some(s) => s.clone(),
        None    => { state.mode = Mode::List; return; }
    };

    // ── Header ───────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            super::clear_detail_state(state);
            state.mode = Mode::List;
            return;
        }
        ui.heading(format!("{} {}", student.first_name, student.last_name));
    });
    ui.separator();

    // ── Info section ─────────────────────────────────────────────────────────
    section_header(ui, "Información");
    egui::Grid::new("student_info").num_columns(2).show(ui, |ui| {
        ui.label("Email");    ui.label(&student.email);         ui.end_row();
        ui.label("Teléfono"); ui.label(&student.phone);         ui.end_row();
        ui.label("Tipo");     ui.label(student.age_group.label()); ui.end_row();
    });
    ui.add_space(4.0);
    ui.separator();

    // ── Tabs ─────────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.detail_tab, DetailTab::Inscripciones, "Inscripciones");
        ui.selectable_value(&mut state.detail_tab, DetailTab::Pagos,         "Pagos");
    });
    ui.separator();

    match state.detail_tab.clone() {
        DetailTab::Inscripciones => show_inscripciones(ui, client, state, &student, notifs),
        DetailTab::Pagos         => show_pagos(ui, client, state, &student, notifs),
    }
}

// ── Inscripciones tab ────────────────────────────────────────────────────────

fn show_inscripciones(
    ui: &mut egui::Ui,
    client: &Arc<Mutex<Client>>,
    state: &mut StudentsState,
    student: &crate::application::student::dto::StudentDto,
    notifs: &mut Notifications,
) {
    if state.needs_reload_enrollments {
        match EnrollmentGetByStudentUseCase::new(make_enrollment_repo(client)).execute(student.id) {
            Ok(e)  => { state.student_enrollments = e; state.needs_reload_enrollments = false; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }

    if state.show_enroll_form {
        ui.heading("Inscribir en período");
        egui::Grid::new("enroll_form").num_columns(2).show(ui, |ui| {
            ui.label("Curso");
            egui::ComboBox::from_id_salt("enroll_course")
                .selected_text(
                    state.enroll_sel_course
                        .and_then(|id| state.enroll_courses.iter().find(|c| c.id == id))
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    for c in &state.enroll_courses {
                        if ui.selectable_value(&mut state.enroll_sel_course, Some(c.id), &c.name).clicked() {
                            if let Ok(ps) = CoursePeriodGetByCourseUseCase::new(make_course_period_repo(client)).execute(c.id) {
                                state.enroll_periods    = ps;
                                state.enroll_sel_period = None;
                            }
                        }
                    }
                });
            ui.end_row();

            ui.label("Período");
            egui::ComboBox::from_id_salt("enroll_period")
                .selected_text(
                    state.enroll_sel_period
                        .and_then(|id| state.enroll_periods.iter().find(|p| p.id == id))
                        .map(|p| p.label.clone())
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    for p in &state.enroll_periods {
                        ui.selectable_value(&mut state.enroll_sel_period, Some(p.id), &p.label);
                    }
                });
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("Inscribir").clicked() {
                match state.enroll_sel_period {
                    Some(period_id) => {
                        match EnrollmentCreateUseCase::new(make_enrollment_repo(client))
                            .execute(EnrollmentCreateInput { student_id: student.id, course_period_id: period_id }) {
                            Ok(_) => {
                                push_success(notifs, "Alumno inscrito");
                                state.show_enroll_form         = false;
                                state.enroll_sel_course        = None;
                                state.enroll_sel_period        = None;
                                state.enroll_periods           = Vec::new();
                                state.needs_reload_enrollments = true;
                            }
                            Err(e) => push_error(notifs, e.to_string()),
                        }
                    }
                    None => push_error(notifs, "Seleccionar un período"),
                }
            }
            if ui.button("Cancelar").clicked() {
                state.show_enroll_form  = false;
                state.enroll_sel_course = None;
                state.enroll_sel_period = None;
                state.enroll_periods    = Vec::new();
            }
        });
        ui.separator();
    } else {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+ Inscribir").clicked() {
                    if let Ok(courses) = CourseGetAllUseCase::new(make_course_repo(client)).execute() {
                        state.enroll_courses    = courses.into_iter().filter(|c| c.age_group == student.age_group).collect();
                        state.enroll_sel_course = None;
                        state.enroll_sel_period = None;
                        state.enroll_periods    = Vec::new();
                        state.show_enroll_form  = true;
                    }
                }
            });
        });
    }

    let mut enroll_action: Option<(EnrollAction, Uuid)> = None;

    table::builder(ui)
        .column(Column::remainder().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(70.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(80.0))
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Curso"));
            h.col(|ui| table::head(ui, "Período"));
            h.col(|ui| table::head(ui, "Estado"));
            h.col(|ui| table::head(ui, "Último pago"));
            h.col(|ui| table::head(ui, "Inscripto"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for e in &state.student_enrollments {
                let status = e.effective_status();
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&e.course_name); });
                    row.col(|ui| { ui.label(&e.period_label); });
                    row.col(|ui| {
                        let color = match &status {
                            EffectiveStatus::Active    => crate::theme::colors::SUCCESS,
                            EffectiveStatus::Dropped   => crate::theme::colors::ERROR,
                            EffectiveStatus::Completed => crate::theme::colors::TEXT_MUTED,
                        };
                        ui.colored_label(color, status.label());
                    });
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
                                    enroll_action = Some((EnrollAction::SetDropped(false), e.id));
                                }
                            }
                            EffectiveStatus::Active => {
                                if ui.small_button("Dar de baja").clicked() {
                                    enroll_action = Some((EnrollAction::SetDropped(true), e.id));
                                }
                            }
                            EffectiveStatus::Completed => {}
                        }
                        if ui.small_button("🗑").clicked() {
                            enroll_action = Some((EnrollAction::Delete, e.id));
                        }
                    });
                });
            }
        });

    if let Some((act, id)) = enroll_action {
        match act {
            EnrollAction::SetDropped(dropped) => {
                match EnrollmentSetDroppedUseCase::new(make_enrollment_repo(client))
                    .execute(EnrollmentSetDroppedInput { id, dropped }) {
                    Ok(_)  => state.needs_reload_enrollments = true,
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            EnrollAction::Delete => { state.confirm_delete_enrollment = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete_enrollment) {
        match EnrollmentDeleteUseCase::new(make_enrollment_repo(client)).execute(id) {
            Ok(_)  => { push_success(notifs, "Inscripción eliminada"); state.needs_reload_enrollments = true; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

enum EnrollAction { SetDropped(bool), Delete }

// ── Pagos tab ────────────────────────────────────────────────────────────────

fn show_pagos(
    ui: &mut egui::Ui,
    client: &Arc<Mutex<Client>>,
    state: &mut StudentsState,
    student: &crate::application::student::dto::StudentDto,
    notifs: &mut Notifications,
) {
    if state.needs_reload_payments {
        match PaymentGetByStudentUseCase::new(make_payment_repo(client)).execute(student.id) {
            Ok(p)  => { state.student_payments = p; state.needs_reload_payments = false; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }

    if state.show_payment_form {
        ui.heading("Registrar pago");
        egui::Grid::new("payment_form").num_columns(2).show(ui, |ui| {
            ui.label("Inscripción");
            egui::ComboBox::from_id_salt("pay_enrollment")
                .selected_text(
                    state.payment_sel_enrollment
                        .and_then(|id| state.payment_enrollments.iter().find(|e| e.id == id))
                        .map(|e| format!("{} — {}", e.course_name, e.period_label))
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    for e in &state.payment_enrollments {
                        let label = format!("{} — {}", e.course_name, e.period_label);
                        ui.selectable_value(&mut state.payment_sel_enrollment, Some(e.id), label);
                    }
                });
            ui.end_row();
            ui.label("Monto");
            ui.text_edit_singleline(&mut state.payment_amount);
            ui.end_row();
            ui.label("Fecha");
            date_selector(ui, "pay_date", &mut state.payment_due_date);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("Guardar").clicked() {
                let enrollment_id = match state.payment_sel_enrollment {
                    Some(id) => id,
                    None     => { push_error(notifs, "Seleccionar inscripción"); return; }
                };
                let amount_cents = match parse_cents(&state.payment_amount) {
                    Some(v) => v,
                    None    => { push_error(notifs, "Monto inválido"); return; }
                };
                match PaymentCreateUseCase::new(make_payment_repo(client))
                    .execute(PaymentCreateInput { enrollment_id, amount_cents, due_date: state.payment_due_date, notes: None }) {
                    Ok(_) => {
                        push_success(notifs, "Pago registrado");
                        state.show_payment_form      = false;
                        state.payment_amount         = String::new();
                        state.payment_sel_enrollment = None;
                        state.needs_reload_payments  = true;
                    }
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            if ui.button("Cancelar").clicked() {
                state.show_payment_form      = false;
                state.payment_sel_enrollment = None;
                state.payment_amount         = String::new();
            }
        });
        ui.separator();
    } else {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+ Registrar pago").clicked() {
                    if let Ok(enrollments) = EnrollmentGetByStudentUseCase::new(make_enrollment_repo(client)).execute(student.id) {
                        state.payment_enrollments    = enrollments;
                        state.payment_sel_enrollment = None;
                        state.show_payment_form      = true;
                    }
                }
            });
        });
    }

    let mut pay_action: Option<(PayAction, Uuid)> = None;

    table::builder(ui)
        .column(Column::remainder().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::exact(75.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::auto().at_least(80.0))
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Curso"));
            h.col(|ui| table::head(ui, "Período"));
            h.col(|ui| table::head(ui, "Monto"));
            h.col(|ui| table::head(ui, "Estado"));
            h.col(|ui| table::head(ui, "Pagado"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for p in &state.student_payments {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&p.course_name); });
                    row.col(|ui| { ui.label(&p.period_label); });
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
                            if ui.small_button("✓ Pagar").clicked() {
                                pay_action = Some((PayAction::MarkPaid, p.id));
                            }
                        }
                        if ui.small_button("🗑").clicked() {
                            pay_action = Some((PayAction::Delete, p.id));
                        }
                    });
                });
            }
        });

    if let Some((act, id)) = pay_action {
        match act {
            PayAction::MarkPaid => {
                match PaymentMarkPaidUseCase::new(make_payment_repo(client)).execute(id) {
                    Ok(_)  => { push_success(notifs, "Pago marcado como pagado"); state.needs_reload_payments = true; }
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            PayAction::Delete => { state.confirm_delete_payment = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete_payment) {
        match PaymentDeleteUseCase::new(make_payment_repo(client)).execute(id) {
            Ok(_)  => { push_success(notifs, "Pago eliminado"); state.needs_reload_payments = true; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

enum PayAction { MarkPaid, Delete }

fn parse_cents(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32).filter(|&c| c > 0)
}
