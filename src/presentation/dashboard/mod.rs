use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use postgres::Client;
use uuid::Uuid;

use crate::application::course::dto::CourseDto;
use crate::application::course::get_all::CourseGetAllUseCase;
use crate::application::course_period::dto::CoursePeriodDto;
use crate::application::course_period::get_by_course::CoursePeriodGetByCourseUseCase;
use crate::application::dashboard::{DashboardData, DashboardUseCase, DebtorRow};
use crate::application::enrollment::create::{EnrollmentCreateInput, EnrollmentCreateUseCase};
use crate::application::student::dto::StudentDto;
use crate::application::student::get_all::StudentGetAllUseCase;
use crate::domain::course::repository::CourseRepo;
use crate::domain::student::repository::StudentRepo;
use crate::domain::teacher::repository::TeacherRepo;
use crate::infrastructure::course::CoursePgRepo;
use crate::infrastructure::course_period::CoursePeriodPgRepo;
use crate::infrastructure::enrollment::EnrollmentPgRepo;
use crate::infrastructure::student::StudentPgRepo;
use crate::presentation::{push_error, push_success, Notifications};
use crate::theme::{colors, sizes};

pub struct DashboardState {
    pub debtors:         Vec<DebtorRow>,
    pub students_adult:  usize,
    pub students_minor:  usize,
    pub courses_adult:   usize,
    pub courses_minor:   usize,
    pub teachers_total:  usize,
    pub needs_reload:    bool,

    // enroll form
    pub show_enroll_form:      bool,
    pub enroll_students:       Vec<StudentDto>,
    pub enroll_courses:        Vec<CourseDto>,
    pub enroll_periods:        Vec<CoursePeriodDto>,
    pub enroll_sel_student:    Option<Uuid>,
    pub enroll_sel_course:     Option<Uuid>,
    pub enroll_sel_period:     Option<Uuid>,
    pub enroll_pricing_type:   String,
    pub enroll_student_filter: String,
    pub enroll_course_filter:  String,
    pub enroll_period_filter:  String,
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
            show_enroll_form:      false,
            enroll_students:       Vec::new(),
            enroll_courses:        Vec::new(),
            enroll_periods:        Vec::new(),
            enroll_sel_student:    None,
            enroll_sel_course:     None,
            enroll_sel_period:     None,
            enroll_pricing_type:   "monthly".into(),
            enroll_student_filter: String::new(),
            enroll_course_filter:  String::new(),
            enroll_period_filter:  String::new(),
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
            if ui.button("+ Inscribir").clicked() {
                if state.enroll_students.is_empty() {
                    if let Ok(students) = StudentGetAllUseCase::new(Arc::new(StudentPgRepo::new(Arc::clone(client)))).execute() {
                        state.enroll_students = students;
                    }
                }
                if state.enroll_courses.is_empty() {
                    if let Ok(courses) = CourseGetAllUseCase::new(Arc::new(CoursePgRepo::new(Arc::clone(client)))).execute() {
                        state.enroll_courses = courses;
                    }
                }
                state.enroll_sel_student    = None;
                state.enroll_sel_course     = None;
                state.enroll_sel_period     = None;
                state.enroll_pricing_type   = "monthly".into();
                state.enroll_student_filter = String::new();
                state.enroll_course_filter  = String::new();
                state.enroll_period_filter  = String::new();
                state.enroll_periods        = Vec::new();
                state.show_enroll_form      = true;
            }
        });
    });
    ui.separator();
    ui.add_space(sizes::SPACING_NORMAL);

    // ── Enroll form modal ─────────────────────────────────────────────────────
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
                    ui.label(egui::RichText::new("Alumno").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("dash_enroll_student")
                        .width(ui.available_width())
                        .selected_text(
                            state.enroll_sel_student
                                .and_then(|id| state.enroll_students.iter().find(|s| s.id == id))
                                .map(|s| format!("{} {}", s.first_name, s.last_name))
                                .unwrap_or_else(|| "Seleccionar...".into()),
                        )
                        .show_ui(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.enroll_student_filter)
                                .id(egui::Id::new("dash_enroll_student_filter")));
                            let sf = state.enroll_student_filter.to_lowercase();
                            let students: Vec<_> = state.enroll_students.iter()
                                .filter(|s| {
                                    let name = format!("{} {}", s.first_name, s.last_name).to_lowercase();
                                    sf.is_empty() || name.contains(&sf)
                                })
                                .cloned()
                                .collect();
                            for s in &students {
                                let label = format!("{} {}", s.first_name, s.last_name);
                                ui.selectable_value(&mut state.enroll_sel_student, Some(s.id), label);
                            }
                        });
                    ui.add_space(sizes::SPACING_SMALL);

                    ui.label(egui::RichText::new("Curso").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    egui::ComboBox::from_id_salt("dash_enroll_course")
                        .width(ui.available_width())
                        .selected_text(
                            state.enroll_sel_course
                                .and_then(|id| state.enroll_courses.iter().find(|c| c.id == id))
                                .map(|c| c.name.clone())
                                .unwrap_or_else(|| "Seleccionar...".into()),
                        )
                        .show_ui(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.enroll_course_filter)
                                .id(egui::Id::new("dash_enroll_course_filter")));
                            let cf = state.enroll_course_filter.to_lowercase();
                            let courses: Vec<_> = state.enroll_courses.iter()
                                .filter(|c| cf.is_empty() || c.name.to_lowercase().contains(&cf))
                                .cloned()
                                .collect();
                            for c in &courses {
                                if ui.selectable_value(&mut state.enroll_sel_course, Some(c.id), &c.name).clicked() {
                                    if let Ok(ps) = CoursePeriodGetByCourseUseCase::new(
                                        Arc::new(CoursePeriodPgRepo::new(Arc::clone(client)))
                                    ).execute(c.id) {
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
                    egui::ComboBox::from_id_salt("dash_enroll_period")
                        .width(ui.available_width())
                        .selected_text(
                            state.enroll_sel_period
                                .and_then(|id| state.enroll_periods.iter().find(|p| p.id == id))
                                .map(|p| p.label.clone())
                                .unwrap_or_else(|| "Seleccionar...".into()),
                        )
                        .show_ui(ui, |ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.enroll_period_filter)
                                .id(egui::Id::new("dash_enroll_period_filter")));
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
                                state.show_enroll_form = false;
                            }
                            if ui.button("Inscribir").clicked() {
                                match (state.enroll_sel_student, state.enroll_sel_period) {
                                    (Some(student_id), Some(period_id)) => {
                                        match EnrollmentCreateUseCase::new(
                                            Arc::new(EnrollmentPgRepo::new(Arc::clone(client)))
                                        ).execute(EnrollmentCreateInput {
                                            student_id,
                                            course_period_id: period_id,
                                            pricing_type: state.enroll_pricing_type.clone(),
                                        }) {
                                            Ok(_) => {
                                                push_success(notifs, "Alumno inscrito");
                                                state.show_enroll_form = false;
                                                state.needs_reload     = true;
                                            }
                                            Err(e) => push_error(notifs, e.to_string()),
                                        }
                                    }
                                    (None, _) => push_error(notifs, "Seleccionar un alumno"),
                                    (_, None) => push_error(notifs, "Seleccionar un período"),
                                }
                            }
                        });
                    });
                });
            });
    }

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

    // ── Debtors table ─────────────────────────────────────────────────────────
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
