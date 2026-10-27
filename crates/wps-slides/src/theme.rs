use serde::{Deserialize, Serialize};
use wps_core::Color;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeColorScheme {
    pub dark1: Color,
    pub dark2: Color,
    pub light1: Color,
    pub light2: Color,
    pub accent1: Color,
    pub accent2: Color,
    pub accent3: Color,
    pub accent4: Color,
    pub accent5: Color,
    pub accent6: Color,
    pub hyperlink: Color,
    pub followed_hyperlink: Color,
}

impl Default for ThemeColorScheme {
    fn default() -> Self {
        // Mercury Office Theme Color Scheme
        Self {
            dark1: Color::MERCURY_INK,         // #28253b
            dark2: Color::MERCURY_NIGHT,       // #171721
            light1: Color::WHITE,
            light2: Color::MERCURY_CANVAS,     // #fbfcfd
            accent1: Color::MERCURY_PRIMARY,   // Indigo #5266eb
            accent2: Color::ACCENT_SHEETS,     // Jade #0ca678
            accent3: Color::ACCENT_SLIDES,     // Coral #e8590c
            accent4: Color::ACCENT_PDF,        // Crimson #e03131
            accent5: Color::ACCENT_AI,         // Iris #7048e8
            accent6: Color::MERCURY_PERIWINKLE,// Periwinkle #9cb4e8
            hyperlink: Color::MERCURY_PRIMARY,
            followed_hyperlink: Color::ACCENT_AI,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeFontScheme {
    pub major_font: String, // Headings
    pub minor_font: String, // Body
}

impl Default for ThemeFontScheme {
    fn default() -> Self {
        Self {
            major_font: "Inter".to_string(),
            minor_font: "Inter".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SlideTheme {
    pub name: String,
    pub color_scheme: ThemeColorScheme,
    pub font_scheme: ThemeFontScheme,
}

impl SlideTheme {
    pub fn mercury_modern() -> Self {
        Self {
            name: "Mercury Modern".to_string(),
            color_scheme: ThemeColorScheme::default(),
            font_scheme: ThemeFontScheme::default(),
        }
    }
}
