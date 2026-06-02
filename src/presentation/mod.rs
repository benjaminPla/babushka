pub mod courses;
pub mod enrollments;
pub mod payments;
pub mod students;
pub mod teachers;

use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Utc};
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
    let mut remove_idx = None;
    egui::Panel::bottom("notifications_bar")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            for (i, notif) in notifs.iter().enumerate() {
                let (bg, fg) = match notif.kind {
                    NotificationKind::Error   => (egui::Color32::from_rgb(180, 50,  50), egui::Color32::WHITE),
                    NotificationKind::Success => (egui::Color32::from_rgb(34,  139, 70), egui::Color32::WHITE),
                    NotificationKind::Warning => (egui::Color32::from_rgb(200, 140, 20), egui::Color32::WHITE),
                };
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("×").clicked() { remove_idx = Some(i); }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.colored_label(fg, &notif.message);
                            });
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
    dt.with_timezone(&art()).format("%d-%m-%Y %H:%M").to_string()
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
