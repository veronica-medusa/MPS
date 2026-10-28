use crate::error::CoreError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const TRANSPARENT: Self = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Self = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Color { r: 255, g: 255, b: 255, a: 255 };

    // Mercury Design System Brand Tokens
    pub const MERCURY_PRIMARY: Self = Color { r: 82, g: 102, b: 235, a: 255 }; // #5266eb
    pub const MERCURY_PERIWINKLE: Self = Color { r: 156, g: 180, b: 232, a: 255 }; // #9cb4e8
    pub const MERCURY_CANVAS: Self = Color { r: 251, g: 252, b: 253, a: 255 }; // #fbfcfd
    pub const MERCURY_SURFACE: Self = Color { r: 255, g: 255, b: 255, a: 255 }; // #ffffff
    pub const MERCURY_LAVENDER: Self = Color { r: 237, g: 236, b: 251, a: 255 }; // #edecfb
    pub const MERCURY_NIGHT: Self = Color { r: 23, g: 23, b: 33, a: 255 }; // #171721
    pub const MERCURY_NIGHT_RAISED: Self = Color { r: 30, g: 30, b: 42, a: 255 }; // #1e1e2a
    pub const MERCURY_INK: Self = Color { r: 40, g: 37, b: 59, a: 255 }; // #28253b
    pub const MERCURY_INK_SECONDARY: Self = Color { r: 85, g: 78, b: 123, a: 255 }; // #554e7b
    pub const MERCURY_ON_DARK: Self = Color { r: 237, g: 237, b: 243, a: 255 }; // #ededf3
    pub const MERCURY_FOG: Self = Color { r: 195, g: 195, b: 204, a: 255 }; // #c3c3cc
    pub const MERCURY_SLATE: Self = Color { r: 112, g: 112, b: 125, a: 255 }; // #70707d

    // Module Semantic Accents
    pub const ACCENT_WRITER: Self = Color { r: 59, g: 91, b: 219, a: 255 }; // Cobalt #3B5BDB
    pub const ACCENT_SHEETS: Self = Color { r: 12, g: 166, b: 120, a: 255 }; // Emerald #0CA678
    pub const ACCENT_SLIDES: Self = Color { r: 232, g: 89, b: 12, a: 255 }; // Coral #E8590C
    pub const ACCENT_PDF: Self = Color { r: 224, g: 49, b: 49, a: 255 }; // Crimson #E03131
    pub const ACCENT_AI: Self = Color { r: 112, g: 72, b: 232, a: 255 }; // Iris #7048E8

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Parse hex string e.g. "#5266eb", "5266eb", "#5266eb80"
    pub fn from_hex(hex: &str) -> Result<Self, CoreError> {
        let clean = hex.trim().trim_start_matches('#');
        match clean.len() {
            3 => {
                let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                Ok(Self::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&clean[0..2], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let g = u8::from_str_radix(&clean[2..4], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let b = u8::from_str_radix(&clean[4..6], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                Ok(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&clean[0..2], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let g = u8::from_str_radix(&clean[2..4], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let b = u8::from_str_radix(&clean[4..6], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                let a = u8::from_str_radix(&clean[6..8], 16).map_err(|e| CoreError::InvalidColorFormat(e.to_string()))?;
                Ok(Self::rgba(r, g, b, a))
            }
            _ => Err(CoreError::InvalidColorFormat(format!("Invalid hex length: {}", hex))),
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn to_css_rgba(&self) -> String {
        let alpha = self.a as f32 / 255.0;
        format!("rgba({}, {}, {}, {:.3})", self.r, self.g, self.b, alpha)
    }

    pub fn relative_luminance(&self) -> f64 {
        let srgb = |c: u8| {
            let v = c as f64 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * srgb(self.r) + 0.7152 * srgb(self.g) + 0.0722 * srgb(self.b)
    }

    pub fn contrast_ratio(&self, other: &Self) -> f64 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    pub fn blend_over(&self, background: &Self) -> Self {
        let src_a = self.a as f64 / 255.0;
        let dst_a = background.a as f64 / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);

        if out_a == 0.0 {
            return Self::TRANSPARENT;
        }

        let blend_channel = |s: u8, d: u8| -> u8 {
            let s_f = s as f64 / 255.0;
            let d_f = d as f64 / 255.0;
            let res = (s_f * src_a + d_f * dst_a * (1.0 - src_a)) / out_a;
            (res * 255.0).round().clamp(0.0, 255.0) as u8
        };

        Self {
            r: blend_channel(self.r, background.r),
            g: blend_channel(self.g, background.g),
            b: blend_channel(self.b, background.b),
            a: (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_hex() {
        let c = Color::from_hex("#5266eb").unwrap();
        assert_eq!(c, Color::MERCURY_PRIMARY);
        assert_eq!(c.to_hex(), "#5266eb");

        let with_alpha = Color::from_hex("#5266eb80").unwrap();
        assert_eq!(with_alpha.a, 128);
        assert_eq!(with_alpha.to_hex(), "#5266eb80");
    }

    #[test]
    fn test_contrast() {
        let contrast = Color::BLACK.contrast_ratio(&Color::WHITE);
        assert!((contrast - 21.0).abs() < 0.1);
    }
}
