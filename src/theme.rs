use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId,
    Margin, Stroke, TextStyle, Visuals,
};

// ── Color tokens ──────────────────────────────────────────────────────────────

pub mod colors {
    use eframe::egui::Color32;

    // Backgrounds
    pub const BACKGROUND: Color32 = Color32::from_rgb(26,  26,  28);
    pub const SURFACE:    Color32 = Color32::from_rgb(36,  36,  40);
    pub const SIDEBAR:    Color32 = Color32::from_rgb(20,  20,  22);
    pub const HEADER_BG:  Color32 = Color32::from_rgb(42,  42,  46);

    // Text
    pub const TEXT:           Color32 = Color32::from_rgb(220, 220, 220);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 148, 155);
    pub const TEXT_MUTED:     Color32 = Color32::from_rgb( 96,  96, 104);

    // Structure
    pub const BORDER: Color32 = Color32::from_rgb( 55,  55,  62);
    pub const STRIPE: Color32 = Color32::from_rgb( 30,  30,  34);

    // Accent
    pub const PRIMARY:       Color32 = Color32::from_rgb( 76, 148, 208);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb( 38,  76, 112);
    pub const PRIMARY_DARK:  Color32 = Color32::from_rgb( 55, 112, 168);

    // Status
    pub const SUCCESS: Color32 = Color32::from_rgb( 70, 158,  96);
    pub const ERROR:   Color32 = Color32::from_rgb(198,  68,  68);
    pub const WARNING: Color32 = Color32::from_rgb(200, 152,  50);

    // Notifications
    pub const NOTIF_SUCCESS: Color32 = Color32::from_rgb( 42, 128,  76);
    pub const NOTIF_ERROR:   Color32 = Color32::from_rgb(158,  48,  48);
    pub const NOTIF_WARNING: Color32 = Color32::from_rgb(162, 122,  42);

    pub const WHITE: Color32 = Color32::WHITE;
}

// ── Size / spacing tokens ─────────────────────────────────────────────────────

pub mod sizes {
    // Fonts
    pub const FONT_HEADING: f32 = 13.0;
    pub const FONT_BODY:    f32 = 11.0;
    pub const FONT_BUTTON:  f32 = 11.0;
    pub const FONT_SMALL:   f32 = 10.0;

    // Layout
    pub const SIDEBAR_WIDTH: f32 = 128.0;
    pub const PANEL_MARGIN:  i8  = 6;

    // Spacing
    pub const ITEM_SPACING_X: f32 = 6.0;
    pub const ITEM_SPACING_Y: f32 = 3.0;
    pub const BTN_PAD_X:      f32 = 8.0;
    pub const BTN_PAD_Y:      f32 = 2.0;

    // Table
    pub const ROW_HEIGHT:    f32 = 18.0;
    pub const HEADER_HEIGHT: f32 = 20.0;
    pub const HEAD_FONT:     f32 = 10.0;

    // Stroke weights
    pub const STROKE_THIN:   f32 = 1.0;
    pub const STROKE_MEDIUM: f32 = 1.5;
}

// ── Theme entry point ─────────────────────────────────────────────────────────

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();

    style.text_styles = [
        (TextStyle::Heading,   FontId::new(sizes::FONT_HEADING, FontFamily::Proportional)),
        (TextStyle::Body,      FontId::new(sizes::FONT_BODY,    FontFamily::Proportional)),
        (TextStyle::Button,    FontId::new(sizes::FONT_BUTTON,  FontFamily::Proportional)),
        (TextStyle::Small,     FontId::new(sizes::FONT_SMALL,   FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(sizes::FONT_BODY,    FontFamily::Monospace)),
    ]
    .into();

    style.spacing.item_spacing   = egui::vec2(sizes::ITEM_SPACING_X, sizes::ITEM_SPACING_Y);
    style.spacing.button_padding = egui::vec2(sizes::BTN_PAD_X,      sizes::BTN_PAD_Y);
    style.spacing.menu_margin    = Margin::same(sizes::PANEL_MARGIN);
    style.spacing.window_margin  = Margin::same(sizes::PANEL_MARGIN);

    ctx.set_global_style(style);

    let cr = CornerRadius::ZERO;

    let mut v = Visuals::dark();

    // Backgrounds
    v.panel_fill       = colors::BACKGROUND;
    v.window_fill      = colors::SURFACE;
    v.faint_bg_color   = colors::STRIPE;
    v.extreme_bg_color = colors::SURFACE;
    v.code_bg_color    = colors::STRIPE;

    v.override_text_color = Some(colors::TEXT);

    // Windows / menus
    v.window_corner_radius = cr;
    v.menu_corner_radius   = cr;
    v.window_stroke        = Stroke::new(sizes::STROKE_THIN, colors::BORDER);

    v.hyperlink_color = colors::PRIMARY;
    v.warn_fg_color   = colors::WARNING;
    v.error_fg_color  = colors::ERROR;

    v.selection.bg_fill = colors::PRIMARY_LIGHT;
    v.selection.stroke  = Stroke::new(sizes::STROKE_THIN, colors::PRIMARY);

    // ── Noninteractive ────────────────────────────────

    v.widgets.noninteractive.corner_radius = cr;
    v.widgets.noninteractive.bg_fill       = colors::BACKGROUND;
    v.widgets.noninteractive.weak_bg_fill  = colors::STRIPE;
    v.widgets.noninteractive.fg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::TEXT_SECONDARY);
    v.widgets.noninteractive.bg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::BORDER);

    // ── Inactive ──────────────────────────────────────

    v.widgets.inactive.corner_radius = cr;
    v.widgets.inactive.bg_fill       = colors::SURFACE;
    v.widgets.inactive.weak_bg_fill  = colors::SURFACE;
    v.widgets.inactive.fg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::TEXT);
    v.widgets.inactive.bg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::BORDER);

    // ── Hovered ───────────────────────────────────────

    v.widgets.hovered.corner_radius = cr;
    v.widgets.hovered.bg_fill       = colors::PRIMARY_LIGHT;
    v.widgets.hovered.weak_bg_fill  = colors::PRIMARY_LIGHT;
    v.widgets.hovered.fg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::TEXT);
    v.widgets.hovered.bg_stroke     = Stroke::new(sizes::STROKE_MEDIUM, colors::PRIMARY);

    // ── Active ────────────────────────────────────────

    v.widgets.active.corner_radius = cr;
    v.widgets.active.bg_fill       = colors::PRIMARY_DARK;
    v.widgets.active.weak_bg_fill  = colors::PRIMARY_DARK;
    v.widgets.active.fg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::WHITE);
    v.widgets.active.bg_stroke     = Stroke::new(sizes::STROKE_MEDIUM, colors::PRIMARY);

    // ── Open ──────────────────────────────────────────

    v.widgets.open.corner_radius = cr;
    v.widgets.open.bg_fill       = colors::PRIMARY_LIGHT;
    v.widgets.open.weak_bg_fill  = colors::PRIMARY_LIGHT;
    v.widgets.open.fg_stroke     = Stroke::new(sizes::STROKE_THIN, colors::TEXT);
    v.widgets.open.bg_stroke     = Stroke::new(sizes::STROKE_MEDIUM, colors::PRIMARY);

    ctx.set_visuals(v);
}

// ── Convenience ───────────────────────────────────────────────────────────────

pub fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(Margin::same(sizes::PANEL_MARGIN))
}
