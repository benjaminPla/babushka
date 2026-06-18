use eframe::egui;

use crate::presentation::fmt_dt;
use crate::theme::{colors, sizes};

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

    ui.label(egui::RichText::new("Información").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));

    egui::Grid::new("teacher_details").num_columns(2).spacing([sizes::SPACING_NORMAL, sizes::SPACING_SMALL]).show(ui, |ui| {
        ui.label(egui::RichText::new("Nombre").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&teacher.first_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Apellido").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&teacher.last_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        if let Some(ref email) = teacher.email {
            ui.label(egui::RichText::new("Email").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(email.as_str()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();
        }

        ui.label(egui::RichText::new("Teléfono").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(&teacher.phone).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        if let Some(n) = &teacher.notes {
            ui.label(egui::RichText::new("Notas").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
            ui.label(egui::RichText::new(n.as_str()).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
            ui.end_row();
        }

        ui.label(egui::RichText::new("Creado").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(fmt_dt(teacher.created_at)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();

        ui.label(egui::RichText::new("Editado").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        ui.label(egui::RichText::new(fmt_dt(teacher.updated_at)).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
        ui.end_row();
    });
}
