use wps_core::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontStyle {
    pub name: String,
    pub size_pt: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color: Color,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            name: "Inter".to_string(),
            size_pt: 11.0,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: Color::MERCURY_INK,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BorderStyle {
    pub left_color: Option<Color>,
    pub right_color: Option<Color>,
    pub top_color: Option<Color>,
    pub bottom_color: Option<Color>,
    pub width_pt: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CellFormat {
    pub font_index: usize,
    pub fill_color: Option<Color>,
    pub border_index: usize,
    pub num_format: String,
}

#[derive(Debug, Clone, Default)]
pub struct StylesSheet {
    pub fonts: Vec<FontStyle>,
    pub borders: Vec<BorderStyle>,
    pub formats: Vec<CellFormat>,
}

impl StylesSheet {
    pub fn new() -> Self {
        let mut ss = Self::default();
        ss.fonts.push(FontStyle::default());
        ss.borders.push(BorderStyle::default());
        ss.formats.push(CellFormat::default());
        ss
    }

    pub fn to_spreadsheet_styles_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">");
        // Fonts
        xml.push_str(&format!("<fonts count=\"{}\">", self.fonts.len()));
        for f in &self.fonts {
            xml.push_str("<font>");
            if f.bold { xml.push_str("<b/>"); }
            if f.italic { xml.push_str("<i/>"); }
            if f.underline { xml.push_str("<u/>"); }
            if f.strike { xml.push_str("<strike/>"); }
            xml.push_str(&format!("<sz val=\"{}\"/>", f.size_pt));
            xml.push_str(&format!("<color rgb=\"{}\"/>", f.color.to_hex().trim_start_matches('#')));
            xml.push_str(&format!("<name val=\"{}\"/>", quick_xml::escape::escape(&f.name)));
            xml.push_str("</font>");
        }
        xml.push_str("</fonts>");

        // Fills
        xml.push_str("<fills count=\"2\"><fill><patternFill patternType=\"none\"/></fill><fill><patternFill patternType=\"gray125\"/></fill></fills>");

        // Borders
        xml.push_str(&format!("<borders count=\"{}\">", self.borders.len()));
        for _ in &self.borders {
            xml.push_str("<border><left/><right/><top/><bottom/><diagonal/></border>");
        }
        xml.push_str("</borders>");

        // CellXfs
        xml.push_str(&format!("<cellXfs count=\"{}\">", self.formats.len()));
        for fmt in &self.formats {
            xml.push_str(&format!("<xf fontId=\"{}\" borderId=\"{}\" fillId=\"0\"/>", fmt.font_index, fmt.border_index));
        }
        xml.push_str("</cellXfs>");
        xml.push_str("</styleSheet>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styles_xml() {
        let mut ss = StylesSheet::new();
        ss.fonts.push(FontStyle {
            name: "JetBrains Mono".to_string(),
            size_pt: 10.0,
            bold: true,
            italic: false,
            underline: false,
            strike: false,
            color: Color::ACCENT_SHEETS,
        });
        let xml = ss.to_spreadsheet_styles_xml();
        assert!(xml.contains("JetBrains Mono"));
        assert!(xml.contains("<b/>"));
    }
}
