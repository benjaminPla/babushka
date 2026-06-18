use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, TimeZone, Utc};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use postgres::Client;

use crate::application::course::get_all::CourseGetAllUseCase;
use crate::application::course_period::get_by_course::CoursePeriodGetByCourseUseCase;
use crate::application::enrollment::create::{EnrollmentCreateInput, EnrollmentCreateUseCase};
use crate::application::enrollment::delete::EnrollmentDeleteUseCase;
use crate::application::enrollment::pay::{EnrollmentPayInput, EnrollmentPayUseCase};
use crate::domain::enrollment::repository::EnrollmentRepo;
use crate::presentation::confirm_delete_modal;
use crate::presentation::date_selector;
use crate::presentation::fmt_ars;
use crate::presentation::fmt_dt;
use crate::presentation::push_error;
use crate::presentation::push_success;
use crate::presentation::Notifications;
use crate::theme::{colors, sizes};

use super::{
    Mode, StudentsState,
    make_course_period_repo, make_course_repo, make_enrollment_repo,
};

const PAYMENT_METHODS: &[(&str, &str)] = &[
    ("cash",     "Efectivo"),
    ("transfer", "Transferencia"),
    ("card",     "Tarjeta"),
];

fn method_label(s: &str) -> &str {
    PAYMENT_METHODS.iter().find(|(v, _)| *v == s).map(|(_, l)| *l).unwrap_or(s)
}

fn parse_cents(s: &str) -> Option<i32> {
    s.trim().parse::<f64>().ok().map(|f| (f * 100.0).round() as i32)
}

fn naive_date_to_utc(d: NaiveDate) -> chrono::DateTime<Utc> {
    Utc.from_utc_datetime(&d.and_hms_opt(12, 0, 0).unwrap())
}

fn today() -> NaiveDate {
    use chrono::{Datelike, Local};
    let n = Local::now();
    NaiveDate::from_ymd_opt(n.year(), n.month(), n.day()).unwrap()
}

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut StudentsState, notifs: &mut Notifications) {
    let student = match &state.selected_student {
        Some(s) => s.clone(),
        None    => { state.mode = Mode::List; return; }
    };

    if state.needs_reload_enrollments {
        state.needs_reload_enrollments = false;
        match make_enrollment_repo(client).get_by_student(student.id) {
            Ok(enrollments) => state.enrollments = enrollments,
            Err(e)          => push_error(notifs, e.to_string()),
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
        ui.label(egui::RichText::new("Movimientos").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Inscribir").clicked() {
                if let Ok(courses) = CourseGetAllUseCase::new(make_course_repo(client)).execute() {
                    state.enroll_courses       = courses.into_iter().filter(|c| c.age_group == student.age_group).collect();
                    state.enroll_sel_course    = None;
                    state.enroll_course_filter = String::new();
                    state.enroll_sel_period    = None;
                    state.enroll_period_filter = String::new();
                    state.enroll_periods       = Vec::new();
                    state.enroll_pricing_type  = "monthly".into();
                    state.show_enroll_form     = true;
                }
            }
        });
    });

    // ── Enroll form ───────────────────────────────────────────────────────────
    if state.show_enroll_form {
        egui::Window::new("Inscribir alumno")
            .collapsible(false)
            .resizable(false)
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
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Precio").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut state.enroll_pricing_type, "monthly".into(), "Mensual");
                        ui.radio_value(&mut state.enroll_pricing_type, "class".into(),   "Por clase");
                    });
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
                                        ).execute(EnrollmentCreateInput {
                                            student_id:       student.id,
                                            course_period_id: period_id,
                                            pricing_type:     state.enroll_pricing_type.clone(),
                                        }) {
                                            Ok(_) => {
                                                push_success(notifs, "Alumno inscrito");
                                                state.show_enroll_form           = false;
                                                state.enroll_sel_course          = None;
                                                state.enroll_course_filter       = String::new();
                                                state.enroll_sel_period          = None;
                                                state.enroll_period_filter       = String::new();
                                                state.enroll_periods             = Vec::new();
                                                state.needs_reload_enrollments   = true;
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

    // ── Payment form ──────────────────────────────────────────────────────────
    if state.show_payment_form {
        // Find the enrollment being paid to show reference prices
        let (course_info, pricing_type_str) = state.pay_enrollment_id
            .and_then(|eid| state.enrollments.iter().find(|e| e.id() == eid))
            .and_then(|e| {
                state.enroll_courses.iter().find(|c| c.id == e.course_id())
                    .map(|c| (Some(c.clone()), e.pricing_type().to_owned()))
            })
            .unwrap_or((None, "monthly".into()));

        egui::Window::new("Registrar pago")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::new()
                .fill(colors::BLACK)
                .stroke(egui::Stroke::new(sizes::STROKE_SMALL, colors::WHITE))
                .inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL))
            )
            .show(ui.ctx(), |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    // Reference prices — only the 2 relevant to this enrollment's pricing_type
                    if let Some(ref c) = course_info {
                        let is_monthly = pricing_type_str == "monthly";
                        let (label, price_cash, price_transfer) = if is_monthly {
                            ("Mensual", c.month_price_cash_cents, c.month_price_transfer_cents)
                        } else {
                            ("Por clase", c.class_price_cash_cents, c.class_price_transfer_cents)
                        };
                        ui.label(egui::RichText::new(format!("Precio de referencia ({label})")).color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                        egui::Grid::new("pay_prices")
                            .num_columns(2)
                            .spacing([sizes::SPACING_NORMAL, 2.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Efectivo").color(colors::LIGHT_GRAY));
                                ui.label(egui::RichText::new(fmt_ars(price_cash)).color(colors::WHITE));
                                ui.end_row();
                                ui.label(egui::RichText::new("Transferencia").color(colors::LIGHT_GRAY));
                                ui.label(egui::RichText::new(fmt_ars(price_transfer)).color(colors::WHITE));
                                ui.end_row();
                            });
                        ui.add_space(sizes::SPACING_SMALL);
                    }

                    ui.label(egui::RichText::new("Monto").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    let amt_resp = ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.pay_amount));
                    if amt_resp.gained_focus() && state.pay_amount.is_empty() {
                        // auto-fill on first focus
                        auto_fill_amount(state, &course_info, &pricing_type_str);
                    }
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Método").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    let method_resp = egui::ComboBox::from_id_salt("pay_method")
                        .width(ui.available_width())
                        .selected_text(method_label(&state.pay_method))
                        .show_ui(ui, |ui| {
                            for (val, label) in PAYMENT_METHODS {
                                ui.selectable_value(&mut state.pay_method, val.to_string(), *label);
                            }
                        });
                    if method_resp.response.changed() {
                        auto_fill_amount(state, &course_info, &pricing_type_str);
                    }
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Fecha").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    date_selector(ui, "pay_date", &mut state.pay_date);
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Notas").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::multiline(&mut state.pay_notes).desired_rows(3));
                    ui.add_space(sizes::SPACING_NORMAL);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Guardar").clicked() {
                                let amount_cents = match parse_cents(&state.pay_amount) {
                                    Some(v) => v,
                                    None    => { push_error(notifs, "Monto inválido"); return; }
                                };
                                let enrollment_id = match state.pay_enrollment_id {
                                    Some(id) => id,
                                    None     => { push_error(notifs, "Sin inscripción seleccionada"); return; }
                                };
                                let paid_at = naive_date_to_utc(state.pay_date);
                                let notes = if state.pay_notes.trim().is_empty() { None } else { Some(state.pay_notes.trim().to_owned()) };
                                match EnrollmentPayUseCase::new(make_enrollment_repo(client))
                                    .execute(EnrollmentPayInput {
                                        enrollment_id,
                                        amount_cents,
                                        payment_method: state.pay_method.clone(),
                                        paid_at,
                                        notes,
                                    }) {
                                    Ok(_) => {
                                        push_success(notifs, "Pago registrado");
                                        state.show_payment_form        = false;
                                        state.pay_enrollment_id        = None;
                                        state.pay_amount               = String::new();
                                        state.pay_notes                = String::new();
                                        state.needs_reload_enrollments = true;
                                    }
                                    Err(e) => push_error(notifs, e.to_string()),
                                }
                            }
                            if ui.button("Cancelar").clicked() {
                                state.show_payment_form = false;
                                state.pay_enrollment_id = None;
                                state.pay_amount        = String::new();
                                state.pay_notes         = String::new();
                            }
                        });
                    });
                });
            });
    }

    // ── Enrollments table ─────────────────────────────────────────────────────
    let mut delete_id: Option<uuid::Uuid> = None;
    let mut pay_id:    Option<uuid::Uuid> = None;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::auto())
        .header(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut header| {
            header.col(|ui| { ui.label("Curso"); });
            header.col(|ui| { ui.label("Período"); });
            header.col(|ui| { ui.label("Tipo"); });
            header.col(|ui| { ui.label("Monto"); });
            header.col(|ui| { ui.label("Acciones"); });
        })
        .body(|mut body| {
            for enrollment in &state.enrollments {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut row| {
                    row.col(|ui| { ui.label(enrollment.course_name()); });
                    row.col(|ui| { ui.label(enrollment.period_label()); });
                    row.col(|ui| {
                        let tipo = if enrollment.is_monthly() { "Mensual" } else { "Por clase" };
                        ui.label(egui::RichText::new(tipo).color(colors::LIGHT_GRAY));
                    });
                    row.col(|ui| {
                        if enrollment.is_paid() {
                            let amt = enrollment.paid_amount_cents().unwrap_or(0);
                            ui.colored_label(colors::GREEN, fmt_ars(amt));
                        } else {
                            ui.colored_label(colors::LIGHT_GRAY, "pendiente");
                        }
                    });
                    row.col(|ui| {
                        if !enrollment.is_paid() {
                            if ui.small_button("Pagar").clicked() {
                                pay_id = Some(enrollment.id());
                            }
                        }
                        if ui.small_button(egui_phosphor::regular::TRASH).clicked() {
                            delete_id = Some(enrollment.id());
                        }
                    });
                });
            }
        });

    if let Some(eid) = pay_id {
        // Load courses for price reference if not yet loaded
        if state.enroll_courses.is_empty() {
            if let Ok(courses) = CourseGetAllUseCase::new(make_course_repo(client)).execute() {
                state.enroll_courses = courses.into_iter().filter(|c| c.age_group == student.age_group).collect();
            }
        }
        let course = state.enrollments.iter()
            .find(|e| e.id() == eid)
            .and_then(|e| state.enroll_courses.iter().find(|c| c.id == e.course_id()))
            .cloned();
        let pricing_type = state.enrollments.iter()
            .find(|e| e.id() == eid)
            .map(|e| e.pricing_type().to_owned())
            .unwrap_or_else(|| "monthly".into());

        state.pay_enrollment_id = Some(eid);
        state.pay_amount        = String::new();
        state.pay_method        = "cash".into();
        state.pay_date          = today();
        state.pay_notes         = String::new();
        state.show_payment_form = true;
        // Pre-fill amount
        auto_fill_amount(state, &course, &pricing_type);
    }

    if let Some(id) = delete_id {
        state.confirm_delete = Some(id);
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match EnrollmentDeleteUseCase::new(make_enrollment_repo(client)).execute(id) {
            Ok(_)  => {
                push_success(notifs, "Inscripción eliminada");
                state.needs_reload_enrollments = true;
            }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

fn auto_fill_amount(state: &mut StudentsState, course: &Option<crate::application::course::dto::CourseDto>, pricing_type: &str) {
    let c = match course { Some(c) => c, None => return };
    let is_monthly = pricing_type == "monthly";
    let price = match (is_monthly, state.pay_method.as_str()) {
        (true,  "transfer") | (true,  "card") => c.month_price_transfer_cents,
        (true,  _)                             => c.month_price_cash_cents,
        (false, "transfer") | (false, "card") => c.class_price_transfer_cents,
        (false, _)                             => c.class_price_cash_cents,
    };
    state.pay_amount = format!("{:.2}", price as f64 / 100.0);
}
