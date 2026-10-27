use serde::{Deserialize, Serialize};
use wps_core::{Color, Insets, Rect, Twips};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageDimensions {
    pub width: Twips,
    pub height: Twips,
    pub orientation: PageOrientation,
}

impl PageDimensions {
    pub const A4_PORTRAIT: Self = Self {
        width: Twips(11906),  // 210mm
        height: Twips(16838), // 297mm
        orientation: PageOrientation::Portrait,
    };

    pub const LETTER_PORTRAIT: Self = Self {
        width: Twips(12240),  // 8.5 in * 1440
        height: Twips(15840), // 11 in * 1440
        orientation: PageOrientation::Portrait,
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionProperties {
    pub page_dimensions: PageDimensions,
    pub margins: Insets, // in twips
    pub columns_count: usize,
    pub column_spacing: Twips,
    pub title_page_different_header: bool,
}

impl Default for SectionProperties {
    fn default() -> Self {
        Self {
            page_dimensions: PageDimensions::A4_PORTRAIT,
            margins: Insets::new(1440.0, 1440.0, 1440.0, 1440.0), // 1 inch margins = 1440 twips
            columns_count: 1,
            column_spacing: Twips(720),
            title_page_different_header: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Left
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub font_family: String,
    pub size_pt: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color: Color,
    pub highlight: Option<Color>,
    pub revision_id: Option<String>,
}

impl Default for TextRun {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_family: "Inter".to_string(),
            size_pt: 11.0,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: Color::MERCURY_INK,
            highlight: None,
            revision_id: None,
        }
    }
}

impl TextRun {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Default::default()
        }
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.size_pt = size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Page,
    NumPages,
    Date,
    Time,
    Author,
    Toc,
    MergeField(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldElement {
    pub field_type: FieldType,
    pub cached_value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineElement {
    Run(TextRun),
    Field(FieldElement),
    BookmarkStart { name: String },
    BookmarkEnd { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Paragraph {
    pub id: String,
    pub style_name: String,
    pub alignment: TextAlignment,
    pub space_before_twips: Twips,
    pub space_after_twips: Twips,
    pub line_spacing_multiple: f64, // 1.0, 1.15, 1.5, 2.0
    pub indent_left_twips: Twips,
    pub indent_right_twips: Twips,
    pub indent_first_line_twips: Twips,
    pub inlines: Vec<InlineElement>,
}

impl Paragraph {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            style_name: "Normal".to_string(),
            line_spacing_multiple: 1.15,
            ..Default::default()
        }
    }

    pub fn heading(id: &str, level: usize, text: &str) -> Self {
        let size = match level {
            1 => 18.0,
            2 => 14.0,
            3 => 12.0,
            _ => 11.0,
        };
        let mut p = Self::new(id);
        p.style_name = format!("Heading {}", level);
        p.space_before_twips = Twips(240);
        p.space_after_twips = Twips(120);
        p.inlines.push(InlineElement::Run(
            TextRun::new(text).bold().with_size(size).with_color(Color::ACCENT_WRITER)
        ));
        p
    }

    pub fn add_run(&mut self, run: TextRun) {
        self.inlines.push(InlineElement::Run(run));
    }

    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for inline in &self.inlines {
            match inline {
                InlineElement::Run(r) => text.push_str(&r.text),
                InlineElement::Field(f) => text.push_str(&f.cached_value),
                _ => {}
            }
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableCell {
    pub elements: Vec<BlockElement>,
    pub col_span: usize,
    pub row_span: usize,
    pub width_twips: Twips,
    pub background_color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub is_header: bool,
    pub cant_split: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Table {
    pub id: String,
    pub rows: Vec<TableRow>,
    pub alignment: TextAlignment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WrapMode {
    Inline,
    Square,
    Tight,
    BehindText,
    InFrontOfText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingObject {
    pub id: String,
    pub bounds_twips: Rect,
    pub wrap_mode: WrapMode,
    pub image_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockElement {
    Paragraph(Paragraph),
    Table(Table),
    FloatingObject(FloatingObject),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub properties: SectionProperties,
    pub elements: Vec<BlockElement>,
    pub header_elements: Vec<BlockElement>,
    pub footer_elements: Vec<BlockElement>,
}

impl Default for Section {
    fn default() -> Self {
        Self {
            properties: SectionProperties::default(),
            elements: Vec::new(),
            header_elements: Vec::new(),
            footer_elements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Document {
    pub title: String,
    pub author: String,
    pub sections: Vec<Section>,
}

impl Document {
    pub fn new(title: &str, author: &str) -> Self {
        Self {
            title: title.to_string(),
            author: author.to_string(),
            sections: vec![Section::default()],
        }
    }

    pub fn add_paragraph(&mut self, p: Paragraph) {
        if self.sections.is_empty() {
            self.sections.push(Section::default());
        }
        self.sections[0].elements.push(BlockElement::Paragraph(p));
    }

    pub fn total_word_count(&self) -> usize {
        let mut count = 0;
        for sec in &self.sections {
            for elem in &sec.elements {
                if let BlockElement::Paragraph(p) = elem {
                    count += p.plain_text().split_whitespace().count();
                }
            }
        }
        count
    }
}
