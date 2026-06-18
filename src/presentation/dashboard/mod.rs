use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use postgres::Client;
use uuid::Uuid;

use crate::application::dashboard::{DashboardData, DashboardUseCase, DebtorRow};
use crate::domain::course::repository::CourseRepo;
use crate::domain::student::repository::StudentRepo;
use crate::domain::teacher::repository::TeacherRepo;
use crate::infrastructure::enrollment::EnrollmentPgRepo;
use crate::infrastructure::payment::PaymentPgRepo;
use crate::presentation::{push_error, Notifications};
use crate::theme::{colors, sizes};

pub struct DashboardState {
    pub debtors:         Vec<DebtorRow>,
    pub students_adult:  usize,
    pub students_minor:  usize,
    pub courses_adult:   usize,
    pub courses_minor:   usize,
    pub teachers_total:  usize,
    pub needs_reload:    bool,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            debtors:        Vec::new(),
            students_adult: 0,
            students_minor: 0,
            courses_adult:  0,
            courses_minor:  0,
            teachers_total: 0,
            needs_reload:   true,
        }
    }
}

pub fn show(
    ui:           &mut egui::Ui,
    student_repo: &Arc<dyn StudentRepo>,
    course_repo:  &Arc<dyn CourseRepo>,
    teacher_repo: &Arc<dyn TeacherRepo>,
    client:       &Arc<Mutex<Client>>,
    state:        &mut DashboardState,
    notifs:       &mut Notifications,
) -> Option<Uuid> {
    if state.needs_reload {
        state.needs_reload = false;
        let uc = DashboardUseCase::new(
            Arc::clone(student_repo),
            Arc::new(EnrollmentPgRepo::new(Arc::clone(client))),
            Arc::new(PaymentPgRepo::new(Arc::clone(client))),
            Arc::clone(course_repo),
            Arc::clone(teacher_repo),
        );
        match uc.load() {
            Ok(DashboardData { debtors, students_adult, students_minor, courses_adult, courses_minor, teachers_total }) => {
                state.debtors        = debtors;
                state.students_adult = students_adult;
                state.students_minor = students_minor;
                state.courses_adult  = courses_adult;
                state.courses_minor  = courses_minor;
                state.teachers_total = teachers_total;
            }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }

    ui.horizontal(|ui| {
        ui.heading("Panel");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui_phosphor::regular::ARROWS_CLOCKWISE).clicked() {
                state.needs_reload = true;
            }
        });
    });
    ui.separator();
    ui.add_space(sizes::SPACING_NORMAL);

    // ── Summary widgets ───────────────────────────────────────────────────────
    ui.columns(3, |cols| {
        cols[0].label(egui::RichText::new("Alumnos").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        cols[0].add_space(sizes::SPACING_SMALL);
        egui::Grid::new("dash_students")
            .num_columns(2)
            .spacing([sizes::SPACING_NORMAL, sizes::SPACING_SMALL])
            .show(&mut cols[0], |ui| {
                ui.label(egui::RichText::new("Adultos").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new(state.students_adult.to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
                ui.label(egui::RichText::new("Menores").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new(state.students_minor.to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
                ui.label(egui::RichText::new("Total").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new((state.students_adult + state.students_minor).to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
            });

        cols[1].label(egui::RichText::new("Cursos").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        cols[1].add_space(sizes::SPACING_SMALL);
        egui::Grid::new("dash_courses")
            .num_columns(2)
            .spacing([sizes::SPACING_NORMAL, sizes::SPACING_SMALL])
            .show(&mut cols[1], |ui| {
                ui.label(egui::RichText::new("Adultos").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new(state.courses_adult.to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
                ui.label(egui::RichText::new("Menores").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new(state.courses_minor.to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
                ui.label(egui::RichText::new("Total").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new((state.courses_adult + state.courses_minor).to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
            });

        cols[2].label(egui::RichText::new("Profesores").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
        cols[2].add_space(sizes::SPACING_SMALL);
        egui::Grid::new("dash_teachers")
            .num_columns(2)
            .spacing([sizes::SPACING_NORMAL, sizes::SPACING_SMALL])
            .show(&mut cols[2], |ui| {
                ui.label(egui::RichText::new("Total").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.label(egui::RichText::new(state.teachers_total.to_string()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                ui.end_row();
            });
    });

    ui.add_space(sizes::SPACING_NORMAL);
    ui.separator();
    ui.add_space(sizes::SPACING_NORMAL);

    // ── Debtors widget ────────────────────────────────────────────────────────
    ui.label(egui::RichText::new("Alumnos con meses pendientes").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
    ui.add_space(sizes::SPACING_SMALL);

    if state.debtors.is_empty() {
        ui.label(egui::RichText::new("No hay alumnos con meses pendientes.").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        return None;
    }

    let mut navigate_to: Option<Uuid> = None;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::auto())
        .header(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut h| {
            h.col(|ui| { ui.label("Alumno"); });
            h.col(|ui| { ui.label("Pendiente"); });
            h.col(|_ui| {});
        })
        .body(|mut body| {
            for row in &state.debtors {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut r| {
                    r.col(|ui| {
                        ui.label(egui::RichText::new(&row.full_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                    });
                    r.col(|ui| {
                        ui.colored_label(colors::RED, &row.pending);
                    });
                    r.col(|ui| {
                        if ui.small_button(egui_phosphor::regular::EYE).clicked() {
                            navigate_to = Some(row.id);
                        }
                    });
                });
            }
        });

    navigate_to
}
