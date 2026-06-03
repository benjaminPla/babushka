use eframe::egui;
pub use egui_extras::Column;
use egui_extras::TableBuilder;

use crate::theme::{colors, sizes};

pub fn builder(ui: &mut egui::Ui) -> TableBuilder<'_> {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
}

pub fn head(ui: &mut egui::Ui, label: &str) {
    ui.painter().rect_filled(ui.max_rect(), egui::CornerRadius::ZERO, colors::HEADER_BG);
    ui.label(
        egui::RichText::new(label)
            .size(sizes::HEAD_FONT)
            .color(colors::TEXT_SECONDARY)
            .strong(),
    );
}

pub fn head_filter(ui: &mut egui::Ui, placeholder: &str, filter: &mut String) {
    ui.painter().rect_filled(ui.max_rect(), egui::CornerRadius::ZERO, colors::HEADER_BG);
    let avail = ui.available_width();
    let text_w = if filter.is_empty() { avail } else { (avail - 20.0).max(20.0) };
    ui.horizontal(|ui| {
        let hint = egui::RichText::new(placeholder)
            .size(sizes::HEAD_FONT)
            .color(colors::TEXT_SECONDARY)
            .strong();
        ui.add(
            egui::TextEdit::singleline(filter)
                .hint_text(hint)
                .desired_width(text_w)
                .font(egui::FontId::proportional(sizes::HEAD_FONT)),
        );
        if !filter.is_empty() {
            if ui.small_button("×").clicked() { filter.clear(); }
        }
    });
}

pub fn row_height()    -> f32 { sizes::ROW_HEIGHT }
pub fn header_height() -> f32 { sizes::HEADER_HEIGHT }
