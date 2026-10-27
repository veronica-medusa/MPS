use serde::{Deserialize, Serialize};

/// Typographic Point (1/72 of an inch)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Pt(pub f64);

/// Twip (1/20 of a Point, standard for Word processing & OOXML margins/fonts)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Twips(pub i64);

/// English Metric Unit (EMU: 1/914400 of an inch, standard for DrawingML & OOXML vector shapes)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Emu(pub i64);

/// Screen Pixel at 96 DPI (1 Pt = 96/72 Px = 1.3333333333333333 Px)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Px(pub f64);

/// Millimeter
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Mm(pub f64);

/// Inch
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Inches(pub f64);

/// Percentage relative to container (0.0 to 100.0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct Percent(pub f64);

impl Pt {
    pub const ZERO: Self = Pt(0.0);

    pub fn to_twips(self) -> Twips {
        Twips((self.0 * 20.0).round() as i64)
    }

    pub fn to_emu(self) -> Emu {
        Emu((self.0 * 12700.0).round() as i64)
    }

    pub fn to_px(self, dpi: f64) -> Px {
        Px(self.0 * (dpi / 72.0))
    }

    pub fn to_inches(self) -> Inches {
        Inches(self.0 / 72.0)
    }

    pub fn to_mm(self) -> Mm {
        Mm(self.0 * (25.4 / 72.0))
    }
}

impl Twips {
    pub const ZERO: Self = Twips(0);

    pub fn to_pt(self) -> Pt {
        Pt(self.0 as f64 / 20.0)
    }

    pub fn to_emu(self) -> Emu {
        Emu(self.0 * 635)
    }

    pub fn to_px(self, dpi: f64) -> Px {
        self.to_pt().to_px(dpi)
    }
}

impl Emu {
    pub const ZERO: Self = Emu(0);

    pub fn to_pt(self) -> Pt {
        Pt(self.0 as f64 / 12700.0)
    }

    pub fn to_twips(self) -> Twips {
        Twips(self.0 / 635)
    }

    pub fn to_px(self, dpi: f64) -> Px {
        Px(self.0 as f64 * dpi / 914400.0)
    }

    pub fn to_inches(self) -> Inches {
        Inches(self.0 as f64 / 914400.0)
    }
}

impl Px {
    pub const ZERO: Self = Px(0.0);

    pub fn to_pt(self, dpi: f64) -> Pt {
        Pt(self.0 * (72.0 / dpi))
    }

    pub fn to_twips(self, dpi: f64) -> Twips {
        self.to_pt(dpi).to_twips()
    }

    pub fn to_emu(self, dpi: f64) -> Emu {
        Emu((self.0 * 914400.0 / dpi).round() as i64)
    }
}

impl Inches {
    pub fn to_pt(self) -> Pt {
        Pt(self.0 * 72.0)
    }

    pub fn to_twips(self) -> Twips {
        Twips((self.0 * 1440.0).round() as i64)
    }

    pub fn to_emu(self) -> Emu {
        Emu((self.0 * 914400.0).round() as i64)
    }

    pub fn to_mm(self) -> Mm {
        Mm(self.0 * 25.4)
    }
}

impl Mm {
    pub fn to_pt(self) -> Pt {
        Pt(self.0 * (72.0 / 25.4))
    }

    pub fn to_twips(self) -> Twips {
        Twips((self.0 * (1440.0 / 25.4)).round() as i64)
    }

    pub fn to_emu(self) -> Emu {
        Emu((self.0 * 36000.0).round() as i64)
    }

    pub fn to_inches(self) -> Inches {
        Inches(self.0 / 25.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        let pt = Pt(72.0);
        assert_eq!(pt.to_inches(), Inches(1.0));
        assert_eq!(pt.to_twips(), Twips(1440));
        assert_eq!(pt.to_emu(), Emu(914400));
        assert!((pt.to_px(96.0).0 - 96.0).abs() < 1e-6);

        let emu = Emu(914400);
        assert_eq!(emu.to_twips(), Twips(1440));
        assert_eq!(emu.to_pt(), Pt(72.0));
        assert_eq!(emu.to_inches(), Inches(1.0));

        let mm = Mm(25.4);
        assert!((mm.to_inches().0 - 1.0).abs() < 1e-6);
        assert!((mm.to_pt().0 - 72.0).abs() < 1e-4);
    }
}
