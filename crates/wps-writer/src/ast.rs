use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Range;
use wps_core::color::Color;
use wps_core::geometry::Rect;
pub use wps_core::units::Twips;
pub use crate::buffer::{AttributeMap, AttributeSpan, RopeBuffer, RunStyle, UnderlineStyle, VerticalAlignScript};
pub use crate::styles::{ParagraphProperties, StyleSheet, TextAlignment};
pub use crate::numbering::NumberingRegistry;
pub use crate::revisions::{MarkupMode, Revision, RevisionManager, RevisionType};
pub use crate::comments::CommentStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PageOrientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageMargins {
    pub top_twips: u32,
    pub bottom_twips: u32,
    pub left_twips: u32,
    pub right_twips: u32,
    pub header_twips: u32,
    pub footer_twips: u32,
    pub gutter_twips: u32,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top_twips: 1440,    // 1 inch = 1440 twips
            bottom_twips: 1440,
            left_twips: 1440,
            right_twips: 1440,
            header_twips: 720,
            footer_twips: 720,
            gutter_twips: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageSetup {
    pub width_twips: u32,
    pub height_twips: u32,
    pub orientation: PageOrientation,
    pub margins: PageMargins,
}

impl Default for PageSetup {
    fn default() -> Self {
        Self::A4_PORTRAIT
    }
}

impl PageSetup {
    pub const A4_PORTRAIT: Self = Self {
        width_twips: 11906,  // 210mm
        height_twips: 16838, // 297mm
        orientation: PageOrientation::Portrait,
        margins: PageMargins {
            top_twips: 1440,
            bottom_twips: 1440,
            left_twips: 1440,
            right_twips: 1440,
            header_twips: 720,
            footer_twips: 720,
            gutter_twips: 0,
        },
    };

    pub const LETTER_PORTRAIT: Self = Self {
        width_twips: 12240,  // 8.5 in * 1440
        height_twips: 15840, // 11 in * 1440
        orientation: PageOrientation::Portrait,
        margins: PageMargins {
            top_twips: 1440,
            bottom_twips: 1440,
            left_twips: 1440,
            right_twips: 1440,
            header_twips: 720,
            footer_twips: 720,
            gutter_twips: 0,
        },
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnLayout {
    pub count: u8,
    pub space_twips: u32,
    pub equal_width: bool,
    pub separator_line: bool,
}

impl Default for ColumnLayout {
    fn default() -> Self {
        Self {
            count: 1,
            space_twips: 720,
            equal_width: true,
            separator_line: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SectionBreakType {
    #[default]
    NextPage,
    Continuous,
    EvenPage,
    OddPage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HeaderFooterSet {
    pub default: Vec<BlockNode>,
    pub first_page: Option<Vec<BlockNode>>,
    pub even_page: Option<Vec<BlockNode>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DynamicField {
    Page,
    NumPages,
    Date,
    Time,
    Author,
    Toc,
    MergeField(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineObjectAnchor {
    Image { media_id: String, width_twips: u32, height_twips: u32 },
    Field { field_type: DynamicField, cached_value: String },
    FootnoteRef { note_index: usize },
    BookmarkStart { name: String },
    BookmarkEnd { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WrapMode {
    #[default]
    Inline,
    Square,
    Tight,
    TopAndBottom,
    BehindText,
    InFrontOfText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingObject {
    pub id: String,
    pub media_id: String,
    pub bounds_twips: Rect,
    pub wrap_mode: WrapMode,
    pub horizontal_offset_twips: i32,
    pub vertical_offset_twips: i32,
    pub image_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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

impl TextRun {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            font_family: "Inter".to_string(),
            size_pt: 11.0,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: Color::rgb(23, 23, 33),
            highlight: None,
            revision_id: None,
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
pub enum InlineElement {
    Run(TextRun),
    Field(FieldElement),
    BookmarkStart { name: String },
    BookmarkEnd { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldElement {
    pub field_type: DynamicField,
    pub cached_value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParagraphNode {
    pub id: String,
    pub char_range: Range<usize>,
    pub style_id: String,
    pub direct_props: ParagraphProperties,
    pub inlines: Vec<InlineElement>,
    pub inline_objects: Vec<InlineObjectAnchor>,
    pub floating_anchors: Vec<FloatingObject>,
    pub indent_left_twips: Twips,
    pub indent_right_twips: Twips,
    pub indent_first_line_twips: Twips,
    pub space_before_twips: Twips,
    pub space_after_twips: Twips,
    pub line_spacing_multiple: f64,
    pub alignment: TextAlignment,
}

pub type Paragraph = ParagraphNode;

impl ParagraphNode {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            style_id: "Normal".to_string(),
            alignment: TextAlignment::Left,
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
        p.style_id = format!("Heading {}", level);
        p.space_before_twips = Twips(240);
        p.space_after_twips = Twips(120);
        p.inlines.push(InlineElement::Run(
            TextRun::new(text).bold().with_size(size).with_color(Color::rgb(59, 91, 219))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VerticalMergeState {
    #[default]
    None,
    Restart,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CellVAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableCell {
    pub grid_span: u16,
    pub v_merge: VerticalMergeState,
    pub width_twips: Twips,
    pub vertical_align: CellVAlign,
    pub elements: Vec<BlockNode>,
    pub background_color: Option<Color>,
    pub col_span: usize,
    pub row_span: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableRow {
    pub height_twips: Option<u32>,
    pub is_header: bool,
    pub is_header_repeat: bool,
    pub allow_row_split_across_pages: bool,
    pub cant_split: bool,
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableProperties {
    pub width_twips: Option<u32>,
    pub alignment: TextAlignment,
    pub style_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TableNode {
    pub id: String,
    pub grid_columns_twips: Vec<u32>,
    pub props: TableProperties,
    pub rows: Vec<TableRow>,
    pub alignment: TextAlignment,
}

pub type Table = TableNode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockNode {
    Paragraph(ParagraphNode),
    Table(TableNode),
    FloatingObject(FloatingObject),
}

pub type BlockElement = BlockNode;

impl BlockNode {
    pub fn plain_text(&self) -> String {
        match self {
            BlockNode::Paragraph(p) => p.plain_text(),
            BlockNode::Table(t) => {
                let mut text = String::new();
                for r in &t.rows {
                    for c in &r.cells {
                        for elem in &c.elements {
                            text.push_str(&elem.plain_text());
                            text.push(' ');
                        }
                    }
                    text.push('\n');
                }
                text
            }
            BlockNode::FloatingObject(_) => String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub page_setup: PageSetup,
    pub columns: ColumnLayout,
    pub break_type: SectionBreakType,
    pub headers: HeaderFooterSet,
    pub footers: HeaderFooterSet,
    pub elements: Vec<BlockNode>,
    pub header_elements: Vec<BlockNode>,
    pub footer_elements: Vec<BlockNode>,
}

impl Default for Section {
    fn default() -> Self {
        Self {
            page_setup: PageSetup::default(),
            columns: ColumnLayout::default(),
            break_type: SectionBreakType::NextPage,
            headers: HeaderFooterSet::default(),
            footers: HeaderFooterSet::default(),
            elements: Vec::new(),
            header_elements: Vec::new(),
            footer_elements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteBlock {
    pub id: usize,
    pub blocks: Vec<BlockNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub author: String,
    pub buffer: RopeBuffer,
    pub attributes: AttributeMap,
    pub sections: Vec<Section>,
    pub styles: StyleSheet,
    pub numbering: NumberingRegistry,
    pub revisions: RevisionManager,
    pub comments: CommentStore,
    pub bookmarks: BTreeMap<String, Range<usize>>,
    pub footnotes: Vec<NoteBlock>,
    pub endnotes: Vec<NoteBlock>,
    pub media_store: BTreeMap<String, Vec<u8>>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            title: "Untitled Document".to_string(),
            author: "OpenWPS Author".to_string(),
            buffer: RopeBuffer::new(),
            attributes: AttributeMap::new(),
            sections: vec![Section::default()],
            styles: StyleSheet::default(),
            numbering: NumberingRegistry::new(),
            revisions: RevisionManager::default(),
            comments: CommentStore::default(),
            bookmarks: BTreeMap::new(),
            footnotes: Vec::new(),
            endnotes: Vec::new(),
            media_store: BTreeMap::new(),
        }
    }
}

impl Document {
    pub fn new(title: &str, author: &str) -> Self {
        Self {
            title: title.to_string(),
            author: author.to_string(),
            sections: vec![Section::default()],
            ..Default::default()
        }
    }

    pub fn add_paragraph(&mut self, p: ParagraphNode) {
        if self.sections.is_empty() {
            self.sections.push(Section::default());
        }

        // Synchronize plain text into rope buffer & attributes
        let p_text = p.plain_text();
        if !p_text.is_empty() {
            let start = self.buffer.len_chars();
            self.buffer.insert(start, &p_text);
            self.buffer.insert(start + p_text.chars().count(), "\n");

            let style = self.styles.get_style(&p.style_id).map(|s| s.run_style.clone()).unwrap_or_default();
            self.attributes.apply_style(start..start + p_text.chars().count(), style);
        }

        self.sections[0].elements.push(BlockNode::Paragraph(p));
    }

    pub fn total_word_count(&self) -> usize {
        let mut count = 0;
        for sec in &self.sections {
            for elem in &sec.elements {
                if let BlockNode::Paragraph(p) = elem {
                    count += p.plain_text().split_whitespace().count();
                }
            }
        }
        count
    }

    pub fn total_char_count(&self) -> usize {
        self.buffer.len_chars()
    }

    pub fn add_table(&mut self, t: TableNode) {
        if self.sections.is_empty() {
            self.sections.push(Section::default());
        }
        self.sections[0].elements.push(BlockNode::Table(t));
    }
}
