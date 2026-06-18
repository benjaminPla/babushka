use std::sync::{Arc, Mutex};

use chrono::{Duration, NaiveDate};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use postgres::Client;
use uuid::Uuid;

use crate::application::course_period::create::CoursePeriodCreateInput;
use crate::application::course_period::create::CoursePeriodCreateUseCase;
use crate::application::course_period::delete::CoursePeriodDeleteUseCase;
use crate::application::course_period::get_by_course::CoursePeriodGetByCourseUseCase;
use crate::application::teacher::get_all::TeacherGetAllUseCase;
use crate::presentation::confirm_delete_modal;
use crate::presentation::push_error;
use crate::presentation::push_success;
use crate::presentation::Notifications;
use crate::theme::colors;
use crate::theme::sizes;

use crate::domain::enrollment::repository::EnrollmentRepo;

use super::{format_price, make_course_period_repo, make_enrollment_repo, CoursesState, Mode};

pub fn show(
    ui:     &mut egui::Ui,
    client: &Arc<Mutex<Client>>,
    state:  &mut CoursesState,
    notifs: &mut Notifications,
) {
    let course = match &state.selected_course {
        Some(c) => c.clone(),
        None => {
            state.mode = Mode::List;
            return;
        }
    };

    if state.teachers.is_empty() {
        if let Ok(ts) = TeacherGetAllUseCase::new(super::make_teacher_repo(client)).execute() {
            state.teachers = ts;
        }
    }

    if state.needs_reload_periods {
        state.needs_reload_periods = false;
        match CoursePeriodGetByCourseUseCase::new(make_course_period_repo(client)).execute(course.id) {
            Ok(periods) => state.periods = periods,
            Err(e)      => push_error(notifs, e.to_string()),
        }
    }

    if state.needs_reload_students {
        state.needs_reload_students = false;
        match make_enrollment_repo(client).get_by_course(course.id) {
            Ok(students) => state.course_students = students,
            Err(e)       => push_error(notifs, e.to_string()),
        }
    }

    if ui.button("<- Volver").clicked() {
        state.mode              = Mode::List;
        state.selected_course   = None;
        state.periods           = Vec::new();
        state.course_students   = Vec::new();
        state.show_period_form  = false;
        return;
    }
    ui.separator();

    ui.columns(2, |cols| {
        // ── Left: course info ─────────────────────────────────────────────
        cols[0].label(egui::RichText::new("Información").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        egui::Grid::new("course_details").num_columns(2).spacing([sizes::SPACING_NORMAL, sizes::SPACING_SMALL]).show(&mut cols[0], |ui| {
            let teacher_name = state.teachers.iter()
                .find(|t| t.id == course.teacher_id)
                .map(|t| format!("{} {}", t.first_name, t.last_name))
                .unwrap_or_else(|| course.teacher_id.to_string());

            ui.label(egui::RichText::new("Nombre").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(&course.name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Profesor").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(teacher_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Grupo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(course.age_group.label()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Capacidad").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(course.capacity.to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Mensual efectivo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(format_price(course.month_price_cash_cents)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Mensual transferencia").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(format_price(course.month_price_transfer_cents)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Clase efectivo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(format_price(course.class_price_cash_cents)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Clase transferencia").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(format_price(course.class_price_transfer_cents)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            if let Some(n) = &course.notes {
                ui.label(egui::RichText::new("Notas").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new(n.as_str()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
            }

            ui.label(egui::RichText::new("Creado").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(crate::presentation::fmt_dt(course.created_at)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();

            ui.label(egui::RichText::new("Editado").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(crate::presentation::fmt_dt(course.updated_at)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();
        });

        // ── Right: enrolled students ──────────────────────────────────────
        let n = state.course_students.len();
        cols[1].label(egui::RichText::new(format!("Alumnos ({n})")).color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        TableBuilder::new(&mut cols[1])
            .striped(true)
            .max_scroll_height(250.0)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::remainder())
            .column(Column::auto())
            .header(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut header| {
                header.col(|ui| { ui.label("Nombre"); });
                header.col(|ui| { ui.label("Estado"); });
            })
            .body(|mut body| {
                for e in &state.course_students {
                    body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut row| {
                        row.col(|ui| { ui.label(e.student_name()); });
                        row.col(|ui| {
                            if e.is_paid() {
                                ui.colored_label(colors::GREEN,      "pagado");
                            } else {
                                ui.colored_label(colors::LIGHT_GRAY, "pendiente");
                            }
                        });
                    });
                }
            });
    });

    ui.add_space(sizes::SPACING_SMALL);
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Períodos").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Nuevo período").clicked() {
                state.show_period_form = true;
            }
        });
    });

    if state.show_period_form {
        egui::Window::new("Nuevo período")
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
                    ui.label(egui::RichText::new("Año").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("period_year")
                        .width(ui.available_width())
                        .selected_text(state.period_year.to_string())
                        .show_ui(ui, |ui| {
                            for y in 2024..=2030 {
                                ui.selectable_value(&mut state.period_year, y, y.to_string());
                            }
                        });
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Mes").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("period_month")
                        .width(ui.available_width())
                        .selected_text(MONTHS[(state.period_month - 1) as usize])
                        .show_ui(ui, |ui| {
                            for (i, name) in MONTHS.iter().enumerate() {
                                ui.selectable_value(&mut state.period_month, (i + 1) as u32, *name);
                            }
                        });
                    ui.add_space(sizes::SPACING_NORMAL);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Guardar").clicked() {
                                let y = state.period_year;
                                let m = state.period_month;
                                let start_date = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
                                let end_date = if m == 12 {
                                    NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap() - Duration::days(1)
                                } else {
                                    NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap() - Duration::days(1)
                                };
                                match CoursePeriodCreateUseCase::new(make_course_period_repo(client)).execute(
                                    CoursePeriodCreateInput { course_id: course.id, start_date, end_date },
                                ) {
                                    Ok(_) => {
                                        push_success(notifs, "Período creado");
                                        state.show_period_form     = false;
                                        state.needs_reload_periods = true;
                                    }
                                    Err(e) => push_error(notifs, e.to_string()),
                                }
                            }
                            if ui.button("Cancelar").clicked() {
                                state.show_period_form = false;
                            }
                        });
                    });
                });
            });
    }

    let mut delete_id: Option<Uuid> = None;

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
            header.col(|ui| { ui.label("Etiqueta"); });
            header.col(|ui| { ui.label("Inicio"); });
            header.col(|ui| { ui.label("Fin"); });
            header.col(|ui| { ui.label("Inscritos"); });
            header.col(|ui| { ui.label("Estado"); });
            header.col(|ui| { ui.label("Acciones"); });
        })
        .body(|mut body| {
            for p in &state.periods {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut row| {
                    row.col(|ui| { ui.label(&p.label); });
                    row.col(|ui| { ui.label(p.start_date.format("%d/%m/%Y").to_string()); });
                    row.col(|ui| { ui.label(p.end_date.format("%d/%m/%Y").to_string()); });
                    row.col(|ui| { ui.label(p.enrolled.to_string()); });
                    row.col(|ui| {
                        let today = chrono::Local::now().date_naive();
                        if p.start_date > today {
                            ui.colored_label(crate::theme::colors::YELLOW, "Futuro");
                        } else if p.end_date >= today {
                            ui.colored_label(crate::theme::colors::GREEN, "Activo");
                        } else {
                            ui.colored_label(crate::theme::colors::LIGHT_GRAY, "Finalizado");
                        }
                    });
                    row.col(|ui| {
                        if ui.small_button(egui_phosphor::regular::TRASH).clicked() { delete_id = Some(p.id); }
                    });
                });
            }
        });

    if let Some(id) = delete_id { state.confirm_delete_period = Some(id); }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete_period) {
        match CoursePeriodDeleteUseCase::new(make_course_period_repo(client)).execute(id) {
            Ok(_)  => { push_success(notifs, "Período eliminado"); state.needs_reload_periods = true; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

const MONTHS: [&str; 12] = [
    "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
    "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre",
];
