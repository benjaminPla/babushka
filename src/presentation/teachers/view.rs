use eframe::egui;

use crate::presentation::{fmt_dt, section_header};

use super::{Mode, TeachersState};

pub fn show(ui: &mut egui::Ui, state: &mut TeachersState) {
    let Some(teacher) = state.viewing_id
        .and_then(|id| state.teachers.iter().find(|t| t.id == id))
        .cloned()
    else {
        state.mode = Mode::List;
        return;
    };

    if ui.button("<- Volver").clicked() {
        state.viewing_id = None;
        state.mode       = Mode::List;
    }
    ui.separator();

    section_header(ui, "Información");
    ui.heading(format!("{} {}", teacher.first_name, teacher.last_name));
    egui::Grid::new("teacher_view").num_columns(2).spacing([16.0, 2.0]).show(ui, |ui| {
        ui.label(egui::RichText::new("Email").color(crate::theme::colors::TEXT_MUTED));
        ui.label(&teacher.email);
        ui.end_row();
        ui.label(egui::RichText::new("Teléfono").color(crate::theme::colors::TEXT_MUTED));
        ui.label(&teacher.phone);
        ui.end_row();
        if let Some(n) = &teacher.notes {
            ui.label(egui::RichText::new("Notas").color(crate::theme::colors::TEXT_MUTED));
            ui.label(n.as_str());
            ui.end_row();
        }
        ui.label(egui::RichText::new("Creado").color(crate::theme::colors::TEXT_MUTED));
        ui.label(fmt_dt(teacher.created_at));
        ui.end_row();
        ui.label(egui::RichText::new("Editado").color(crate::theme::colors::TEXT_MUTED));
        ui.label(fmt_dt(teacher.updated_at));
        ui.end_row();
    });
}
