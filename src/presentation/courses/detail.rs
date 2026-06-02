use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        enrollment::{
            create::{EnrollmentCreateInput, EnrollmentCreateUseCase},
            delete::EnrollmentDeleteUseCase,
            get_by_course::EnrollmentGetByCourseUseCase,
            update_status::{EnrollmentUpdateStatusInput, EnrollmentUpdateStatusUseCase},
        },
        student::get_all::StudentGetAllUseCase,
    },
    domain::enrollment::EnrollmentStatus,
};

use crate::presentation::confirm_delete_modal;

use super::{
    CoursesState, Mode,
    make_course_repo, make_enrollment_repo, make_student_repo,
};

enum Action { Open, ChangeStatus(EnrollmentStatus), Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState) {
    let course = match &state.selected_course {
        Some(c) => c.clone(),
        None    => { state.mode = Mode::List; return; }
    };

    if state.needs_reload_enrollments {
        match EnrollmentGetByCourseUseCase::new(make_enrollment_repo(client)).execute(course.id) {
            Ok(enrollments) => { state.enrollments = enrollments; state.needs_reload_enrollments = false; }
            Err(e)          => { state.error = Some(e.to_string()); }
        }
    }

    ui.horizontal(|ui| {
        if ui.button("← Cursos").clicked() {
            state.mode             = Mode::List;
            state.selected_course  = None;
            state.enrollments      = Vec::new();
            state.error            = None;
        }
        ui.heading(format!("{} — {}", course.name, course.age_group.label()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("Precio: ${} · Cap: {}/{}", super::format_price(course.price_cents), course.enrolled, course.capacity));
        });
    });
    ui.label(format!("Profesor: {}", course.teacher_name));
    ui.separator();

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
        ui.separator();
    }

    // Add enrollment section
    if state.mode == Mode::AddEnrollment {
        ui.heading("Inscribir alumno");
        if state.available_students.is_empty() {
            ui.label("No hay alumnos disponibles para este grupo.");
        } else {
            egui::ComboBox::from_id_salt("student_combo")
                .selected_text(
                    state.selected_student_id
                        .and_then(|id| state.available_students.iter().find(|s| s.id == id))
                        .map(|s| format!("{} {}", s.first_name, s.last_name))
                        .unwrap_or_else(|| "Seleccionar alumno...".into()),
                )
                .show_ui(ui, |ui| {
                    for s in &state.available_students {
                        let label = format!("{} {}", s.first_name, s.last_name);
                        ui.selectable_value(&mut state.selected_student_id, Some(s.id), label);
                    }
                });

            ui.horizontal(|ui| {
                if ui.button("Inscribir").clicked() {
                    if let Some(student_id) = state.selected_student_id {
                        let result = EnrollmentCreateUseCase::new(
                            make_enrollment_repo(client),
                            make_course_repo(client),
                            make_student_repo(client),
                        ).execute(EnrollmentCreateInput { student_id, course_id: course.id });

                        match result {
                            Ok(_) => {
                                state.needs_reload             = true;
                                state.needs_reload_enrollments = true;
                                state.selected_student_id      = None;
                                state.mode                     = Mode::Detail;
                                state.error                    = None;
                            }
                            Err(e) => state.error = Some(e.to_string()),
                        }
                    }
                }
                if ui.button("Cancelar").clicked() {
                    state.mode                = Mode::Detail;
                    state.selected_student_id = None;
                    state.error               = None;
                }
            });
        }
        ui.separator();
    } else if ui.button("+ Inscribir alumno").clicked() {
        // load available students (matching age_group, not yet enrolled)
        let enrolled_ids: std::collections::HashSet<Uuid> =
            state.enrollments.iter().map(|e| e.student_id).collect();
        if let Ok(all_students) = StudentGetAllUseCase::new(make_student_repo(client)).execute() {
            state.available_students = all_students
                .into_iter()
                .filter(|s| s.age_group == course.age_group && !enrolled_ids.contains(&s.id))
                .collect();
        }
        state.selected_student_id = None;
        state.error               = None;
        state.mode                = Mode::AddEnrollment;
    }

    // Enrollment list
    let mut action: Option<(Action, Uuid)> = None;

    egui::Grid::new("enrollments_grid")
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Alumno");
            ui.strong("Estado");
            ui.strong("Último pago");
            ui.strong("");
            ui.end_row();

            for e in &state.enrollments {
                ui.label(&e.student_name);
                ui.label(e.status.label());

                match &e.latest_payment {
                    None => { ui.label("—"); }
                    Some(s) => {
                        let (text, color) = match s.as_str() {
                            "paid"    => ("✓ Pagado",    egui::Color32::GREEN),
                            "overdue" => ("✗ Vencido",   egui::Color32::RED),
                            _         => ("⚠ Pendiente", egui::Color32::YELLOW),
                        };
                        ui.colored_label(color, text);
                    }
                }

                ui.horizontal(|ui| {
                    if ui.small_button("Pagos").clicked() {
                        action = Some((Action::Open, e.id));
                    }
                    egui::ComboBox::from_id_salt(format!("status_{}", e.id))
                        .selected_text(e.status.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut (),
                                (),
                                EnrollmentStatus::Active.label(),
                            );
                            if ui.selectable_label(false, EnrollmentStatus::Active.label()).clicked() {
                                action = Some((Action::ChangeStatus(EnrollmentStatus::Active), e.id));
                            }
                            if ui.selectable_label(false, EnrollmentStatus::Dropped.label()).clicked() {
                                action = Some((Action::ChangeStatus(EnrollmentStatus::Dropped), e.id));
                            }
                            if ui.selectable_label(false, EnrollmentStatus::Completed.label()).clicked() {
                                action = Some((Action::ChangeStatus(EnrollmentStatus::Completed), e.id));
                            }
                        });
                    if ui.small_button("🗑").clicked() {
                        action = Some((Action::Delete, e.id));
                    }
                });
                ui.end_row();
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Open => {
                if let Some(e) = state.enrollments.iter().find(|e| e.id == id) {
                    state.selected_enrollment  = Some(e.clone());
                    state.needs_reload_payments = true;
                    state.payment_amount       = super::format_price(course.price_cents);
                    state.payment_due_date     = String::new();
                    state.payment_notes        = String::new();
                    state.error                = None;
                    state.mode                 = Mode::EnrollmentDetail;
                }
            }
            Action::ChangeStatus(new_status) => {
                match EnrollmentUpdateStatusUseCase::new(make_enrollment_repo(client))
                    .execute(EnrollmentUpdateStatusInput { id, status: new_status }) {
                    Ok(_)  => { state.needs_reload_enrollments = true; state.needs_reload = true; }
                    Err(e) => state.error = Some(e.to_string()),
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match EnrollmentDeleteUseCase::new(make_enrollment_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload_enrollments = true; state.needs_reload = true; }
            Err(e) => state.error = Some(e.to_string()),
        }
    }
}
