pub mod courses;
pub mod students;
pub mod table;
pub mod teachers;

use std::time::{Duration, Instant};

use chrono::{Datelike, DateTime, FixedOffset, NaiveDate, Utc};
use eframe::egui;
use uuid::Uuid;

// ── Notifications ────────────────────────────────────────────────────────────

const TIMEOUT: Duration = Duration::from_secs(5);

pub enum NotificationKind { Error, Success, Warning }

pub struct Notification {
    pub message: String,
    pub kind:    NotificationKind,
    born:        Instant,
}

pub type Notifications = Vec<Notification>;

pub fn push_error(n: &mut Notifications, msg: impl Into<String>) {
    n.push(Notification { message: msg.into(), kind: NotificationKind::Error, born: Instant::now() });
}
pub fn push_success(n: &mut Notifications, msg: impl Into<String>) {
    n.push(Notification { message: msg.into(), kind: NotificationKind::Success, born: Instant::now() });
}

pub fn render_notifications(ui: &mut egui::Ui, notifs: &mut Notifications) {
    notifs.retain(|n| n.born.elapsed() < TIMEOUT);
    if notifs.is_empty() { return; }
    ui.ctx().request_repaint_after(Duration::from_secs(1));

    const W: f32 = 300.0;
    let mut remove_idx = None;

    egui::Area::new(egui::Id::new("notifications"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [-8.0, -8.0])
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ui.ctx(), |ui| {
            ui.set_min_width(W);
            ui.set_max_width(W);
            for (i, notif) in notifs.iter().enumerate() {
                let bg = match notif.kind {
                    NotificationKind::Error   => crate::theme::colors::NOTIF_ERROR,
                    NotificationKind::Success => crate::theme::colors::NOTIF_SUCCESS,
                    NotificationKind::Warning => crate::theme::colors::NOTIF_WARNING,
                };
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.set_min_width(W - 16.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(crate::theme::colors::WHITE, &notif.message);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let btn = egui::Button::new(
                                    egui::RichText::new("×").color(crate::theme::colors::WHITE).size(14.0)
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE);
                                if ui.add(btn).clicked() { remove_idx = Some(i); }
                            });
                        });
                    });
                ui.add_space(2.0);
            }
        });

    if let Some(i) = remove_idx { notifs.remove(i); }
}

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

pub fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        egui::RichText::new(label)
            .size(crate::theme::sizes::FONT_SMALL)
            .color(crate::theme::colors::TEXT_MUTED)
            .strong(),
    );
    ui.separator();
}

pub fn confirm_delete_modal(ctx: &egui::Context, pending: &mut Option<Uuid>) -> Option<Uuid> {
    let Some(id) = *pending else { return None };
    let mut confirmed = None;
    egui::Window::new("Confirmar")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("¿Eliminar este registro?");
            ui.label("Esta acción no se puede deshacer.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked()      { *pending = None; }
                if ui.button("Sí, eliminar").clicked()  { confirmed = Some(id); *pending = None; }
            });
        });
    confirmed
}
