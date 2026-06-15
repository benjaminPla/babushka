use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, TimeZone, Utc};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use postgres::Client;
use uuid::Uuid;

use crate::theme::{colors, sizes};
use crate::application::course::get_all::CourseGetAllUseCase;
use crate::application::course_period::get_by_course::CoursePeriodGetByCourseUseCase;
use crate::application::enrollment::create::EnrollmentCreateInput;
use crate::application::enrollment::create::EnrollmentCreateUseCase;
use crate::application::enrollment::delete::EnrollmentDeleteUseCase;
use crate::application::payment::create::PaymentCreateInput;
use crate::application::payment::create::PaymentCreateUseCase;
use crate::application::payment::delete::PaymentDeleteUseCase;
use crate::application::student_ledger::LedgerKind;
use crate::application::student_ledger::StudentLedgerUseCase;
use crate::presentation::confirm_delete_modal;
use crate::presentation::date_selector;
use crate::presentation::fmt_ars;
use crate::presentation::fmt_dt;
use crate::presentation::push_error;
use crate::presentation::push_success;
use crate::presentation::Notifications;
use crate::presentation::table;

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

    if ui.button("<- Volver").clicked() {
        super::clear_detail_state(state);
        state.mode = Mode::List;
        return;
    }
    ui.separator();

    ui.label(egui::RichText::new("Información").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));

    egui::Grid::new("student_detail_info").num_columns(2).spacing([sizes::SPACING_NORMAL, sizes::SPACING_SMALL]).show(ui, |ui| {
        ui.label(egui::RichText::new("Nombre").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&student.first_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Apellido").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&student.last_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Grupo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(student.age_group.label()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Email").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&student.email).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Teléfono").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&student.phone).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        if let Some(n) = &student.notes {
            ui.label(egui::RichText::new("Notas").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(n.as_str()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();
        }
        ui.label(egui::RichText::new("Creado").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(fmt_dt(student.created_at)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Editado").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(fmt_dt(student.updated_at)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();
    });

    ui.add_space(sizes::SPACING_SMALL);
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Saldo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Pago").clicked() {
                state.payment_amount    = String::new();
                state.payment_method    = "cash".into();
                state.payment_paid_at   = today();
                state.payment_notes     = String::new();
                state.show_payment_form = true;
            }
            if ui.button("+ Inscribir").clicked() {
                if let Ok(courses) = CourseGetAllUseCase::new(make_course_repo(client)).execute() {
                    state.enroll_courses       = courses.into_iter().filter(|c| c.age_group == student.age_group).collect();
                    state.enroll_sel_course    = None;
                    state.enroll_course_filter = String::new();
                    state.enroll_sel_period    = None;
                    state.enroll_period_filter = String::new();
                    state.enroll_periods       = Vec::new();
                    state.show_enroll_form     = true;
                }
            }
            let (color, sign) = if state.balance_cents >= 0 {
                (colors::GREEN, "+")
            } else {
                (colors::RED, "")
            };
            ui.colored_label(color, format!("{sign}{}", fmt_ars(state.balance_cents)));
        });
    });

    if state.show_enroll_form {
        egui::Window::new("Inscribir alumno")
            .collapsible(false)
            .resizable(false)
            .auto_sized()
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::new()
                .fill(colors::BLACK)
                .stroke(egui::Stroke::new(sizes::STROKE_SMALL, colors::WHITE))
                .inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL))
            )
            .show(ui.ctx(), |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.label(egui::RichText::new("Curso").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("enroll_course")
                        .width(ui.available_width())
                        .selected_text(
                            state.enroll_sel_course
                                .and_then(|id| state.enroll_courses.iter().find(|c| c.id == id))
                                .map(|c| c.name.clone())
                                .unwrap_or_else(|| "Seleccionar...".into()),
                        )
                        .show_ui(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.enroll_course_filter).id(egui::Id::new("enroll_course_filter")));
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
                                        state.enroll_course_filter = String::new();
                                    }
                                }
                            }
                        });
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Período").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("enroll_period")
                        .width(ui.available_width())
                        .selected_text(
                            state.enroll_sel_period
                                .and_then(|id| state.enroll_periods.iter().find(|p| p.id == id))
                                .map(|p| p.label.clone())
                                .unwrap_or_else(|| "Seleccionar...".into()),
                        )
                        .show_ui(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.enroll_period_filter).id(egui::Id::new("enroll_period_filter")));
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
                        ui.add_space(sizes::SPACING_SMALL);
                        ui.colored_label(colors::DARK_GRAY, format!("Precio mensual: {}", fmt_ars(price)));
                    }
                    ui.add_space(sizes::SPACING_NORMAL);

                    ui.horizontal(|ui| {
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
                });
            });
    }

    if state.show_payment_form {
        egui::Window::new("Registrar pago")
            .collapsible(false)
            .resizable(false)
            .auto_sized()
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::new()
                .fill(colors::BLACK)
                .stroke(egui::Stroke::new(sizes::STROKE_SMALL, colors::WHITE))
                .inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL))
            )
            .show(ui.ctx(), |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.label(egui::RichText::new("Monto").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.payment_amount));
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Método").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("pay_method")
                        .width(ui.available_width())
                        .selected_text(method_label(&state.payment_method))
                        .show_ui(ui, |ui| {
                            for (val, label) in PAYMENT_METHODS {
                                ui.selectable_value(&mut state.payment_method, val.to_string(), *label);
                            }
                        });
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Fecha").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    date_selector(ui, "pay_date", &mut state.payment_paid_at);
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Notas").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::multiline(&mut state.payment_notes).desired_rows(4));
                    ui.add_space(sizes::SPACING_NORMAL);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                            if ui.button("Cancelar").clicked() {
                                state.show_payment_form = false;
                                state.payment_amount    = String::new();
                            }
                        });
                    });
                });
            });
    }

    // ── Ledger table ──────────────────────────────────────────────────────────
    let mut action: Option<(LedgerAction, Uuid)> = None;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::auto())
        .header(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut header| {
            header.col(|ui| { ui.label("Fecha"); });
            header.col(|ui| { ui.label("Descripción"); });
            header.col(|ui| { ui.label("Método"); });
            header.col(|ui| { ui.label("Monto"); });
            header.col(|ui| { ui.label("Balance"); });
            header.col(|ui| { ui.label("Acciones"); });
        })
        .body(|mut body| {
            for entry in &state.ledger {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut row| {
                    row.col(|ui| { ui.label(fmt_dt(entry.date)); });
                    row.col(|ui| { ui.label(&entry.description); });
                    row.col(|ui| {
                        match &entry.payment_method {
                            Some(m) => { ui.label(m); }
                            None    => { ui.label("—"); }
                        }
                    });
                    row.col(|ui| {
                        match entry.kind {
                            LedgerKind::Debt => ui.colored_label(
                                colors::RED,
                                format!("-{}", fmt_ars(entry.amount_cents)),
                            ),
                            LedgerKind::Credit => ui.colored_label(
                                colors::GREEN,
                                format!("+{}", fmt_ars(entry.amount_cents)),
                            ),
                        };
                    });
                    row.col(|ui| {
                        let color = if entry.running_balance < 0 {
                            colors::RED
                        } else {
                            colors::GREEN
                        };
                        let sign = if entry.running_balance < 0 { "" } else { "+" };
                        ui.colored_label(color, format!("{sign}{}", fmt_ars(entry.running_balance)));
                    });
                    row.col(|ui| {
                        if ui.small_button(egui_phosphor::regular::TRASH).clicked() {
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
