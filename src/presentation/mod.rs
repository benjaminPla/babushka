pub mod courses;
pub mod enrollments;
pub mod payments;
pub mod students;
pub mod table;
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
                            let gap = ui.available_width() - 24.0;
                            if gap > 0.0 { ui.add_space(gap); }
                            if ui.small_button("×").clicked() { remove_idx = Some(i); }
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
