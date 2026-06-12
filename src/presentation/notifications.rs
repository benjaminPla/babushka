use std::time::{Duration, Instant};

use eframe::egui;

use crate::theme::sizes;

const TIMEOUT: Duration = Duration::from_secs(5);

pub enum NotificationKind { Error, Success }

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

pub fn render(ui: &mut egui::Ui, notifs: &mut Notifications) {
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
                    NotificationKind::Error   => crate::theme::colors::RED,
                    NotificationKind::Success => crate::theme::colors::GREEN,
                };
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(sizes::MARGIN_NORMAL)
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
