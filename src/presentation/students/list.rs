use std::sync::Arc;

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::MAGNIFYING_GLASS;
use uuid::Uuid;

use crate::application::student::delete::StudentDeleteUseCase;
use crate::domain::student::repository::StudentRepo;
use crate::presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications};
use crate::theme::sizes;

use super::{StudentsState, clear_form};

enum Action { Open, Edit, Delete }

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn StudentRepo>, state: &mut StudentsState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Alumnos");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Nuevo").clicked() {
                clear_form(state);
                state.show_modal = true;
            }
        });
    });
    ui.separator();

    let mut action: Option<(Action, Uuid)> = None;

    let fn_f = state.filter_first_name.to_lowercase();
    let ln_f = state.filter_last_name.to_lowercase();
    let em_f = state.filter_email.to_lowercase();

    let visible: Vec<_> = state.students.iter()
        .filter(|s| {
            (fn_f.is_empty() || s.first_name.to_lowercase().contains(&fn_f)) &&
            (ln_f.is_empty() || s.last_name.to_lowercase().contains(&ln_f))  &&
            (em_f.is_empty() || s.email.to_lowercase().contains(&em_f))
        })
        .cloned()
        .collect();

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
            header.col(|ui| { ui.add(egui::TextEdit::singleline(&mut state.filter_first_name).hint_text(format!("{MAGNIFYING_GLASS} Nombre"))); });
            header.col(|ui| { ui.add(egui::TextEdit::singleline(&mut state.filter_last_name).hint_text(format!("{MAGNIFYING_GLASS} Apellido"))); });
            header.col(|ui| { ui.add(egui::TextEdit::singleline(&mut state.filter_email).hint_text(format!("{MAGNIFYING_GLASS} Email"))); });
            header.col(|ui| { ui.label("Teléfono"); });
            header.col(|ui| { ui.label("Tipo"); });
            header.col(|ui| { ui.label("Acciones"); });
        })
        .body(|mut body| {
            for s in &visible {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut row| {
                    row.col(|ui| { ui.label(&s.first_name); });
                    row.col(|ui| { ui.label(&s.last_name); });
                    row.col(|ui| { ui.label(&s.email); });
                    row.col(|ui| { ui.label(&s.phone); });
                    row.col(|ui| { ui.label(s.age_group.label()); });
                    row.col(|ui| {
                        if ui.small_button(egui_phosphor::regular::EYE).clicked()           { action = Some((Action::Open,   s.id)); }
                        if ui.small_button(egui_phosphor::regular::PENCIL_SIMPLE).clicked() { action = Some((Action::Edit,   s.id)); }
                        if ui.small_button(egui_phosphor::regular::TRASH).clicked()         { action = Some((Action::Delete, s.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::Open => {
                if let Some(s) = state.students.iter().find(|s| s.id == id).cloned() {
                    super::clear_detail_state(state);
                    state.selected_student    = Some(s);
                    state.needs_reload_ledger = true;
                    state.mode               = super::Mode::Detail;
                }
            }
            Action::Edit => {
                if let Some(s) = state.students.iter().find(|s| s.id == id) {
                    state.age_group  = s.age_group;
                    state.first_name = s.first_name.clone();
                    state.last_name  = s.last_name.clone();
                    state.email      = s.email.clone();
                    state.phone      = s.phone.clone();
                    state.notes      = s.notes.clone().unwrap_or_default();
                    state.created_at = fmt_dt(s.created_at);
                    state.updated_at = fmt_dt(s.updated_at);
                    state.editing_id = Some(id);
                    state.show_modal = true;
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match StudentDeleteUseCase::new(Arc::clone(repo)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Alumno eliminado"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
