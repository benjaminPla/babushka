use std::sync::{Arc, Mutex};

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use postgres::Client;
use uuid::Uuid;

use crate::application::dashboard::{DashboardUseCase, DebtorRow};
use crate::domain::student::repository::StudentRepo;
use crate::infrastructure::enrollment::EnrollmentPgRepo;
use crate::infrastructure::payment::PaymentPgRepo;
use crate::presentation::{fmt_ars, push_error, Notifications};
use crate::theme::{colors, sizes};

pub struct DashboardState {
    pub debtors:      Vec<DebtorRow>,
    pub needs_reload: bool,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self { debtors: Vec::new(), needs_reload: true }
    }
}

pub fn show(
    ui:     &mut egui::Ui,
    repo:   &Arc<dyn StudentRepo>,
    client: &Arc<Mutex<Client>>,
    state:  &mut DashboardState,
    notifs: &mut Notifications,
) -> Option<Uuid> {
    if state.needs_reload {
        state.needs_reload = false;
        let uc = DashboardUseCase::new(
            Arc::clone(repo),
            Arc::new(EnrollmentPgRepo::new(Arc::clone(client))),
            Arc::new(PaymentPgRepo::new(Arc::clone(client))),
        );
        match uc.debtors() {
            Ok(rows) => state.debtors = rows,
            Err(e)   => push_error(notifs, e.to_string()),
        }
    }

    ui.horizontal(|ui| {
        ui.heading("Panel");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(egui_phosphor::regular::ARROWS_CLOCKWISE).clicked() {
                state.needs_reload = true;
            }
        });
    });
    ui.separator();
    ui.add_space(sizes::SPACING_NORMAL);

    ui.label(egui::RichText::new("Alumnos con deuda").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_BIG));
    ui.add_space(sizes::SPACING_SMALL);

    if state.debtors.is_empty() {
        ui.label(egui::RichText::new("No hay alumnos con deuda.").color(colors::LIGHT_GRAY).size(sizes::FONT_SIZE_NORMAL));
        return None;
    }

    let mut navigate_to: Option<Uuid> = None;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder())
        .column(Column::auto())
        .column(Column::auto())
        .header(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut h| {
            h.col(|ui| { ui.label("Alumno"); });
            h.col(|ui| { ui.label("Deuda"); });
            h.col(|_ui| {});
        })
        .body(|mut body| {
            for row in &state.debtors {
                body.row(sizes::TABLE_ROW_HEIGHT_NORMAL, |mut r| {
                    r.col(|ui| {
                        ui.label(egui::RichText::new(&row.full_name).color(colors::WHITE).size(sizes::FONT_SIZE_NORMAL));
                    });
                    r.col(|ui| {
                        ui.colored_label(colors::RED, fmt_ars(row.balance_cents));
                    });
                    r.col(|ui| {
                        if ui.small_button(egui_phosphor::regular::EYE).clicked() {
                            navigate_to = Some(row.id);
                        }
                    });
                });
            }
        });

    navigate_to
}
