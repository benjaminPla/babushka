use std::sync::{Arc, Mutex};

use eframe::egui;
use postgres::Client;

use crate::{
    application::course::{
        create::{CourseCreateInput, CourseCreateUseCase},
        update::{CourseUpdateInput, CourseUpdateUseCase},
    },
    domain::shared::value_objects::age_group::AgeGroup,
    presentation::{push_error, push_success, Notifications},
};

use super::{CoursesState, Mode, clear_course_form, make_course_repo, parse_price};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState, notifs: &mut Notifications) {
    let title = if state.mode == Mode::CreateCourse { "Nuevo Curso" } else { "Editar Curso" };

    ui.horizontal(|ui| {
        if ui.button("← Volver").clicked() {
            state.mode = Mode::List;
            clear_course_form(state);
        }
        ui.heading(title);
    });
    ui.separator();

    egui::Grid::new("course_form").num_columns(2).show(ui, |ui| {
        ui.label("Nombre");
        ui.text_edit_singleline(&mut state.name);
        ui.end_row();

        ui.label("Profesor");
        egui::ComboBox::from_id_salt("teacher_combo")
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
        ui.end_row();

        ui.label("Grupo");
        ui.horizontal(|ui| {
            ui.radio_value(&mut state.age_group, AgeGroup::Adult, AgeGroup::Adult.label());
            ui.radio_value(&mut state.age_group, AgeGroup::Minor, AgeGroup::Minor.label());
        });
        ui.end_row();

        ui.label("Capacidad");
        ui.text_edit_singleline(&mut state.capacity);
        ui.end_row();

        ui.label("Precio mensual");
        ui.text_edit_singleline(&mut state.price);
        ui.end_row();

        ui.label("Precio por clase");
        ui.text_edit_singleline(&mut state.class_price);
        ui.end_row();

        ui.label("Notas");
        ui.text_edit_multiline(&mut state.course_notes);
        ui.end_row();

        if state.mode == Mode::EditCourse {
            ui.label("Creado");  ui.label(&state.created_at); ui.end_row();
            ui.label("Editado"); ui.label(&state.updated_at); ui.end_row();
        }
    });

    if ui.button("Guardar").clicked() {
        let teacher_id = match state.teacher_id {
            Some(id) => id,
            None     => { push_error(notifs, "Seleccionar un profesor"); return; }
        };
        let capacity = match state.capacity.trim().parse::<i16>() {
            Ok(v)  => v,
            Err(_) => { push_error(notifs, "Capacidad inválida"); return; }
        };
        let price_cents = match parse_price(&state.price) {
            Some(v) => v,
            None    => { push_error(notifs, "Precio mensual inválido"); return; }
        };
        let class_price_cents = match parse_price(&state.class_price) {
            Some(v) => v,
            None    => { push_error(notifs, "Precio por clase inválido"); return; }
        };
        let notes = if state.course_notes.trim().is_empty() { None } else { Some(state.course_notes.clone()) };

        let result = match state.mode {
            Mode::CreateCourse => CourseCreateUseCase::new(make_course_repo(client)).execute(CourseCreateInput {
                teacher_id, name: state.name.clone(), age_group: state.age_group.clone(),
                capacity, price_cents, class_price_cents, notes,
            }),
            Mode::EditCourse => CourseUpdateUseCase::new(make_course_repo(client)).execute(CourseUpdateInput {
                id: state.editing_id.unwrap(), teacher_id, name: state.name.clone(),
                age_group: state.age_group.clone(), capacity, price_cents, class_price_cents, notes,
            }),
            _ => unreachable!(),
        };

        match result {
            Ok(_)  => {
                push_success(notifs, "Curso guardado");
                state.needs_reload = true;
                state.mode = Mode::List;
                clear_course_form(state);
            }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}
