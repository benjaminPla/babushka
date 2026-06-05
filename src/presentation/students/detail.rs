use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, TimeZone, Utc};
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
        },
        payment::{
            create::{PaymentCreateInput, PaymentCreateUseCase},
            delete::PaymentDeleteUseCase,
        },
        student_ledger::{LedgerKind, StudentLedgerUseCase},
    },
    presentation::{
        confirm_delete_modal, date_selector, fmt_ars, fmt_dt, push_error, push_success,
        section_header, Notifications,
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
    }

    // ── Navigation ────────────────────────────────────────────────────────────
    if ui.button("<- Volver").clicked() {
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
        ui.label(egui::RichText::new("Creado").color(crate::theme::colors::TEXT_MUTED));
        ui.label(fmt_dt(student.created_at));
        ui.end_row();
        ui.label(egui::RichText::new("Editado").color(crate::theme::colors::TEXT_MUTED));
        ui.label(fmt_dt(student.updated_at));
        ui.end_row();
    });
    ui.add_space(4.0);

    // ── Balance + action buttons ──────────────────────────────────────────────
    ui.horizontal(|ui| {
        let (bal_color, bal_text) = if state.balance_cents >= 0 {
            (crate::theme::colors::SUCCESS, format!("Balance: +{}", fmt_ars(state.balance_cents)))
        } else {
            (crate::theme::colors::ERROR, format!("Balance: {}", fmt_ars(state.balance_cents)))
        };
        ui.colored_label(bal_color, bal_text);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Pago").clicked() && !state.show_payment_form {
                state.payment_amount    = String::new();
                state.payment_method    = "cash".into();
                state.payment_paid_at   = today();
                state.payment_notes     = String::new();
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

            if let Some(price) = state.enroll_sel_course
                .and_then(|id| state.enroll_courses.iter().find(|c| c.id == id))
                .map(|c| c.month_price_cents)
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
            ui.add(egui::TextEdit::singleline(&mut state.payment_amount).desired_width(80.0));

            ui.label("Método");
            egui::ComboBox::from_id_salt("pay_method")
                .selected_text(method_label(&state.payment_method))
                .show_ui(ui, |ui| {
                    for (val, label) in PAYMENT_METHODS {
                        ui.selectable_value(&mut state.payment_method, val.to_string(), *label);
                    }
                });

            ui.label("Fecha");
            date_selector(ui, "pay_date", &mut state.payment_paid_at);

            ui.label("Notas");
            ui.add(egui::TextEdit::singleline(&mut state.payment_notes).desired_width(120.0));

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
                    let paid_at = naive_date_to_utc(state.payment_paid_at);
                    let notes   = if state.payment_notes.trim().is_empty() { None } else { Some(state.payment_notes.trim().to_owned()) };
                    match PaymentCreateUseCase::new(make_payment_repo(client))
                        .execute(PaymentCreateInput {
                            student_id:     student.id,
                            amount_cents,
                            payment_method: state.payment_method.clone(),
                            paid_at,
                            notes,
                        }) {
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

    // ── Ledger ────────────────────────────────────────────────────────────────
    let mut action: Option<(LedgerAction, Uuid)> = None;

    table::builder(ui)
        .column(Column::auto().at_least(110.0))
        .column(Column::remainder().at_least(150.0))
        .column(Column::exact(80.0))
        .column(Column::exact(90.0))
        .column(Column::auto().at_least(40.0))
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
                        let sign = if entry.running_balance < 0 { "" } else { "+" };
                        ui.colored_label(color, format!("{sign}{}", fmt_ars(entry.running_balance)));
                    });
                    row.col(|ui| {
                        if ui.small_button("🗑").clicked() {
                            action = Some((
                                match entry.kind {
                                    LedgerKind::Debt   => LedgerAction::DeleteEnrollment,
                                    LedgerKind::Credit => LedgerAction::DeletePayment,
                                },
                                entry.id,
                            ));
                        }
                    });
                });
            }
        });

    if let Some((_, id)) = action {
        state.confirm_delete = Some(id);
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
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

const PAYMENT_METHODS: &[(&str, &str)] = &[
    ("cash",     "Efectivo"),
    ("transfer", "Transferencia"),
    ("card",     "Tarjeta"),
    ("discount", "Descuento"),
];

fn method_label(method: &str) -> &str {
    PAYMENT_METHODS.iter()
        .find(|(val, _)| *val == method)
        .map(|(_, label)| *label)
        .unwrap_or("Efectivo")
}

fn today() -> NaiveDate {
    use chrono::{Datelike, Local};
    let n = Local::now();
    NaiveDate::from_ymd_opt(n.year(), n.month(), n.day()).unwrap()
}

fn naive_date_to_utc(date: NaiveDate) -> chrono::DateTime<Utc> {
    Utc.from_utc_datetime(&date.and_hms_opt(12, 0, 0).unwrap())
}

fn parse_cents(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32).filter(|&c| c > 0)
}
