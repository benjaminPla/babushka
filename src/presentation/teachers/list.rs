use std::sync::Arc;

use eframe::egui;
use uuid::Uuid;

use crate::application::teacher::delete::TeacherDeleteUseCase;
use crate::domain::teacher::repository::TeacherRepo;
use crate::presentation::{confirm_delete_modal, fmt_dt, push_error, push_success, Notifications};
use crate::presentation::table::{self, Column};

use super::{Mode, TeachersState, clear_form};

enum Action { View, Edit, Delete }

pub fn show(ui: &mut egui::Ui, repo: &Arc<dyn TeacherRepo>, state: &mut TeachersState, notifs: &mut Notifications) {
    ui.horizontal(|ui| {
        ui.heading("Profesores");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Nuevo").clicked() {
                clear_form(state);
                state.mode = Mode::Create;
            }
        });
    });
    ui.separator();

    let fn_f = state.filter_first_name.to_lowercase();
    let ln_f = state.filter_last_name.to_lowercase();
    let em_f = state.filter_email.to_lowercase();

    let visible: Vec<_> = state.teachers.iter()
        .filter(|t| {
            (fn_f.is_empty() || t.first_name.to_lowercase().contains(&fn_f)) &&
            (ln_f.is_empty() || t.last_name.to_lowercase().contains(&ln_f))  &&
            (em_f.is_empty() || t.email.to_lowercase().contains(&em_f))
        })
        .cloned()
        .collect();

    let mut action: Option<(Action, Uuid)> = None;

    table::builder(ui)
        .column(Column::remainder().at_least(80.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(100.0))
        .column(Column::exact(110.0))
        .column(Column::exact(160.0))
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head_filter(ui, "Nombre",   &mut state.filter_first_name));
            h.col(|ui| table::head_filter(ui, "Apellido", &mut state.filter_last_name));
            h.col(|ui| table::head_filter(ui, "Email",    &mut state.filter_email));
            h.col(|ui| table::head(ui, "Teléfono"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for t in &visible {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&t.first_name); });
                    row.col(|ui| { ui.label(&t.last_name); });
                    row.col(|ui| { ui.label(&t.email); });
                    row.col(|ui| { ui.label(&t.phone); });
                    row.col(|ui| {
                        if ui.small_button("Ver").clicked()      { action = Some((Action::View,   t.id)); }
                        if ui.small_button("Editar").clicked()   { action = Some((Action::Edit,   t.id)); }
                        if ui.small_button("Eliminar").clicked() { action = Some((Action::Delete, t.id)); }
                    });
                });
            }
        });

    if let Some((act, id)) = action {
        match act {
            Action::View => {
                state.viewing_id = Some(id);
                state.mode       = Mode::View;
            }
            Action::Edit => {
                if let Some(t) = state.teachers.iter().find(|t| t.id == id) {
                    state.first_name = t.first_name.clone();
                    state.last_name  = t.last_name.clone();
                    state.email      = t.email.clone();
                    state.phone      = t.phone.clone();
                    state.notes      = t.notes.clone().unwrap_or_default();
                    state.created_at = fmt_dt(t.created_at);
                    state.updated_at = fmt_dt(t.updated_at);
                    state.editing_id = Some(id);
                    state.mode       = Mode::Edit;
                }
            }
            Action::Delete => { state.confirm_delete = Some(id); }
        }
    }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete) {
        match TeacherDeleteUseCase::new(Arc::clone(repo)).execute(id) {
            Ok(_)  => { state.needs_reload = true; push_success(notifs, "Profesor eliminado"); }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
