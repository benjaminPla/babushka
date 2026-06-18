use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::MAGNIFYING_GLASS;
use postgres::Client;
use uuid::Uuid;

use crate::application::course::delete::CourseDeleteUseCase;
use crate::domain::course::repository::CourseRepo;
use crate::presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications};
use crate::theme::sizes;

use super::{clear_course_form, format_price, CoursesState, Mode};

enum Action { Open, Edit, Delete }

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn CourseRepo>, client: &Arc<Mutex<Client>>, state: &mut CoursesState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Cursos");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Nuevo").clicked() {
                clear_course_form(state);
                if let Ok(ts) = crate::application::teacher::get_all::TeacherGetAllUseCase::new(
                    super::make_teacher_repo(client)
                ).execute() {
                    state.teachers = ts;
                }
                state.show_modal = true;
            }
        });
    });
    ui.separator();

    let name_f = state.filter_name.to_lowercase();

    let visible: Vec<_> = state.courses.iter()
        .filter(|c| name_f.is_empty() || c.name.to_lowercase().contains(&name_f))
        .cloned()
        .collect();

    let mut action: Option<(Action, Uuid)> = None;

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
            header.col(|ui| { ui.add(egui::TextEdit::singleline(&mut state.filter_name).hint_text(format!("{MAGNIFYING_GLASS} Nombre"))); });
            header.col(|ui| { ui.label("Grupo"); });
            header.col(|ui| { ui.label("Cap."); });
            header.col(|ui| { ui.label("Mensual"); });
            header.col(|ui| { ui.label("Por clase"); });
            header.col(|ui| { ui.label("Acciones"); });
        })
        .body(|mut body| {
            for c in &visible {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut row| {
                    row.col(|ui| { ui.label(&c.name); });
                    row.col(|ui| { ui.label(c.age_group.label()); });
                    row.col(|ui| { ui.label(c.capacity.to_string()); });
                    row.col(|ui| { ui.label(format!("{} / {}", format_price(c.month_price_cash_cents), format_price(c.month_price_transfer_cents))); });
                    row.col(|ui| { ui.label(format!("{} / {}", format_price(c.class_price_cash_cents), format_price(c.class_price_transfer_cents))); });
                    row.col(|ui| {
                        if ui.small_button(egui_phosphor::regular::EYE).clicked()           { action = Some((Action::Open,   c.id)); }
                        if ui.small_button(egui_phosphor::regular::PENCIL_SIMPLE).clicked() { action = Some((Action::Edit,   c.id)); }
                        if ui.small_button(egui_phosphor::regular::TRASH).clicked()         { action = Some((Action::Delete, c.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Open => {
                if let Some(c) = state.courses.iter().find(|c| c.id == id) {
                    state.selected_course       = Some(c.clone());
                    state.needs_reload_periods  = true;
                    state.needs_reload_students = true;
                    state.mode                  = Mode::Detail;
                }
            }
            Action::Edit => {
                if let Some(c) = state.courses.iter().find(|c| c.id == id) {
                    state.editing_id   = Some(id);
                    state.name         = c.name.clone();
                    state.teacher_id   = Some(c.teacher_id);
                    state.age_group    = c.age_group;
                    state.capacity     = c.capacity.to_string();
                    state.month_price_cash     = (c.month_price_cash_cents     as f64 / 100.0).to_string();
                    state.month_price_transfer = (c.month_price_transfer_cents as f64 / 100.0).to_string();
                    state.class_price_cash     = (c.class_price_cash_cents     as f64 / 100.0).to_string();
                    state.class_price_transfer = (c.class_price_transfer_cents as f64 / 100.0).to_string();
                    state.course_notes = c.notes.clone().unwrap_or_default();
                    state.created_at   = fmt_dt(c.created_at);
                    state.updated_at   = fmt_dt(c.updated_at);
                    if let Ok(ts) = crate::application::teacher::get_all::TeacherGetAllUseCase::new(
                        super::make_teacher_repo(client)
                    ).execute() {
                        state.teachers = ts;
                    }
                    state.show_modal = true;
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match CourseDeleteUseCase::new(Arc::clone(repo)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Curso eliminado"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
