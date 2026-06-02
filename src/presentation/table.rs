use eframe::egui;
pub use egui_extras::Column;
use egui_extras::TableBuilder;

const ROW_H:    f32 = 22.0;
const HEADER_H: f32 = 24.0;

/// Start a styled TableBuilder: striped, non-resizable, left-center layout.
pub fn builder(ui: &mut egui::Ui) -> TableBuilder<'_> {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
}

/// Renders a single header cell with a distinct background and muted bold label.
pub fn head(ui: &mut egui::Ui, label: &str) {
    ui.painter().rect_filled(
        ui.max_rect(),
        egui::CornerRadius::ZERO,
        crate::theme::colors::SIDEBAR,
    );
    ui.label(
        egui::RichText::new(label)
            .size(11.5)
            .color(crate::theme::colors::TEXT_SECONDARY)
            .strong(),
    );
}

pub fn row_height() -> f32 { ROW_H }
pub fn header_height() -> f32 { HEADER_H }
