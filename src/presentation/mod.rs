pub mod courses;
pub mod enrollments;
pub mod payments;
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
    ui.ctx().request_repaint_after(Duration::from_millis(250));
    let panel_rect = ui.max_rect();
    let width      = panel_rect.width();
    let mut remove_idx = None;

    egui::Area::new(egui::Id::new("notifications"))
        .anchor(egui::Align2::LEFT_BOTTOM, [panel_rect.min.x, 0.0])
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ui.ctx(), |ui| {
            for (i, notif) in notifs.iter().enumerate() {
                let (bg, fg) = match notif.kind {
                    NotificationKind::Error   => (crate::theme::colors::NOTIF_ERROR,   crate::theme::colors::BACKGROUND),
                    NotificationKind::Success => (crate::theme::colors::NOTIF_SUCCESS, crate::theme::colors::BACKGROUND),
                    NotificationKind::Warning => (crate::theme::colors::NOTIF_WARNING, crate::theme::colors::BACKGROUND),
                };
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_width(width);
                            ui.colored_label(fg, &notif.message);
                            let gap = ui.available_width() - 20.0;
                            if gap > 0.0 { ui.add_space(gap); }
                            let btn = egui::Button::new(
                                egui::RichText::new("×").color(fg).size(13.0)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE);
                            if ui.add(btn).clicked() { remove_idx = Some(i); }
                        });
                    });
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
