use std::sync::{Arc, Mutex};

use chrono::{Duration, NaiveDate};
use eframe::egui;
use postgres::Client;
use uuid::Uuid;

use crate::{
    application::course_period::{
        create::{CoursePeriodCreateInput, CoursePeriodCreateUseCase},
        delete::CoursePeriodDeleteUseCase,
        get_by_course::CoursePeriodGetByCourseUseCase,
    },
    presentation::{confirm_delete_modal, push_error, push_success, section_header, Notifications},
    presentation::table::{self, Column},
};

use super::{CoursesState, Mode, format_price, make_course_period_repo};

pub fn show(ui: &mut egui::Ui, client: &Arc<Mutex<Client>>, state: &mut CoursesState, notifs: &mut Notifications) {
    let course = match &state.selected_course {
        Some(c) => c.clone(),
        None    => { state.mode = Mode::List; return; }
    };

    if state.needs_reload_periods {
        state.needs_reload_periods = false;
        match CoursePeriodGetByCourseUseCase::new(make_course_period_repo(client)).execute(course.id) {
            Ok(periods) => { state.periods = periods; }
            Err(e)      => push_error(notifs, e.to_string()),
        }
    }

    // ── Header ────────────────────────────────────────────────────────────────
    if ui.button("← Volver").clicked() {
        state.mode             = Mode::List;
        state.selected_course  = None;
        state.periods          = Vec::new();
        state.show_period_form = false;
        return;
    }
    ui.separator();

    // ── Info section ──────────────────────────────────────────────────────────
    section_header(ui, "Información");
    ui.heading(format!("{} — {}", course.name, course.age_group.label()));
    egui::Grid::new("course_detail_info").num_columns(2).show(ui, |ui| {
        ui.label("Capacidad");      ui.label(course.capacity.to_string());                                ui.end_row();
        ui.label("Precio mensual"); ui.label(format_price(course.month_price_cents));                     ui.end_row();
        ui.label("Precio clase");   ui.label(format_price(course.class_price_cents));                     ui.end_row();
        if let Some(n) = &course.notes {
            ui.label("Notas"); ui.label(n); ui.end_row();
        }
        ui.label("Creado");  ui.label(crate::presentation::fmt_dt(course.created_at)); ui.end_row();
        ui.label("Editado"); ui.label(crate::presentation::fmt_dt(course.updated_at)); ui.end_row();
    });
    ui.add_space(4.0);
    ui.separator();

    // ── Periods section ───────────────────────────────────────────────────────
    if state.show_period_form {
        ui.horizontal(|ui| {
            ui.label("Año");
            egui::ComboBox::from_id_salt("period_year")
                .selected_text(state.period_year.to_string())
                .show_ui(ui, |ui| {
                    for y in 2024..=2030 {
                        ui.selectable_value(&mut state.period_year, y, y.to_string());
                    }
                });
            ui.label("Mes");
            egui::ComboBox::from_id_salt("period_month")
                .selected_text(MONTHS[(state.period_month - 1) as usize])
                .show_ui(ui, |ui| {
                    for (i, name) in MONTHS.iter().enumerate() {
                        ui.selectable_value(&mut state.period_month, (i + 1) as u32, *name);
                    }
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancelar").clicked() { state.show_period_form = false; }
                if ui.button("Guardar").clicked() {
                    let y = state.period_year;
                    let m = state.period_month;
                    let label      = format!("{} {}", MONTHS[(m - 1) as usize], y);
                    let start_date = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
                    let end_date   = if m == 12 {
                        NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap() - Duration::days(1)
                    } else {
                        NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap() - Duration::days(1)
                    };
                    match CoursePeriodCreateUseCase::new(make_course_period_repo(client))
                        .execute(CoursePeriodCreateInput { course_id: course.id, label, start_date, end_date }) {
                        Ok(_) => {
                            push_success(notifs, "Período creado");
                            state.show_period_form     = false;
                            state.needs_reload_periods = true;
                        }
                        Err(e) => push_error(notifs, e.to_string()),
                    }
                }
            });
        });
        ui.separator();
    } else {
        ui.horizontal(|ui| {
            section_header(ui, "Períodos");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+ Nuevo período").clicked() { state.show_period_form = true; }
            });
        });
    }

    // ── Periods table ─────────────────────────────────────────────────────────
    let mut delete_id: Option<Uuid> = None;

    table::builder(ui)
        .column(Column::remainder().at_least(100.0))
        .column(Column::exact(90.0))
        .column(Column::exact(90.0))
        .column(Column::exact(70.0))
        .column(Column::exact(80.0))
        .column(Column::auto().at_least(60.0))
        .header(table::header_height(), |mut h| {
            h.col(|ui| table::head(ui, "Etiqueta"));
            h.col(|ui| table::head(ui, "Inicio"));
            h.col(|ui| table::head(ui, "Fin"));
            h.col(|ui| table::head(ui, "Inscritos"));
            h.col(|ui| table::head(ui, "Estado"));
            h.col(|ui| table::head(ui, "Acciones"));
        })
        .body(|mut body| {
            for p in &state.periods {
                body.row(table::row_height(), |mut row| {
                    row.col(|ui| { ui.label(&p.label); });
                    row.col(|ui| { ui.label(p.start_date.format("%d/%m/%Y").to_string()); });
                    row.col(|ui| { ui.label(p.end_date.format("%d/%m/%Y").to_string()); });
                    row.col(|ui| { ui.label(p.enrolled.to_string()); });
                    row.col(|ui| {
                        let today = chrono::Local::now().date_naive();
                        if p.start_date > today {
                            ui.colored_label(crate::theme::colors::WARNING, "Futuro");
                        } else if p.end_date >= today {
                            ui.colored_label(crate::theme::colors::SUCCESS, "Activo");
                        } else {
                            ui.colored_label(crate::theme::colors::TEXT_MUTED, "Finalizado");
                        }
                    });
                    row.col(|ui| {
                        if ui.small_button("🗑").clicked() { delete_id = Some(p.id); }
                    });
                });
            }
        });

    if let Some(id) = delete_id { state.confirm_delete_period = Some(id); }

    if let Some(id) = confirm_delete_modal(ui.ctx(), &mut state.confirm_delete_period) {
        match CoursePeriodDeleteUseCase::new(make_course_period_repo(client)).execute(id) {
            Ok(_)  => { push_success(notifs, "Período eliminado"); state.needs_reload_periods = true; }
            Err(e) => push_error(notifs, e.to_string()),
        }
    }
}

const MONTHS: [&str; 12] = [
    "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
    "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre",
];

