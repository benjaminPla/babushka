use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::{
        course::get_all::CourseGetAllUseCase,
        enrollment::{
            create::{EnrollmentCreateInput, EnrollmentCreateUseCase},
            delete::EnrollmentDeleteUseCase,
            update_status::{EnrollmentUpdateStatusInput, EnrollmentUpdateStatusUseCase},
        },
        student::get_all::StudentGetAllUseCase,
    },
    domain::enrollment::EnrollmentStatus,
    presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications},
};

use super::{EnrollmentsState, Mode, make_course_repo, make_enrollment_repo, make_student_repo, status_label_color};

enum Action { ChangeStatus(EnrollmentStatus), Delete }

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut EnrollmentsState, notifs: &mut Notifications) {
    // ── Header ───────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.heading("Inscripciones");
        if state.mode == Mode::List && ui.button("+ Nueva").clicked() {
            if let Ok(ss) = StudentGetAllUseCase::new(make_student_repo(client)).execute() {
                state.students = ss;
            }
            if let Ok(cs) = CourseGetAllUseCase::new(make_course_repo(client)).execute() {
                state.courses = cs;
            }
            state.sel_student = None;
            state.sel_course  = None;
            state.mode        = Mode::Create;
        }
    });
    ui.separator();

    // ── Create form ──────────────────────────────────────────────────────────
    if state.mode == Mode::Create {
        egui::Grid::new("enrollment_create").num_columns(2).show(ui, |ui| {
            ui.label("Alumno");
            egui::ComboBox::from_id_salt("enr_student")
                .selected_text(
                    state.sel_student
                        .and_then(|id| state.students.iter().find(|s| s.id == id))
                        .map(|s| format!("{} {}", s.first_name, s.last_name))
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    for s in &state.students {
                        let label = format!("{} {}", s.first_name, s.last_name);
                        ui.selectable_value(&mut state.sel_student, Some(s.id), label);
                    }
                });
            ui.end_row();

            ui.label("Curso");
            egui::ComboBox::from_id_salt("enr_course")
                .selected_text(
                    state.sel_course
                        .and_then(|id| state.courses.iter().find(|c| c.id == id))
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "Seleccionar...".into()),
                )
                .show_ui(ui, |ui| {
                    for c in &state.courses {
                        ui.selectable_value(&mut state.sel_course, Some(c.id), &c.name);
                    }
                });
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("Guardar").clicked() {
                match (state.sel_student, state.sel_course) {
                    (Some(student_id), Some(course_id)) => {
                        let result = EnrollmentCreateUseCase::new(
                            make_enrollment_repo(client),
                            make_course_repo(client),
                            make_student_repo(client),
                        ).execute(EnrollmentCreateInput { student_id, course_id });

                        match result {
                            Ok(_) => {
                                push_success(notifs, "Alumno inscrito");
                                state.needs_reload = true;
                                state.mode         = Mode::List;
                            }
                            Err(e) => push_error(notifs, e.to_string()),
                        }
                    }
                    _ => push_error(notifs, "Seleccionar alumno y curso"),
                }
            }
            if ui.button("Cancelar").clicked() { state.mode = Mode::List; }
        });
        ui.separator();
    }

    // ── List ─────────────────────────────────────────────────────────────────
    let mut action: Option<(Action, Uuid)> = None;

    egui::Grid::new("enrollments_grid")
        .num_columns(6)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Alumno");
            ui.strong("Curso");
            ui.strong("Estado");
            ui.strong("Último pago");
            ui.strong("Inscripto");
            ui.strong("");
            ui.end_row();

            for e in &state.enrollments {
                ui.label(&e.student_name);
                ui.label(&e.course_name);
                ui.colored_label(status_label_color(&e.status), e.status.label());

                match e.latest_payment.as_deref() {
                    None          => { ui.label("—"); }
                    Some("paid")    => { ui.colored_label(egui::Color32::GREEN,  "✓ Pagado"); }
                    Some("overdue") => { ui.colored_label(egui::Color32::RED,    "✗ Vencido"); }
                    _               => { ui.colored_label(egui::Color32::YELLOW, "⚠ Pendiente"); }
                }

                ui.label(fmt_dt(e.enrolled_at));

                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt(format!("enr_status_{}", e.id))
                        .selected_text(e.status.label())
                        .show_ui(ui, |ui| {
                            for s in [EnrollmentStatus::Active, EnrollmentStatus::Dropped, EnrollmentStatus::Completed] {
                                if ui.selectable_label(e.status == s, s.label()).clicked() {
                                    action = Some((Action::ChangeStatus(s), e.id));
                                }
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
            Action::ChangeStatus(new_status) => {
                match EnrollmentUpdateStatusUseCase::new(make_enrollment_repo(client))
                    .execute(EnrollmentUpdateStatusInput { id, status: new_status }) {
                    Ok(_)  => state.needs_reload = true,
                    Err(e) => push_error(notifs, e.to_string()),
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match EnrollmentDeleteUseCase::new(make_enrollment_repo(client)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Inscripción eliminada"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
