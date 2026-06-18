use std::sync::Arc;

use eframe::egui;

use crate::application::course::{
    create::{CourseCreateInput, CourseCreateUseCase},
    update::{CourseUpdateInput, CourseUpdateUseCase},
};
use crate::domain::{course::repository::CourseRepo, shared::value_objects::age_group::AgeGroup};
use crate::presentation::{push_error, push_success, Notifications};
use crate::theme::{colors, sizes};

use super::{clear_course_form, parse_price, CoursesState};

pub fn show(ctx: &egui::Context, repo: &Arc<dyn CourseRepo>, state: &mut CoursesState, notifs: &mut Notifications) {
    if !state.show_modal { return; }

    let is_edit = state.editing_id.is_some();
    let title   = if is_edit { "Editar Curso" } else { "Nuevo Curso" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::new()
            .fill(colors::BLACK)
            .stroke(egui::Stroke::new(sizes::STROKE_SMALL, colors::WHITE))
            .inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL))
        )
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.label(egui::RichText::new("Nombre").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.name));
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Profesor").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                egui::ComboBox::from_id_salt("course_form_teacher")
                    .width(ui.available_width())
                    .selected_text(
                        state.teacher_id
                            .and_then(|id| state.teachers.iter().find(|t| t.id == id))
                            .map(|t| format!("{} {}", t.first_name, t.last_name))
                            .unwrap_or_else(|| "Seleccionar...".into()),
                    )
                    .show_ui(ui, |ui| {
                        for t in &state.teachers {
                            let label = format!("{} {}", t.first_name, t.last_name);
                            ui.selectable_value(&mut state.teacher_id, Some(t.id), label);
                        }
                    });
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Grupo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.horizontal(|ui| {
                    ui.radio_value(&mut state.age_group, AgeGroup::Adult, AgeGroup::Adult.label());
                    ui.radio_value(&mut state.age_group, AgeGroup::Minor, AgeGroup::Minor.label());
                });
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Capacidad").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.capacity));
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Mensual efectivo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.month_price_cash));
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Mensual transferencia").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.month_price_transfer));
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Clase efectivo").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.class_price_cash));
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Clase transferencia").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::singleline(&mut state.class_price_transfer));
                ui.add_space(sizes::SPACING_SMALL);

                ui.label(egui::RichText::new("Notas").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                ui.add_sized([ui.available_width(), 0.0], egui::TextEdit::multiline(&mut state.course_notes).desired_rows(3));

                if is_edit {
                    ui.add_space(sizes::SPACING_SMALL);
                    ui.label(egui::RichText::new(format!("Creado: {}", state.created_at)).color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                    ui.label(egui::RichText::new(format!("Editado: {}", state.updated_at)).color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
                }

                ui.add_space(sizes::SPACING_NORMAL);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Guardar").clicked() {
                            let teacher_id = match state.teacher_id {
                                Some(id) => id,
                                None     => { push_error(notifs, "Seleccionar un profesor"); return; }
                            };
                            let capacity = match state.capacity.trim().parse::<i16>() {
                                Ok(v)  => v,
                                Err(_) => { push_error(notifs, "Capacidad inválida"); return; }
                            };
                            let month_price_cash_cents = match parse_price(&state.month_price_cash) {
                                Some(v) => v,
                                None    => { push_error(notifs, "Mensual efectivo inválido"); return; }
                            };
                            let month_price_transfer_cents = match parse_price(&state.month_price_transfer) {
                                Some(v) => v,
                                None    => { push_error(notifs, "Mensual transferencia inválido"); return; }
                            };
                            let class_price_cash_cents = match parse_price(&state.class_price_cash) {
                                Some(v) => v,
                                None    => { push_error(notifs, "Clase efectivo inválido"); return; }
                            };
                            let class_price_transfer_cents = match parse_price(&state.class_price_transfer) {
                                Some(v) => v,
                                None    => { push_error(notifs, "Clase transferencia inválido"); return; }
                            };
                            let notes = if state.course_notes.trim().is_empty() { None } else { Some(state.course_notes.clone()) };

                            let result = if is_edit {
                                CourseUpdateUseCase::new(Arc::clone(repo)).execute(CourseUpdateInput {
                                    id: state.editing_id.unwrap(),
                                    teacher_id,
                                    name: state.name.clone(),
                                    age_group: state.age_group,
                                    capacity,
                                    month_price_cash_cents,
                                    month_price_transfer_cents,
                                    class_price_cash_cents,
                                    class_price_transfer_cents,
                                    notes,
                                })
                            } else {
                                CourseCreateUseCase::new(Arc::clone(repo)).execute(CourseCreateInput {
                                    teacher_id,
                                    name: state.name.clone(),
                                    age_group: state.age_group,
                                    capacity,
                                    month_price_cash_cents,
                                    month_price_transfer_cents,
                                    class_price_cash_cents,
                                    class_price_transfer_cents,
                                    notes,
                                })
                            };

                            match result {
                                Ok(_) => {
                                    push_success(notifs, "Curso guardado");
                                    state.needs_reload = true;
                                    clear_course_form(state);
                                }
                                Err(e) => push_error(notifs, e.to_string()),
                            }
                        }
                        if ui.button("Cancelar").clicked() {
                            clear_course_form(state);
                        }
                    });
                });
            });
        });
}
