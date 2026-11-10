//! App-wide look: roomier buttons than egui's defaults.

use egui::{Context, Style, vec2};

/// Space between a button's text and its border.
pub const BUTTON_PADDING: egui::Vec2 = egui::Vec2::new(10.0, 5.0);

fn adjust(style: &mut Style) {
    style.spacing.button_padding = BUTTON_PADDING;
    style.spacing.interact_size = vec2(style.spacing.interact_size.x, 28.0);
    style.spacing.item_spacing = vec2(8.0, 6.0);
}

/// Apply to both the light and the dark style.
pub fn apply(ctx: &Context) {
    ctx.all_styles_mut(adjust);
}
