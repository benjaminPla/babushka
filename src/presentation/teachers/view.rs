use eframe::egui;

use crate::presentation::fmt_dt;

use super::{Mode, TeachersState};

pub fn show(ui: &mut egui::Ui, state: &mut TeachersState) {
    let Some(teacher) = state.viewing_id
        .and_then(|id| state.teachers.iter().find(|t| t.id == id))
        .cloned()
    else {
        state.mode = Mode::List;
        return;
    };

    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            state.viewing_id = None;
            state.mode       = Mode::List;
        }
        ui.heading(format!("{} {}", teacher.first_name, teacher.last_name));
    });
    ui.separator();

    egui::Grid::new("teacher_view").num_columns(2).show(ui, |ui| {
        ui.label("Nombre");    ui.label(&teacher.first_name);                          ui.end_row();
        ui.label("Apellido");  ui.label(&teacher.last_name);                           ui.end_row();
        ui.label("Email");     ui.label(&teacher.email);                               ui.end_row();
        ui.label("Teléfono");  ui.label(&teacher.phone);                               ui.end_row();
        ui.label("Notas");     ui.label(teacher.notes.as_deref().unwrap_or("—"));      ui.end_row();
        ui.label("Creado");    ui.label(fmt_dt(teacher.created_at));                   ui.end_row();
        ui.label("Editado");   ui.label(fmt_dt(teacher.updated_at));                   ui.end_row();
    });
}
