pub mod courses;
pub mod notifications;
pub mod students;
pub mod table;
pub mod teachers;

pub use notifications::{Notifications, push_error, push_success, render as render_notifications};

use chrono::{Datelike, DateTime, FixedOffset, NaiveDate, Utc};
use eframe::egui;
use uuid::Uuid;

use crate::theme::{colors, sizes};

fn art() -> FixedOffset {
    FixedOffset::west_opt(3 * 3600).unwrap()
}

pub fn fmt_dt(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&art()).format("%d/%m/%Y %H:%M").to_string()
}

pub fn fmt_ars(cents: i32) -> String {
    let abs = cents.unsigned_abs();
    let sign = if cents < 0 { "-" } else { "" };
    let pesos = (abs / 100) as u64;
    let centavos = abs % 100;
    let pesos_str = {
        let s = pesos.to_string();
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::with_capacity(s.len() + s.len() / 3);
        for (i, &c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 { result.push('.'); }
            result.push(c);
        }
        result
    };
    format!("{sign}${pesos_str},{centavos:02}")
}

/// Three-dropdown date selector (día / mes / año). No external dependency.
pub fn date_selector(ui: &mut egui::Ui, id: &str, date: &mut NaiveDate) {
    const MONTHS: [&str; 12] = [
        "Ene", "Feb", "Mar", "Abr", "May", "Jun",
        "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
    ];

    let mut y = date.year();
    let mut m = date.month();
    let mut d = date.day();

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt(format!("{id}_d"))
            .selected_text(format!("{d:02}"))
            .show_ui(ui, |ui| {
                for day in 1..=days_in_month(y, m) {
                    ui.selectable_value(&mut d, day, format!("{day:02}"));
                }
            });
        egui::ComboBox::from_id_salt(format!("{id}_m"))
            .selected_text(MONTHS[(m - 1) as usize])
            .show_ui(ui, |ui| {
                for (i, name) in MONTHS.iter().enumerate() {
                    ui.selectable_value(&mut m, (i + 1) as u32, *name);
                }
            });
        egui::ComboBox::from_id_salt(format!("{id}_y"))
            .selected_text(y.to_string())
            .show_ui(ui, |ui| {
                for year in 2020..=2030 {
                    ui.selectable_value(&mut y, year, year.to_string());
                }
            });
    });

    let d = d.min(days_in_month(y, m));
    if let Some(nd) = NaiveDate::from_ymd_opt(y, m, d) { *date = nd; }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    (first_next - chrono::Duration::days(1)).day()
}


pub fn confirm_delete_modal(ctx: &egui::Context, pending: &mut Option<Uuid>) -> Option<Uuid> {
    let Some(id) = *pending else { return None };
    let mut confirmed = None;
    egui::Window::new("Confirmar")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::new()
            .fill(colors::BLACK)
            .stroke(egui::Stroke::new(sizes::STROKE_SMALL, colors::WHITE))
            .inner_margin(egui::Margin::same(sizes::MARGIN_NORMAL))
        )
        .show(ctx, |ui| {
            ui.label("¿Eliminar este registro?");
            ui.label("Esta acción no se puede deshacer.");
            ui.add_space(sizes::SPACING_NORMAL);
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked()      { *pending = None; }
                if ui.button("Sí, eliminar").clicked()  { confirmed = Some(id); *pending = None; }
            });
        });
    confirmed
}
