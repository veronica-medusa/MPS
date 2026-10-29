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
        // Mercury Modern Indigo
        Self {
            dark1: Color::MERCURY_INK,           // #28253b
            dark2: Color::MERCURY_NIGHT,         // #171721
            light1: Color::WHITE,
            light2: Color::MERCURY_CANVAS,       // #fbfcfd
            accent1: Color::MERCURY_PRIMARY,     // Indigo #5266eb
            accent2: Color::ACCENT_SHEETS,       // Emerald #0ca678
            accent3: Color::ACCENT_SLIDES,       // Coral #e8590c
            accent4: Color::ACCENT_PDF,          // Crimson #e03131
            accent5: Color::ACCENT_AI,           // Iris #7048e8
            accent6: Color::MERCURY_PERIWINKLE,  // Periwinkle #9cb4e8
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideTheme {
    pub id: String,
    pub name: String,
    pub color_scheme: ThemeColorScheme,
    pub font_scheme: ThemeFontScheme,
}

impl Default for SlideTheme {
    fn default() -> Self {
        Self::mercury_modern()
    }
}

fn hex(s: &str) -> Color {
    Color::from_hex(s).unwrap_or(Color::BLACK)
}

impl SlideTheme {
    /// 1. Mercury Modern (Default Indigo & Clean Grays)
    pub fn mercury_modern() -> Self {
        Self {
            id: "theme-modern".to_string(),
            name: "Mercury Modern".to_string(),
            color_scheme: ThemeColorScheme::default(),
            font_scheme: ThemeFontScheme {
                major_font: "Inter".to_string(),
                minor_font: "Inter".to_string(),
            },
        }
    }

    /// 2. Executive Slate (Professional Cool Grays & Deep Slate)
    pub fn executive_slate() -> Self {
        Self {
            id: "theme-executive-slate".to_string(),
            name: "Executive Slate".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#1e293b"),
                dark2: hex("#0f172a"),
                light1: Color::WHITE,
                light2: hex("#f8fafc"),
                accent1: hex("#334155"),
                accent2: hex("#64748b"),
                accent3: hex("#0284c7"),
                accent4: hex("#0d9488"),
                accent5: hex("#d97706"),
                accent6: hex("#94a3b8"),
                hyperlink: hex("#0284c7"),
                followed_hyperlink: hex("#64748b"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Segoe UI".to_string(),
                minor_font: "Segoe UI".to_string(),
            },
        }
    }

    /// 3. Emerald Growth (Vibrant Greens & Organic Tones)
    pub fn emerald_growth() -> Self {
        Self {
            id: "theme-emerald-growth".to_string(),
            name: "Emerald Growth".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#132a13"),
                dark2: hex("#061a06"),
                light1: Color::WHITE,
                light2: hex("#f4fbf5"),
                accent1: hex("#0ca678"), // Emerald
                accent2: hex("#2f9e44"),
                accent3: hex("#38d9a9"),
                accent4: hex("#f59f00"),
                accent5: hex("#1098ad"),
                accent6: hex("#82c91e"),
                hyperlink: hex("#0ca678"),
                followed_hyperlink: hex("#2f9e44"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Plus Jakarta Sans".to_string(),
                minor_font: "Plus Jakarta Sans".to_string(),
            },
        }
    }

    /// 4. Coral Sunset (Warm Coral & Sun Gold)
    pub fn coral_sunset() -> Self {
        Self {
            id: "theme-coral-sunset".to_string(),
            name: "Coral Sunset".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#2b1e1a"),
                dark2: hex("#1a100c"),
                light1: Color::WHITE,
                light2: hex("#fff9f6"),
                accent1: hex("#e8590c"), // Coral Slides Accent
                accent2: hex("#f76707"),
                accent3: hex("#fd7e14"),
                accent4: hex("#fab005"),
                accent5: hex("#d6336c"),
                accent6: hex("#ffd8a8"),
                hyperlink: hex("#e8590c"),
                followed_hyperlink: hex("#d6336c"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Cabinet Grotesk".to_string(),
                minor_font: "Inter".to_string(),
            },
        }
    }

    /// 5. Crimson Bold (High-Impact Crimson & High-Contrast Dark)
    pub fn crimson_bold() -> Self {
        Self {
            id: "theme-crimson-bold".to_string(),
            name: "Crimson Bold".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#2b1111"),
                dark2: hex("#140505"),
                light1: Color::WHITE,
                light2: hex("#fff5f5"),
                accent1: hex("#e03131"), // Crimson
                accent2: hex("#c92a2a"),
                accent3: hex("#ff6b6b"),
                accent4: hex("#f03e3e"),
                accent5: hex("#495057"),
                accent6: hex("#ffa8a8"),
                hyperlink: hex("#e03131"),
                followed_hyperlink: hex("#862e9c"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Outfit".to_string(),
                minor_font: "Inter".to_string(),
            },
        }
    }

    /// 6. Iris Creative (Playful Purple & Creative Violet)
    pub fn iris_creative() -> Self {
        Self {
            id: "theme-iris-creative".to_string(),
            name: "Iris Creative".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#211933"),
                dark2: hex("#120c1f"),
                light1: Color::WHITE,
                light2: hex("#faf7ff"),
                accent1: hex("#7048e8"), // Iris AI Accent
                accent2: hex("#5f3dc4"),
                accent3: hex("#845ef7"),
                accent4: hex("#be4bdb"),
                accent5: hex("#3b5bdb"),
                accent6: hex("#d0bfff"),
                hyperlink: hex("#7048e8"),
                followed_hyperlink: hex("#be4bdb"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Poppins".to_string(),
                minor_font: "Inter".to_string(),
            },
        }
    }

    /// 7. Midnight Cyber (High-Tech Dark Mode & Neon Cyan)
    pub fn midnight_cyber() -> Self {
        Self {
            id: "theme-midnight-cyber".to_string(),
            name: "Midnight Cyber".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#f8fafc"),
                dark2: hex("#e2e8f0"),
                light1: hex("#090d16"), // Dark slide background
                light2: hex("#131a2a"),
                accent1: hex("#00f2fe"), // Neon Cyan
                accent2: hex("#4facfe"),
                accent3: hex("#7f00ff"),
                accent4: hex("#ff007f"),
                accent5: hex("#00e676"),
                accent6: hex("#38bdf8"),
                hyperlink: hex("#00f2fe"),
                followed_hyperlink: hex("#ff007f"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Space Grotesk".to_string(),
                minor_font: "Inter".to_string(),
            },
        }
    }

    /// 8. Solar Sand (Warm Earthy Ochre & Deep Espresso)
    pub fn solar_sand() -> Self {
        Self {
            id: "theme-solar-sand".to_string(),
            name: "Solar Sand".to_string(),
            color_scheme: ThemeColorScheme {
                dark1: hex("#2d241e"),
                dark2: hex("#1b140f"),
                light1: Color::WHITE,
                light2: hex("#fdfbf7"),
                accent1: hex("#f59f00"), // Amber
                accent2: hex("#d9480f"),
                accent3: hex("#e67700"),
                accent4: hex("#825a2c"),
                accent5: hex("#40c057"),
                accent6: hex("#ffe066"),
                hyperlink: hex("#e67700"),
                followed_hyperlink: hex("#825a2c"),
            },
            font_scheme: ThemeFontScheme {
                major_font: "Newsreader".to_string(),
                minor_font: "Inter".to_string(),
            },
        }
    }

    /// All 8 Mercury themes in a catalogue
    pub fn all_themes() -> Vec<Self> {
        vec![
            Self::mercury_modern(),
            Self::executive_slate(),
            Self::emerald_growth(),
            Self::coral_sunset(),
            Self::crimson_bold(),
            Self::iris_creative(),
            Self::midnight_cyber(),
            Self::solar_sand(),
        ]
    }
}
