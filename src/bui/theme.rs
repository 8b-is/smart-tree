//! BUI Theme for Smart Tree
//!
//! Blender-inspired dark theme with accent colors.

#[cfg(feature = "bui")]
use blenderui::prelude::*;

#[cfg(feature = "bui")]
#[derive(Clone, Copy)]
pub struct BuiTheme {
    pub background: Color,
    pub surface: Color,
    pub surface_variant: Color,
    pub primary: Color,
    pub primary_variant: Color,
    pub secondary: Color,
    pub secondary_variant: Color,
    pub on_background: Color,
    pub on_surface: Color,
    pub on_primary: Color,
    pub on_secondary: Color,
    pub outline: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub info: Color,
}

#[cfg(feature = "bui")]
impl Default for BuiTheme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(feature = "bui")]
impl BuiTheme {
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex("#0d1117"),
            surface: Color::from_hex("#161b22"),
            surface_variant: Color::from_hex("#21262d"),
            primary: Color::from_hex("#58a6ff"),
            primary_variant: Color::from_hex("#1f6feb"),
            secondary: Color::from_hex("#d2a8ff"),
            secondary_variant: Color::from_hex("#a371f7"),
            on_background: Color::from_hex("#e6edf3"),
            on_surface: Color::from_hex("#c9d1d9"),
            on_primary: Color::from_hex("#0d1117"),
            on_secondary: Color::from_hex("#0d1117"),
            outline: Color::from_hex("#30363d"),
            error: Color::from_hex("#f85149"),
            success: Color::from_hex("#3fb950"),
            warning: Color::from_hex("#d29922"),
            info: Color::from_hex("#58a6ff"),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::from_hex("#ffffff"),
            surface: Color::from_hex("#f6f8fa"),
            surface_variant: Color::from_hex("#f3f4f6"),
            primary: Color::from_hex("#0969da"),
            primary_variant: Color::from_hex("#0550ae"),
            secondary: Color::from_hex("#8957e5"),
            secondary_variant: Color::from_hex("#6e40c9"),
            on_background: Color::from_hex("#24292f"),
            on_surface: Color::from_hex("#57606a"),
            on_primary: Color::from_hex("#ffffff"),
            on_secondary: Color::from_hex("#ffffff"),
            outline: Color::from_hex("#d0d7de"),
            error: Color::from_hex("#cf222e"),
            success: Color::from_hex("#1a7f37"),
            warning: Color::from_hex("#9a6700"),
            info: Color::from_hex("#0969da"),
        }
    }

    #[cfg(feature = "bui")]
    pub fn to_blenderui_theme(&self) -> blenderui::Theme {
        blenderui::Theme {
            background: self.background,
            surface: self.surface,
            surface_variant: self.surface_variant,
            primary: self.primary,
            primary_variant: self.primary_variant,
            secondary: self.secondary,
            secondary_variant: self.secondary_variant,
            on_background: self.on_background,
            on_surface: self.on_surface,
            on_primary: self.on_primary,
            on_secondary: self.on_secondary,
            outline: self.outline,
            error: self.error,
            success: self.success,
            warning: self.warning,
            info: self.info,
            font_family: "JetBrains Mono".into(),
            font_size: 13.0,
            line_height: 1.5,
            border_radius: 4.0,
            spacing: 8.0,
        }
    }

    pub fn with_accent(mut self, color: Color) -> Self {
        self.primary = color;
        self.primary_variant = Color::from_rgba(
            (color.r() as f32 * 0.7) as u8,
            (color.g() as f32 * 0.7) as u8,
            (color.b() as f32 * 0.7) as u8,
            color.a(),
        );
        self.secondary = Color::from_hsv(
            (color.hue() + 180.0) % 360.0,
            color.saturation() * 0.8,
            color.value() * 0.9,
        );
        self
    }
}
