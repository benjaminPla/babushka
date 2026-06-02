use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId,
    Margin, Stroke, TextStyle, Visuals,
};

// ── Color tokens ─────────────────────────────────────────────────────────────

pub mod colors {
    use eframe::egui::Color32;

    pub const BACKGROUND:    Color32 = Color32::from_rgb(245, 239, 231);
    pub const SURFACE:       Color32 = Color32::from_rgb(255, 252, 248);
    pub const SIDEBAR:       Color32 = Color32::from_rgb(234, 224, 210);

    pub const PRIMARY:       Color32 = Color32::from_rgb(122, 168, 138);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(188, 213, 197);

    pub const TEXT:           Color32 = Color32::from_rgb( 42,  28,  18);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(108,  82,  66);
    pub const TEXT_MUTED:     Color32 = Color32::from_rgb(162, 138, 122);

    pub const BORDER:         Color32 = Color32::from_rgb(210, 193, 176);
    pub const STRIPE:         Color32 = Color32::from_rgb(239, 231, 220);

    pub const SUCCESS:        Color32 = Color32::from_rgb( 52, 130,  74);
    pub const ERROR:          Color32 = Color32::from_rgb(168,  42,  63);
    pub const WARNING:        Color32 = Color32::from_rgb(192, 118,  18);

    pub const NOTIF_SUCCESS:  Color32 = Color32::from_rgb( 42, 114,  63);
    pub const NOTIF_ERROR:    Color32 = Color32::from_rgb(148,  33,  52);
    pub const NOTIF_WARNING:  Color32 = Color32::from_rgb(172, 104,  14);

    pub const WHITE: Color32 = Color32::WHITE;
}

// ── Spacing tokens ────────────────────────────────────────────────────────────

pub mod spacing {
    pub const PANEL_MARGIN: i8  = 10;
    pub const CORNER:       u8  =  5;
    pub const BTN_PAD_X:    f32 = 12.0;
    pub const BTN_PAD_Y:    f32 =  5.0;
    pub const ITEM_SPACING: f32 =  6.0;
}

// ── Theme entry point ─────────────────────────────────────────────────────────

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();

    style.text_styles = [
        (TextStyle::Heading,  FontId::new(18.0, FontFamily::Proportional)),
        (TextStyle::Body,     FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Button,   FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small,    FontId::new(11.5, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
    ].into();

    style.spacing.item_spacing   = egui::vec2(spacing::ITEM_SPACING, spacing::ITEM_SPACING);
    style.spacing.button_padding = egui::vec2(spacing::BTN_PAD_X, spacing::BTN_PAD_Y);
    style.spacing.menu_margin    = Margin::same(spacing::PANEL_MARGIN);
    style.spacing.window_margin  = Margin::same(spacing::PANEL_MARGIN);

    ctx.set_global_style(style);

    let cr = CornerRadius::same(spacing::CORNER);
    let mut v = Visuals::light();

    v.panel_fill           = colors::BACKGROUND;
    v.window_fill          = colors::SURFACE;
    v.faint_bg_color       = colors::STRIPE;
    v.extreme_bg_color     = colors::SURFACE;
    v.code_bg_color        = colors::STRIPE;
    v.window_corner_radius = cr;
    v.menu_corner_radius   = cr;
    v.window_stroke        = Stroke::new(1.0, colors::BORDER);
    v.warn_fg_color        = colors::WARNING;
    v.error_fg_color       = colors::ERROR;
    v.hyperlink_color      = colors::PRIMARY;
    v.selection.bg_fill    = colors::PRIMARY_LIGHT;
    v.selection.stroke     = Stroke::new(1.0, colors::PRIMARY);

    v.widgets.noninteractive.corner_radius = cr;
    v.widgets.noninteractive.bg_fill       = colors::BACKGROUND;
    v.widgets.noninteractive.weak_bg_fill  = colors::STRIPE;
    v.widgets.noninteractive.fg_stroke     = Stroke::new(1.0, colors::TEXT_SECONDARY);
    v.widgets.noninteractive.bg_stroke     = Stroke::new(1.0, colors::BORDER);

    v.widgets.inactive.corner_radius       = cr;
    v.widgets.inactive.bg_fill            = colors::SURFACE;
    v.widgets.inactive.weak_bg_fill       = colors::STRIPE;
    v.widgets.inactive.fg_stroke          = Stroke::new(1.0, colors::TEXT);
    v.widgets.inactive.bg_stroke          = Stroke::new(1.0, colors::BORDER);

    v.widgets.hovered.corner_radius        = cr;
    v.widgets.hovered.bg_fill             = colors::PRIMARY_LIGHT;
    v.widgets.hovered.weak_bg_fill        = colors::PRIMARY_LIGHT;
    v.widgets.hovered.fg_stroke           = Stroke::new(1.5, colors::TEXT);
    v.widgets.hovered.bg_stroke           = Stroke::new(1.5, colors::PRIMARY);

    v.widgets.active.corner_radius         = cr;
    v.widgets.active.bg_fill              = colors::PRIMARY;
    v.widgets.active.weak_bg_fill         = colors::PRIMARY;
    v.widgets.active.fg_stroke            = Stroke::new(2.0, colors::WHITE);
    v.widgets.active.bg_stroke            = Stroke::new(1.5, colors::PRIMARY);

    v.widgets.open.corner_radius           = cr;
    v.widgets.open.bg_fill                = colors::PRIMARY_LIGHT;
    v.widgets.open.weak_bg_fill           = colors::PRIMARY_LIGHT;
    v.widgets.open.fg_stroke              = Stroke::new(1.0, colors::TEXT);
    v.widgets.open.bg_stroke              = Stroke::new(1.0, colors::PRIMARY);

    ctx.set_visuals(v);
}

// ── Convenience ───────────────────────────────────────────────────────────────

pub fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(Margin::same(spacing::PANEL_MARGIN))
}
