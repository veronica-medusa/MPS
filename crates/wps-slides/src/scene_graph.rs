use crate::animator::SlideTimeline;
use crate::connector::SmartConnector;
use crate::morph::SlideTransition;
use crate::shape::{ParametricShape, ShapeFill};
use crate::theme::SlideTheme;
use serde::{Deserialize, Serialize};
use wps_core::{Color, Point, Rect};
use wps_render::{DisplayList, DrawCommand};

pub const EMU_PER_PT: i64 = 12_700;
pub const EMU_PER_INCH: i64 = 914_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SlideSizeEmu {
    #[default]
    Widescreen16x9, // 12,192,000 x 6,858,000 EMUs (960 x 540 pt)
    Standard4x3,    // 9,144,000 x 6,858,000 EMUs (720 x 540 pt)
    Custom { width: i64, height: i64 },
}

impl SlideSizeEmu {
    pub fn dimensions_emu(&self) -> (i64, i64) {
        match self {
            SlideSizeEmu::Widescreen16x9 => (12_192_000, 6_858_000),
            SlideSizeEmu::Standard4x3 => (9_144_000, 6_858_000),
            SlideSizeEmu::Custom { width, height } => (*width, *height),
        }
    }

    pub fn to_pt_rect(&self) -> Rect {
        let (w_emu, h_emu) = self.dimensions_emu();
        Rect::new(
            0.0,
            0.0,
            w_emu as f64 / EMU_PER_PT as f64,
            h_emu as f64 / EMU_PER_PT as f64,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    Title,
    TitleAndContent,
    SectionHeader,
    TwoContent,
    Comparison,
    TitleOnly,
    Blank,
    ContentWithCaption,
    PictureWithCaption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceholderType {
    Title,
    Subtitle,
    Body,
    Header,
    Footer,
    SlideNumber,
    Date,
    Picture,
    Chart,
    Table,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceholderNode {
    pub id: String,
    pub placeholder_type: PlaceholderType,
    pub index: u32,
    pub bounds: Rect,
    pub default_text: Option<String>,
    pub font_size_pt: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideLayout {
    pub id: String,
    pub name: String,
    pub layout_type: LayoutType,
    pub placeholders: Vec<PlaceholderNode>,
    pub decorative_shapes: Vec<ParametricShape>,
}

impl SlideLayout {
    pub fn title_slide_preset(slide_w: f64, slide_h: f64) -> Self {
        Self {
            id: "layout-title".to_string(),
            name: "Title Slide".to_string(),
            layout_type: LayoutType::Title,
            placeholders: vec![
                PlaceholderNode {
                    id: "ph-title".to_string(),
                    placeholder_type: PlaceholderType::Title,
                    index: 0,
                    bounds: Rect::new(60.0, slide_h * 0.32, slide_w - 120.0, 90.0),
                    default_text: Some("Click to add title".to_string()),
                    font_size_pt: 36.0,
                },
                PlaceholderNode {
                    id: "ph-sub".to_string(),
                    placeholder_type: PlaceholderType::Subtitle,
                    index: 1,
                    bounds: Rect::new(60.0, slide_h * 0.52, slide_w - 120.0, 50.0),
                    default_text: Some("Click to add subtitle".to_string()),
                    font_size_pt: 20.0,
                },
            ],
            decorative_shapes: Vec::new(),
        }
    }

    pub fn title_and_content_preset(slide_w: f64, slide_h: f64) -> Self {
        Self {
            id: "layout-content".to_string(),
            name: "Title and Content".to_string(),
            layout_type: LayoutType::TitleAndContent,
            placeholders: vec![
                PlaceholderNode {
                    id: "ph-title".to_string(),
                    placeholder_type: PlaceholderType::Title,
                    index: 0,
                    bounds: Rect::new(50.0, 40.0, slide_w - 100.0, 60.0),
                    default_text: Some("Click to add title".to_string()),
                    font_size_pt: 32.0,
                },
                PlaceholderNode {
                    id: "ph-body".to_string(),
                    placeholder_type: PlaceholderType::Body,
                    index: 1,
                    bounds: Rect::new(50.0, 120.0, slide_w - 100.0, slide_h - 170.0),
                    default_text: Some("• Click to add text\n• Second bullet point\n• Third bullet point".to_string()),
                    font_size_pt: 18.0,
                },
            ],
            decorative_shapes: Vec::new(),
        }
    }

    pub fn blank_preset() -> Self {
        Self {
            id: "layout-blank".to_string(),
            name: "Blank".to_string(),
            layout_type: LayoutType::Blank,
            placeholders: Vec::new(),
            decorative_shapes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideMaster {
    pub id: String,
    pub theme: SlideTheme,
    pub background_color: Color,
    pub background_shapes: Vec<ParametricShape>,
    pub default_title_font: String,
    pub default_body_font: String,
    pub layouts: Vec<SlideLayout>,
}

impl Default for SlideMaster {
    fn default() -> Self {
        let (w, h) = (960.0, 540.0);
        Self {
            id: "master-1".to_string(),
            theme: SlideTheme::mercury_modern(),
            background_color: Color::WHITE,
            background_shapes: Vec::new(),
            default_title_font: "Inter".to_string(),
            default_body_font: "Inter".to_string(),
            layouts: vec![
                SlideLayout::title_slide_preset(w, h),
                SlideLayout::title_and_content_preset(w, h),
                SlideLayout::blank_preset(),
            ],
        }
    }
}

// ----------------------------------------------------------------------------
// Slide Scene Graph Nodes
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideTableNode {
    pub id: String,
    pub bounds: Rect,
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Vec<String>>,
    pub header_row: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideChartNode {
    pub id: String,
    pub bounds: Rect,
    pub chart_type: String, // "bar", "line", "pie"
    pub title: String,
    pub categories: Vec<String>,
    pub series: Vec<(String, Vec<f64>)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PictureNode {
    pub id: String,
    pub bounds: Rect,
    pub image_id: String,
    pub alt_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlideNode {
    AutoShape(ParametricShape),
    Connector(SmartConnector),
    Table(SlideTableNode),
    Chart(SlideChartNode),
    Picture(PictureNode),
}

impl SlideNode {
    pub fn id(&self) -> &str {
        match self {
            SlideNode::AutoShape(s) => &s.id,
            SlideNode::Connector(c) => &c.id,
            SlideNode::Table(t) => &t.id,
            SlideNode::Chart(c) => &c.id,
            SlideNode::Picture(p) => &p.id,
        }
    }

    pub fn bounds(&self) -> Rect {
        match self {
            SlideNode::AutoShape(s) => s.bounds,
            SlideNode::Connector(_) => Rect::ZERO,
            SlideNode::Table(t) => t.bounds,
            SlideNode::Chart(c) => c.bounds,
            SlideNode::Picture(p) => p.bounds,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideComment {
    pub id: String,
    pub author: String,
    pub text: String,
    pub position: Point,
    pub timestamp_ms: u64,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    pub id: String,
    pub title: String,
    pub layout_id: String,
    pub nodes: Vec<SlideNode>,
    pub show_master_shapes: bool,
    pub speaker_notes: String,
    pub comments: Vec<SlideComment>,
    pub timeline: SlideTimeline,
    pub transition: Option<SlideTransition>,
}

impl Slide {
    pub fn new(id: &str, title: &str, layout_id: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            layout_id: layout_id.to_string(),
            nodes: Vec::new(),
            show_master_shapes: true,
            speaker_notes: String::new(),
            comments: Vec::new(),
            timeline: SlideTimeline::new(),
            transition: None,
        }
    }

    pub fn add_shape(&mut self, shape: ParametricShape) {
        self.nodes.push(SlideNode::AutoShape(shape));
    }

    pub fn add_connector(&mut self, connector: SmartConnector) {
        self.nodes.push(SlideNode::Connector(connector));
    }

    /// Renders the complete 3-Tier scene graph (Master -> Layout -> Slide) to DisplayList
    pub fn render_to_display_list(
        &self,
        deck: &PresentationDeck,
        dl: &mut DisplayList,
    ) {
        let master = &deck.master;
        let slide_rect = deck.slide_size.to_pt_rect();

        // 1. Render Master background
        dl.push(DrawCommand::FillRect {
            rect: slide_rect,
            color: master.background_color,
        });

        // 2. Render Master decorative shapes if enabled
        if self.show_master_shapes {
            for shape in &master.background_shapes {
                shape.render_to_display_list(dl);
            }
        }

        // 3. Render Layout Placeholders
        if let Some(layout) = master.layouts.iter().find(|l| l.id == self.layout_id) {
            for ph in &layout.placeholders {
                if let Some(text) = &ph.default_text {
                    let text_color = master.theme.color_scheme.dark1;
                    dl.push(DrawCommand::DrawTextRun {
                        text: text.clone(),
                        font_family: master.theme.font_scheme.major_font.clone(),
                        font_size_px: ph.font_size_pt * 1.333,
                        bold: ph.placeholder_type == PlaceholderType::Title,
                        italic: false,
                        color: text_color,
                        origin: Point::new(ph.bounds.x, ph.bounds.y + ph.font_size_pt * 1.2),
                    });
                }
            }
        }

        // 4. Render Slide Nodes
        let mut shape_map: std::collections::HashMap<String, Rect> = std::collections::HashMap::new();
        for node in &self.nodes {
            if let SlideNode::AutoShape(s) = node {
                shape_map.insert(s.id.clone(), s.bounds);
            }
        }

        for node in &self.nodes {
            match node {
                SlideNode::AutoShape(s) => {
                    s.render_to_display_list(dl);
                }
                SlideNode::Connector(c) => {
                    let start_b = shape_map.get(&c.start_shape_id).copied().unwrap_or(Rect::ZERO);
                    let end_b = shape_map.get(&c.end_shape_id).copied().unwrap_or(Rect::ZERO);
                    c.render_to_display_list(&start_b, &end_b, dl);
                }
                SlideNode::Table(t) => {
                    // Render Table Grid
                    let row_h = t.bounds.height / t.rows.max(1) as f64;
                    let col_w = t.bounds.width / t.cols.max(1) as f64;
                    for r in 0..t.rows {
                        for c in 0..t.cols {
                            let cell_rect = Rect::new(t.bounds.x + c as f64 * col_w, t.bounds.y + r as f64 * row_h, col_w, row_h);
                            let bg_color = if r == 0 && t.header_row { master.theme.color_scheme.accent1 } else { Color::WHITE };
                            dl.push(DrawCommand::FillRect { rect: cell_rect, color: bg_color });
                            dl.push(DrawCommand::StrokeRect { rect: cell_rect, color: Color::from_hex("#e2e4ea").unwrap_or(Color::MERCURY_INK), width: 1.0 });

                            if let Some(row_data) = t.cells.get(r) {
                                if let Some(val) = row_data.get(c) {
                                    dl.push(DrawCommand::DrawTextRun {
                                        text: val.clone(),
                                        font_family: master.theme.font_scheme.minor_font.clone(),
                                        font_size_px: 13.0,
                                        bold: r == 0 && t.header_row,
                                        italic: false,
                                        color: if r == 0 && t.header_row { Color::WHITE } else { master.theme.color_scheme.dark1 },
                                        origin: Point::new(cell_rect.x + 8.0, cell_rect.y + row_h / 2.0 + 4.0),
                                    });
                                }
                            }
                        }
                    }
                }
                SlideNode::Chart(chart) => {
                    // Render Vector Chart Container & Simple Bar Visualizer
                    dl.push(DrawCommand::FillRect { rect: chart.bounds, color: Color::WHITE });
                    dl.push(DrawCommand::StrokeRect { rect: chart.bounds, color: Color::from_hex("#e2e4ea").unwrap_or(Color::MERCURY_INK), width: 1.0 });

                    // Title
                    dl.push(DrawCommand::DrawTextRun {
                        text: chart.title.clone(),
                        font_family: master.theme.font_scheme.major_font.clone(),
                        font_size_px: 14.0,
                        bold: true,
                        italic: false,
                        color: master.theme.color_scheme.dark1,
                        origin: Point::new(chart.bounds.x + 12.0, chart.bounds.y + 24.0),
                    });

                    // Bars
                    let n = chart.categories.len().max(1);
                    let bar_w = (chart.bounds.width - 40.0) / (n as f64 * 1.5);
                    for (i, cat) in chart.categories.iter().enumerate() {
                        let bx = chart.bounds.x + 30.0 + i as f64 * (bar_w * 1.5);
                        let bh = (chart.bounds.height - 80.0) * ((i + 1) as f64 / (n + 1) as f64);
                        let by = chart.bounds.y + chart.bounds.height - 30.0 - bh;

                        dl.push(DrawCommand::FillRect {
                            rect: Rect::new(bx, by, bar_w, bh),
                            color: master.theme.color_scheme.accent1,
                        });
                        dl.push(DrawCommand::DrawTextRun {
                            text: cat.clone(),
                            font_family: master.theme.font_scheme.minor_font.clone(),
                            font_size_px: 11.0,
                            bold: false,
                            italic: false,
                            color: master.theme.color_scheme.dark1,
                            origin: Point::new(bx, chart.bounds.y + chart.bounds.height - 12.0),
                        });
                    }
                }
                SlideNode::Picture(p) => {
                    dl.push(DrawCommand::DrawImage {
                        image_id: p.image_id.clone(),
                        dest_rect: p.bounds,
                        clip: None,
                    });
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Presentation Deck
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresentationDeck {
    pub title: String,
    pub slide_size: SlideSizeEmu,
    pub master: SlideMaster,
    pub slides: Vec<Slide>,
}

impl PresentationDeck {
    pub fn new(title: &str) -> Self {
        let master = SlideMaster::default();
        let first_slide = Slide::new("slide-1", title, "layout-title");
        Self {
            title: title.to_string(),
            slide_size: SlideSizeEmu::Widescreen16x9,
            master,
            slides: vec![first_slide],
        }
    }

    pub fn add_slide(&mut self, slide: Slide) {
        self.slides.push(slide);
    }

    /// Resolves cascading properties for a shape node (Node -> Layout -> Master -> Theme)
    pub fn resolve_node_properties(
        &self,
        node: &ParametricShape,
        _slide_idx: usize,
    ) -> (Color, Color, f64, String) {
        let theme = &self.master.theme;
        let fill_color = match &node.fill {
            ShapeFill::Solid(c) => *c,
            _ => theme.color_scheme.accent1,
        };
        let stroke_color = node.stroke.as_ref().map(|s| s.color).unwrap_or(theme.color_scheme.dark1);
        let stroke_width = node.stroke.as_ref().map(|s| s.width_px).unwrap_or(1.0);
        let font_family = theme.font_scheme.minor_font.clone();

        (fill_color, stroke_color, stroke_width, font_family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::ShapeGeometryType;

    #[test]
    fn test_slide_master_inheritance() {
        let deck = PresentationDeck::new("Q1 Deck");
        let mut dl = DisplayList::new();
        deck.slides[0].render_to_display_list(&deck, &mut dl);

        assert!(!dl.is_empty());
        assert_eq!(deck.slides.len(), 1);
        let (_fill, _stroke, width, font) = deck.resolve_node_properties(
            &ParametricShape::new("s1", ShapeGeometryType::Rectangle, Rect::new(0.0, 0.0, 100.0, 100.0)),
            0,
        );
        assert_eq!(width, 1.0);
        assert!(!font.is_empty());
    }
}
