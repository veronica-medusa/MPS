use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use wps_core::{Color, Point, Rect};
use wps_render::{DisplayList, DrawCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeGeometryType {
    // Basic Shapes
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Triangle,
    RightTriangle,
    Diamond,
    Parallelogram,
    Trapezoid,
    Hexagon,
    Octagon,
    Decagon,
    Dodecagon,
    Can,
    Cube,
    Donut,
    BlockArc,
    Heart,
    LightningBolt,
    Sun,
    Moon,
    Cloud,
    Teardrop,
    Cross,
    Plaque,

    // Stars & Banners
    Star4,
    Star5,
    Star6,
    Star8,
    Star10,
    Star12,
    Star16,
    Star24,
    Star32,

    // Block Arrows
    RightArrow,
    LeftArrow,
    UpArrow,
    DownArrow,
    LeftRightArrow,
    UpDownArrow,
    QuadArrow,
    CurvedRightArrow,
    CurvedLeftArrow,
    UTurnArrow,
    Chevron,
    NotchedRightArrow,
    StripedRightArrow,

    // Callouts
    RectCallout,
    RoundedRectCallout,
    CloudCallout,
    SpeechBubble,

    // Flowchart
    FlowchartProcess,
    FlowchartDecision,
    FlowchartDocument,
    FlowchartData,
    FlowchartPredefinedProcess,
    FlowchartTerminator,
    FlowchartPreparation,
    FlowchartManualOperation,
    FlowchartMerge,
    FlowchartDelay,

    // Freeform
    Freeform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeFill {
    None,
    Solid(Color),
    LinearGradient {
        angle_deg: f64,
        stops: Vec<(f64, Color)>,
    },
}

impl Default for ShapeFill {
    fn default() -> Self {
        ShapeFill::Solid(Color::MERCURY_PRIMARY)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeStroke {
    pub color: Color,
    pub width_px: f64,
    pub dash_style: String, // "solid", "dash", "dot"
}

impl Default for ShapeStroke {
    fn default() -> Self {
        Self {
            color: Color::MERCURY_INK,
            width_px: 1.0,
            dash_style: "solid".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextAutoFit {
    #[default]
    None,
    ShrinkTextOnOverflow,
    ResizeShapeToFitText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextAlignment {
    Left,
    #[default]
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VerticalAlign {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub font_family: String,
    pub font_size_pt: f64,
    pub bold: bool,
    pub italic: bool,
    pub color: Color,
}

impl Default for TextRun {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_family: "Inter".to_string(),
            font_size_pt: 14.0,
            bold: false,
            italic: false,
            color: Color::WHITE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TextParagraph {
    pub runs: Vec<TextRun>,
    pub alignment: TextAlignment,
    pub line_spacing: f64,
}

impl TextParagraph {
    pub fn plain_text(text: &str, font_size_pt: f64, color: Color) -> Self {
        Self {
            runs: vec![TextRun {
                text: text.to_string(),
                font_family: "Inter".to_string(),
                font_size_pt,
                bold: false,
                italic: false,
                color,
            }],
            alignment: TextAlignment::Center,
            line_spacing: 1.2,
        }
    }

    pub fn to_plain_string(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join("")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TextFrame {
    pub paragraphs: Vec<TextParagraph>,
    pub auto_fit: TextAutoFit,
    pub margin_pt: f64,
    pub vertical_align: VerticalAlign,
}

impl TextFrame {
    pub fn simple(text: &str, font_size_pt: f64, color: Color) -> Self {
        Self {
            paragraphs: vec![TextParagraph::plain_text(text, font_size_pt, color)],
            auto_fit: TextAutoFit::ShrinkTextOnOverflow,
            margin_pt: 7.0,
            vertical_align: VerticalAlign::Middle,
        }
    }

    pub fn to_plain_string(&self) -> String {
        self.paragraphs.iter().map(|p| p.to_plain_string()).collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentHandle {
    pub index: usize,
    pub position: Point,
    pub min_val: f64,
    pub max_val: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParametricShape {
    pub id: String,
    pub name: String,
    pub geometry: ShapeGeometryType,
    pub bounds: Rect,
    pub rotation_deg: f64,
    pub fill: ShapeFill,
    pub stroke: Option<ShapeStroke>,
    pub adjustment_values: Vec<f64>,
    pub text_frame: Option<TextFrame>,
    pub freeform_points: Option<Vec<Point>>,
}

impl ParametricShape {
    pub fn new(id: &str, geometry: ShapeGeometryType, bounds: Rect) -> Self {
        let default_adj = match geometry {
            ShapeGeometryType::RoundedRectangle => vec![0.15],
            ShapeGeometryType::Triangle => vec![0.5],
            ShapeGeometryType::Parallelogram => vec![0.2],
            ShapeGeometryType::Trapezoid => vec![0.2],
            ShapeGeometryType::Hexagon => vec![0.25],
            ShapeGeometryType::Octagon => vec![0.3],
            ShapeGeometryType::Decagon => vec![0.25],
            ShapeGeometryType::Dodecagon => vec![0.25],
            ShapeGeometryType::Can => vec![0.15],
            ShapeGeometryType::Cube => vec![0.2],
            ShapeGeometryType::Donut => vec![0.35],
            ShapeGeometryType::BlockArc => vec![0.0, 180.0, 0.25],
            ShapeGeometryType::Moon => vec![0.5],
            ShapeGeometryType::Star4 => vec![0.38],
            ShapeGeometryType::Star5 => vec![0.38],
            ShapeGeometryType::Star6 => vec![0.4],
            ShapeGeometryType::Star8 => vec![0.4],
            ShapeGeometryType::Star10 => vec![0.42],
            ShapeGeometryType::Star12 => vec![0.42],
            ShapeGeometryType::Star16 => vec![0.45],
            ShapeGeometryType::Star24 => vec![0.45],
            ShapeGeometryType::Star32 => vec![0.45],
            ShapeGeometryType::RightArrow
            | ShapeGeometryType::LeftArrow
            | ShapeGeometryType::UpArrow
            | ShapeGeometryType::DownArrow => vec![0.5, 0.5],
            ShapeGeometryType::LeftRightArrow | ShapeGeometryType::UpDownArrow => vec![0.5, 0.5],
            ShapeGeometryType::QuadArrow => vec![0.5, 0.5],
            ShapeGeometryType::Chevron => vec![0.5],
            ShapeGeometryType::NotchedRightArrow => vec![0.5, 0.5, 0.2],
            ShapeGeometryType::StripedRightArrow => vec![0.5, 0.5],
            ShapeGeometryType::RectCallout | ShapeGeometryType::RoundedRectCallout => vec![0.2, 1.3],
            ShapeGeometryType::SpeechBubble => vec![0.3, 1.2],
            _ => vec![0.2],
        };

        Self {
            id: id.to_string(),
            name: format!("{:?}", geometry),
            geometry,
            bounds,
            rotation_deg: 0.0,
            fill: ShapeFill::Solid(Color::MERCURY_PRIMARY),
            stroke: Some(ShapeStroke::default()),
            adjustment_values: default_adj,
            text_frame: None,
            freeform_points: None,
        }
    }

    /// Convenience builder with a text label
    pub fn with_text(mut self, text: &str) -> Self {
        self.text_frame = Some(TextFrame::simple(text, 14.0, Color::WHITE));
        self
    }

    /// Draggable yellow diamond adjustment handles in stage coordinates
    pub fn get_adjustment_handles(&self) -> Vec<AdjustmentHandle> {
        let b = &self.bounds;
        let mut handles = Vec::new();

        match self.geometry {
            ShapeGeometryType::RoundedRectangle => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.15);
                let radius = adj.clamp(0.0, 0.5) * b.width.min(b.height);
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(b.x + radius, b.y),
                    min_val: 0.0,
                    max_val: 0.5,
                });
            }
            ShapeGeometryType::Triangle => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.5);
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(b.x + adj.clamp(0.0, 1.0) * b.width, b.y),
                    min_val: 0.0,
                    max_val: 1.0,
                });
            }
            ShapeGeometryType::Parallelogram => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.2);
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(b.x + adj.clamp(0.0, 0.8) * b.width, b.y),
                    min_val: 0.0,
                    max_val: 0.8,
                });
            }
            ShapeGeometryType::Trapezoid => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.2);
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(b.x + adj.clamp(0.0, 0.45) * b.width, b.y),
                    min_val: 0.0,
                    max_val: 0.45,
                });
            }
            ShapeGeometryType::Donut => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.35);
                let cx = b.x + b.width / 2.0;
                let cy = b.y + b.height / 2.0;
                let r = b.width.min(b.height) / 2.0;
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(cx + adj.clamp(0.05, 0.95) * r, cy),
                    min_val: 0.05,
                    max_val: 0.95,
                });
            }
            ShapeGeometryType::Star4
            | ShapeGeometryType::Star5
            | ShapeGeometryType::Star6
            | ShapeGeometryType::Star8
            | ShapeGeometryType::Star10
            | ShapeGeometryType::Star12
            | ShapeGeometryType::Star16
            | ShapeGeometryType::Star24
            | ShapeGeometryType::Star32 => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.38);
                let cx = b.x + b.width / 2.0;
                let cy = b.y + b.height / 2.0;
                let r = b.width.min(b.height) / 2.0;
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(cx, cy - adj.clamp(0.1, 0.9) * r),
                    min_val: 0.1,
                    max_val: 0.9,
                });
            }
            ShapeGeometryType::RightArrow => {
                let head_w = self.adjustment_values.first().copied().unwrap_or(0.5);
                let shaft_w = self.adjustment_values.get(1).copied().unwrap_or(0.5);
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(b.x + (1.0 - head_w.clamp(0.1, 0.9)) * b.width, b.y),
                    min_val: 0.1,
                    max_val: 0.9,
                });
                handles.push(AdjustmentHandle {
                    index: 1,
                    position: Point::new(b.x, b.y + (0.5 - shaft_w.clamp(0.1, 0.9) * 0.25) * b.height),
                    min_val: 0.1,
                    max_val: 0.9,
                });
            }
            ShapeGeometryType::RectCallout | ShapeGeometryType::SpeechBubble => {
                let tip_x = self.adjustment_values.first().copied().unwrap_or(0.2);
                let tip_y = self.adjustment_values.get(1).copied().unwrap_or(1.3);
                handles.push(AdjustmentHandle {
                    index: 0,
                    position: Point::new(b.x + tip_x * b.width, b.y + tip_y * b.height),
                    min_val: -0.5,
                    max_val: 2.0,
                });
            }
            _ => {}
        }

        handles
    }

    /// Updates shape parameter from dragged yellow diamond handle
    pub fn set_adjustment_handle(&mut self, handle_idx: usize, pt: Point) {
        let b = &self.bounds;
        match self.geometry {
            ShapeGeometryType::RoundedRectangle => {
                if handle_idx == 0 && b.width.min(b.height) > 0.0 {
                    let fraction = ((pt.x - b.x) / b.width.min(b.height)).clamp(0.0, 0.5);
                    if self.adjustment_values.is_empty() {
                        self.adjustment_values.push(fraction);
                    } else {
                        self.adjustment_values[0] = fraction;
                    }
                }
            }
            ShapeGeometryType::Triangle => {
                if handle_idx == 0 && b.width > 0.0 {
                    let fraction = ((pt.x - b.x) / b.width).clamp(0.0, 1.0);
                    if self.adjustment_values.is_empty() {
                        self.adjustment_values.push(fraction);
                    } else {
                        self.adjustment_values[0] = fraction;
                    }
                }
            }
            ShapeGeometryType::Parallelogram | ShapeGeometryType::Trapezoid => {
                if handle_idx == 0 && b.width > 0.0 {
                    let fraction = ((pt.x - b.x) / b.width).clamp(0.0, 0.45);
                    if self.adjustment_values.is_empty() {
                        self.adjustment_values.push(fraction);
                    } else {
                        self.adjustment_values[0] = fraction;
                    }
                }
            }
            ShapeGeometryType::Donut => {
                if handle_idx == 0 {
                    let cx = b.x + b.width / 2.0;
                    let cy = b.y + b.height / 2.0;
                    let r = b.width.min(b.height) / 2.0;
                    let dist = ((pt.x - cx).powi(2) + (pt.y - cy).powi(2)).sqrt();
                    let frac = (dist / r).clamp(0.05, 0.95);
                    if self.adjustment_values.is_empty() {
                        self.adjustment_values.push(frac);
                    } else {
                        self.adjustment_values[0] = frac;
                    }
                }
            }
            ShapeGeometryType::Star4
            | ShapeGeometryType::Star5
            | ShapeGeometryType::Star6
            | ShapeGeometryType::Star8
            | ShapeGeometryType::Star10
            | ShapeGeometryType::Star12
            | ShapeGeometryType::Star16
            | ShapeGeometryType::Star24
            | ShapeGeometryType::Star32 => {
                if handle_idx == 0 {
                    let cy = b.y + b.height / 2.0;
                    let r = b.width.min(b.height) / 2.0;
                    let frac = ((cy - pt.y) / r).clamp(0.1, 0.9);
                    if self.adjustment_values.is_empty() {
                        self.adjustment_values.push(frac);
                    } else {
                        self.adjustment_values[0] = frac;
                    }
                }
            }
            ShapeGeometryType::RightArrow => {
                if handle_idx == 0 && b.width > 0.0 {
                    let frac = (1.0 - (pt.x - b.x) / b.width).clamp(0.1, 0.9);
                    if self.adjustment_values.is_empty() {
                        self.adjustment_values.push(frac);
                    } else {
                        self.adjustment_values[0] = frac;
                    }
                } else if handle_idx == 1 && b.height > 0.0 {
                    let frac = ((0.5 - (pt.y - b.y) / b.height).abs() / 0.25).clamp(0.1, 0.9);
                    while self.adjustment_values.len() < 2 {
                        self.adjustment_values.push(0.5);
                    }
                    self.adjustment_values[1] = frac;
                }
            }
            ShapeGeometryType::RectCallout | ShapeGeometryType::SpeechBubble => {
                if handle_idx == 0 && b.width > 0.0 && b.height > 0.0 {
                    let tx = (pt.x - b.x) / b.width;
                    let ty = (pt.y - b.y) / b.height;
                    self.adjustment_values = vec![tx, ty];
                }
            }
            _ => {}
        }
    }

    /// Computes boundary vertices for vector rendering and hit-testing
    pub fn generate_outline_points(&self) -> Vec<Point> {
        if let Some(pts) = &self.freeform_points {
            if !pts.is_empty() {
                return pts.clone();
            }
        }

        let b = &self.bounds;
        let x = b.x;
        let y = b.y;
        let w = b.width;
        let h = b.height;
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;

        match self.geometry {
            ShapeGeometryType::Rectangle | ShapeGeometryType::FlowchartProcess => {
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(x + w, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::RoundedRectangle => {
                let adj = self.adjustment_values.first().copied().unwrap_or(0.15).clamp(0.0, 0.5);
                let r = adj * w.min(h);
                let mut pts = Vec::new();
                let steps = 6;
                // Top-right corner
                for i in 0..=steps {
                    let a = -PI / 2.0 + (PI / 2.0) * (i as f64 / steps as f64);
                    pts.push(Point::new(x + w - r + r * a.cos(), y + r + r * a.sin()));
                }
                // Bottom-right corner
                for i in 0..=steps {
                    let a = 0.0 + (PI / 2.0) * (i as f64 / steps as f64);
                    pts.push(Point::new(x + w - r + r * a.cos(), y + h - r + r * a.sin()));
                }
                // Bottom-left corner
                for i in 0..=steps {
                    let a = PI / 2.0 + (PI / 2.0) * (i as f64 / steps as f64);
                    pts.push(Point::new(x + r + r * a.cos(), y + h - r + r * a.sin()));
                }
                // Top-left corner
                for i in 0..=steps {
                    let a = PI + (PI / 2.0) * (i as f64 / steps as f64);
                    pts.push(Point::new(x + r + r * a.cos(), y + r + r * a.sin()));
                }
                pts
            }
            ShapeGeometryType::Ellipse | ShapeGeometryType::Sun => {
                let rx = w / 2.0;
                let ry = h / 2.0;
                let steps = 36;
                let mut pts = Vec::with_capacity(steps);
                for i in 0..steps {
                    let a = 2.0 * PI * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + rx * a.cos(), cy + ry * a.sin()));
                }
                pts
            }
            ShapeGeometryType::Triangle => {
                let apex_frac = self.adjustment_values.first().copied().unwrap_or(0.5).clamp(0.0, 1.0);
                vec![
                    Point::new(x + w * apex_frac, y),
                    Point::new(x + w, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::RightTriangle => {
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::Diamond | ShapeGeometryType::FlowchartDecision => {
                vec![
                    Point::new(cx, y),
                    Point::new(x + w, cy),
                    Point::new(cx, y + h),
                    Point::new(x, cy),
                ]
            }
            ShapeGeometryType::Parallelogram => {
                let slant = self.adjustment_values.first().copied().unwrap_or(0.2).clamp(0.0, 0.8) * w;
                vec![
                    Point::new(x + slant, y),
                    Point::new(x + w, y),
                    Point::new(x + w - slant, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::Trapezoid => {
                let indent = self.adjustment_values.first().copied().unwrap_or(0.2).clamp(0.0, 0.45) * w;
                vec![
                    Point::new(x + indent, y),
                    Point::new(x + w - indent, y),
                    Point::new(x + w, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::Hexagon => {
                let ind = self.adjustment_values.first().copied().unwrap_or(0.25).clamp(0.05, 0.45) * w;
                vec![
                    Point::new(x + ind, y),
                    Point::new(x + w - ind, y),
                    Point::new(x + w, cy),
                    Point::new(x + w - ind, y + h),
                    Point::new(x + ind, y + h),
                    Point::new(x, cy),
                ]
            }
            ShapeGeometryType::Octagon => {
                let ind = self.adjustment_values.first().copied().unwrap_or(0.29).clamp(0.05, 0.45) * w.min(h);
                vec![
                    Point::new(x + ind, y),
                    Point::new(x + w - ind, y),
                    Point::new(x + w, y + ind),
                    Point::new(x + w, y + h - ind),
                    Point::new(x + w - ind, y + h),
                    Point::new(x + ind, y + h),
                    Point::new(x, y + h - ind),
                    Point::new(x, y + ind),
                ]
            }
            ShapeGeometryType::Decagon => {
                Self::regular_polygon_points(cx, cy, w / 2.0, h / 2.0, 10)
            }
            ShapeGeometryType::Dodecagon => {
                Self::regular_polygon_points(cx, cy, w / 2.0, h / 2.0, 12)
            }
            ShapeGeometryType::Can => {
                let rh = self.adjustment_values.first().copied().unwrap_or(0.15).clamp(0.05, 0.4) * h;
                let mut pts = Vec::new();
                for i in 0..=12 {
                    let a = PI + PI * (i as f64 / 12.0);
                    pts.push(Point::new(cx + (w / 2.0) * a.cos(), y + rh + rh * a.sin()));
                }
                for i in 0..=12 {
                    let a = 0.0 + PI * (i as f64 / 12.0);
                    pts.push(Point::new(cx + (w / 2.0) * a.cos(), y + h - rh + rh * a.sin()));
                }
                pts
            }
            ShapeGeometryType::Cube => {
                let d = self.adjustment_values.first().copied().unwrap_or(0.2).clamp(0.05, 0.5) * w.min(h);
                vec![
                    Point::new(x + d, y),
                    Point::new(x + w, y),
                    Point::new(x + w, y + h - d),
                    Point::new(x + w - d, y + h),
                    Point::new(x, y + h),
                    Point::new(x, y + d),
                ]
            }
            ShapeGeometryType::Donut => {
                let inner_frac = self.adjustment_values.first().copied().unwrap_or(0.35).clamp(0.05, 0.95);
                let rx = w / 2.0;
                let ry = h / 2.0;
                let irx = rx * inner_frac;
                let iry = ry * inner_frac;
                let steps = 24;
                let mut pts = Vec::new();
                for i in 0..steps {
                    let a = 2.0 * PI * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + rx * a.cos(), cy + ry * a.sin()));
                }
                // Return to start and wind reverse for inner hole
                for i in (0..steps).rev() {
                    let a = 2.0 * PI * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + irx * a.cos(), cy + iry * a.sin()));
                }
                pts
            }
            ShapeGeometryType::BlockArc => {
                let steps = 20;
                let mut pts = Vec::new();
                for i in 0..=steps {
                    let a = PI + (PI) * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + (w / 2.0) * a.cos(), cy + (h / 2.0) * a.sin()));
                }
                for i in (0..=steps).rev() {
                    let a = PI + (PI) * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + (w * 0.35) * a.cos(), cy + (h * 0.35) * a.sin()));
                }
                pts
            }
            ShapeGeometryType::Heart => {
                let mut pts = Vec::new();
                let steps = 36;
                for i in 0..steps {
                    let t = 2.0 * PI * (i as f64 / steps as f64);
                    let hx = 16.0 * t.sin().powi(3);
                    let hy = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
                    pts.push(Point::new(cx + (hx / 34.0) * w, cy + (hy / 34.0) * h));
                }
                pts
            }
            ShapeGeometryType::LightningBolt => {
                vec![
                    Point::new(x + w * 0.55, y),
                    Point::new(x + w * 0.15, y + h * 0.55),
                    Point::new(x + w * 0.45, y + h * 0.55),
                    Point::new(x + w * 0.35, y + h),
                    Point::new(x + w * 0.85, y + h * 0.42),
                    Point::new(x + w * 0.55, y + h * 0.42),
                ]
            }
            ShapeGeometryType::Moon => {
                let depth = self.adjustment_values.first().copied().unwrap_or(0.5).clamp(0.1, 0.9);
                let rx = w / 2.0;
                let ry = h / 2.0;
                let steps = 18;
                let mut pts = Vec::new();
                for i in 0..=steps {
                    let a = -PI / 2.0 + PI * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + rx * a.cos(), cy + ry * a.sin()));
                }
                for i in (0..=steps).rev() {
                    let a = -PI / 2.0 + PI * (i as f64 / steps as f64);
                    pts.push(Point::new(cx + rx * depth * a.cos(), cy + ry * a.sin()));
                }
                pts
            }
            ShapeGeometryType::Cloud => {
                let mut pts = Vec::new();
                let centers = [
                    (0.3, 0.65, 0.25),
                    (0.5, 0.4, 0.35),
                    (0.7, 0.6, 0.28),
                    (0.5, 0.75, 0.22),
                ];
                for (fx, fy, fr) in &centers {
                    let ccx = x + fx * w;
                    let ccy = y + fy * h;
                    let r = fr * w.min(h);
                    for i in 0..10 {
                        let a = 2.0 * PI * (i as f64 / 10.0);
                        pts.push(Point::new(ccx + r * a.cos(), ccy + r * a.sin()));
                    }
                }
                pts
            }
            ShapeGeometryType::Teardrop => {
                let mut pts = Vec::new();
                for i in 0..24 {
                    let a = 2.0 * PI * (i as f64 / 24.0);
                    let r = (w / 2.0) * (1.0 - a.sin()).abs();
                    pts.push(Point::new(cx + r * a.cos(), cy + (h / 2.0) * a.sin()));
                }
                pts
            }
            ShapeGeometryType::Cross => {
                let t = 0.28;
                vec![
                    Point::new(cx - w * t / 2.0, y),
                    Point::new(cx + w * t / 2.0, y),
                    Point::new(cx + w * t / 2.0, cy - h * t / 2.0),
                    Point::new(x + w, cy - h * t / 2.0),
                    Point::new(x + w, cy + h * t / 2.0),
                    Point::new(cx + w * t / 2.0, cy + h * t / 2.0),
                    Point::new(cx + w * t / 2.0, y + h),
                    Point::new(cx - w * t / 2.0, y + h),
                    Point::new(cx - w * t / 2.0, cy + h * t / 2.0),
                    Point::new(x, cy + h * t / 2.0),
                    Point::new(x, cy - h * t / 2.0),
                    Point::new(cx - w * t / 2.0, cy - h * t / 2.0),
                ]
            }
            ShapeGeometryType::Plaque => {
                let r = 0.2 * w.min(h);
                vec![
                    Point::new(x + r, y),
                    Point::new(x + w - r, y),
                    Point::new(x + w, y + r),
                    Point::new(x + w, y + h - r),
                    Point::new(x + w - r, y + h),
                    Point::new(x + r, y + h),
                    Point::new(x, y + h - r),
                    Point::new(x, y + r),
                ]
            }
            // Stars
            ShapeGeometryType::Star4 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 4, self.adjustment_values.first().copied().unwrap_or(0.38)),
            ShapeGeometryType::Star5 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 5, self.adjustment_values.first().copied().unwrap_or(0.38)),
            ShapeGeometryType::Star6 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 6, self.adjustment_values.first().copied().unwrap_or(0.40)),
            ShapeGeometryType::Star8 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 8, self.adjustment_values.first().copied().unwrap_or(0.40)),
            ShapeGeometryType::Star10 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 10, self.adjustment_values.first().copied().unwrap_or(0.42)),
            ShapeGeometryType::Star12 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 12, self.adjustment_values.first().copied().unwrap_or(0.42)),
            ShapeGeometryType::Star16 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 16, self.adjustment_values.first().copied().unwrap_or(0.45)),
            ShapeGeometryType::Star24 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 24, self.adjustment_values.first().copied().unwrap_or(0.45)),
            ShapeGeometryType::Star32 => Self::star_points(cx, cy, w / 2.0, h / 2.0, 32, self.adjustment_values.first().copied().unwrap_or(0.45)),

            // Block Arrows
            ShapeGeometryType::RightArrow => {
                let head_w = self.adjustment_values.first().copied().unwrap_or(0.5).clamp(0.1, 0.9) * w;
                let shaft_w = self.adjustment_values.get(1).copied().unwrap_or(0.5).clamp(0.1, 0.9) * (h / 2.0);
                vec![
                    Point::new(x, cy - shaft_w / 2.0),
                    Point::new(x + w - head_w, cy - shaft_w / 2.0),
                    Point::new(x + w - head_w, y),
                    Point::new(x + w, cy),
                    Point::new(x + w - head_w, y + h),
                    Point::new(x + w - head_w, cy + shaft_w / 2.0),
                    Point::new(x, cy + shaft_w / 2.0),
                ]
            }
            ShapeGeometryType::LeftArrow => {
                let head_w = self.adjustment_values.first().copied().unwrap_or(0.5).clamp(0.1, 0.9) * w;
                let shaft_w = self.adjustment_values.get(1).copied().unwrap_or(0.5).clamp(0.1, 0.9) * (h / 2.0);
                vec![
                    Point::new(x + head_w, y),
                    Point::new(x + head_w, cy - shaft_w / 2.0),
                    Point::new(x + w, cy - shaft_w / 2.0),
                    Point::new(x + w, cy + shaft_w / 2.0),
                    Point::new(x + head_w, cy + shaft_w / 2.0),
                    Point::new(x + head_w, y + h),
                    Point::new(x, cy),
                ]
            }
            ShapeGeometryType::UpArrow => {
                let head_h = self.adjustment_values.first().copied().unwrap_or(0.5).clamp(0.1, 0.9) * h;
                let shaft_w = self.adjustment_values.get(1).copied().unwrap_or(0.5).clamp(0.1, 0.9) * (w / 2.0);
                vec![
                    Point::new(cx, y),
                    Point::new(x + w, y + head_h),
                    Point::new(cx + shaft_w / 2.0, y + head_h),
                    Point::new(cx + shaft_w / 2.0, y + h),
                    Point::new(cx - shaft_w / 2.0, y + h),
                    Point::new(cx - shaft_w / 2.0, y + head_h),
                    Point::new(x, y + head_h),
                ]
            }
            ShapeGeometryType::DownArrow => {
                let head_h = self.adjustment_values.first().copied().unwrap_or(0.5).clamp(0.1, 0.9) * h;
                let shaft_w = self.adjustment_values.get(1).copied().unwrap_or(0.5).clamp(0.1, 0.9) * (w / 2.0);
                vec![
                    Point::new(cx - shaft_w / 2.0, y),
                    Point::new(cx + shaft_w / 2.0, y),
                    Point::new(cx + shaft_w / 2.0, y + h - head_h),
                    Point::new(x + w, y + h - head_h),
                    Point::new(cx, y + h),
                    Point::new(x, y + h - head_h),
                    Point::new(cx - shaft_w / 2.0, y + h - head_h),
                ]
            }
            ShapeGeometryType::LeftRightArrow => {
                let head_w = 0.25 * w;
                let shaft_w = 0.3 * h;
                vec![
                    Point::new(x + head_w, y),
                    Point::new(x + head_w, cy - shaft_w / 2.0),
                    Point::new(x + w - head_w, cy - shaft_w / 2.0),
                    Point::new(x + w - head_w, y),
                    Point::new(x + w, cy),
                    Point::new(x + w - head_w, y + h),
                    Point::new(x + w - head_w, cy + shaft_w / 2.0),
                    Point::new(x + head_w, cy + shaft_w / 2.0),
                    Point::new(x + head_w, y + h),
                    Point::new(x, cy),
                ]
            }
            ShapeGeometryType::UpDownArrow => {
                let head_h = 0.25 * h;
                let shaft_w = 0.3 * w;
                vec![
                    Point::new(cx, y),
                    Point::new(x + w, y + head_h),
                    Point::new(cx + shaft_w / 2.0, y + head_h),
                    Point::new(cx + shaft_w / 2.0, y + h - head_h),
                    Point::new(x + w, y + h - head_h),
                    Point::new(cx, y + h),
                    Point::new(x, y + h - head_h),
                    Point::new(cx - shaft_w / 2.0, y + h - head_h),
                    Point::new(cx - shaft_w / 2.0, y + head_h),
                    Point::new(x, y + head_h),
                ]
            }
            ShapeGeometryType::QuadArrow => {
                let a = 0.25 * w.min(h);
                let b_len = 0.12 * w.min(h);
                vec![
                    Point::new(cx, y),
                    Point::new(cx + a, y + a),
                    Point::new(cx + b_len, y + a),
                    Point::new(cx + b_len, cy - b_len),
                    Point::new(x + w - a, cy - b_len),
                    Point::new(x + w - a, cy - a),
                    Point::new(x + w, cy),
                    Point::new(x + w - a, cy + a),
                    Point::new(x + w - a, cy + b_len),
                    Point::new(cx + b_len, cy + b_len),
                    Point::new(cx + b_len, y + h - a),
                    Point::new(cx + a, y + h - a),
                    Point::new(cx, y + h),
                    Point::new(cx - a, y + h - a),
                    Point::new(cx - b_len, y + h - a),
                    Point::new(cx - b_len, cy + b_len),
                    Point::new(x + a, cy + b_len),
                    Point::new(x + a, cy + a),
                    Point::new(x, cy),
                    Point::new(x + a, cy - a),
                    Point::new(x + a, cy - b_len),
                    Point::new(cx - b_len, cy - b_len),
                    Point::new(cx - b_len, y + a),
                    Point::new(cx - a, y + a),
                ]
            }
            ShapeGeometryType::Chevron => {
                let slant = 0.25 * w;
                vec![
                    Point::new(x, y),
                    Point::new(x + w - slant, y),
                    Point::new(x + w, cy),
                    Point::new(x + w - slant, y + h),
                    Point::new(x, y + h),
                    Point::new(x + slant, cy),
                ]
            }
            ShapeGeometryType::NotchedRightArrow => {
                let head_w = 0.4 * w;
                let shaft_w = 0.4 * h;
                let notch = 0.15 * w;
                vec![
                    Point::new(x, cy - shaft_w / 2.0),
                    Point::new(x + w - head_w, cy - shaft_w / 2.0),
                    Point::new(x + w - head_w, y),
                    Point::new(x + w, cy),
                    Point::new(x + w - head_w, y + h),
                    Point::new(x + w - head_w, cy + shaft_w / 2.0),
                    Point::new(x, cy + shaft_w / 2.0),
                    Point::new(x + notch, cy),
                ]
            }
            ShapeGeometryType::StripedRightArrow | ShapeGeometryType::CurvedRightArrow | ShapeGeometryType::CurvedLeftArrow | ShapeGeometryType::UTurnArrow => {
                // Approximate curved/striped arrow with polygon
                vec![
                    Point::new(x, y + h * 0.25),
                    Point::new(x + w * 0.6, y + h * 0.25),
                    Point::new(x + w * 0.6, y),
                    Point::new(x + w, cy),
                    Point::new(x + w * 0.6, y + h),
                    Point::new(x + w * 0.6, y + h * 0.75),
                    Point::new(x, y + h * 0.75),
                ]
            }

            // Callouts
            ShapeGeometryType::RectCallout => {
                let tx = self.adjustment_values.first().copied().unwrap_or(0.2) * w;
                let ty = self.adjustment_values.get(1).copied().unwrap_or(1.3) * h;
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(x + w, y + h),
                    Point::new(x + w * 0.35, y + h),
                    Point::new(x + tx, y + ty),
                    Point::new(x + w * 0.15, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::RoundedRectCallout | ShapeGeometryType::SpeechBubble => {
                let tx = self.adjustment_values.first().copied().unwrap_or(0.2) * w;
                let ty = self.adjustment_values.get(1).copied().unwrap_or(1.3) * h;
                let r = 0.15 * w.min(h);
                vec![
                    Point::new(x + r, y),
                    Point::new(x + w - r, y),
                    Point::new(x + w, y + r),
                    Point::new(x + w, y + h - r),
                    Point::new(x + w - r, y + h),
                    Point::new(x + w * 0.35, y + h),
                    Point::new(x + tx, y + ty),
                    Point::new(x + w * 0.15, y + h),
                    Point::new(x + r, y + h),
                    Point::new(x, y + h - r),
                    Point::new(x, y + r),
                ]
            }
            ShapeGeometryType::CloudCallout => {
                let mut pts = Self::regular_polygon_points(cx, cy, w / 2.0, h / 2.0, 16);
                pts.push(Point::new(x + 0.2 * w, y + 1.25 * h));
                pts
            }

            // Flowcharts
            ShapeGeometryType::FlowchartDocument => {
                let mut pts = vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(x + w, y + h * 0.85),
                ];
                // Wavy bottom
                let steps = 12;
                for i in (0..=steps).rev() {
                    let frac = i as f64 / steps as f64;
                    let wave = (frac * 2.0 * PI).sin() * (h * 0.12);
                    pts.push(Point::new(x + frac * w, y + h * 0.88 + wave));
                }
                pts
            }
            ShapeGeometryType::FlowchartData => {
                let slant = 0.2 * w;
                vec![
                    Point::new(x + slant, y),
                    Point::new(x + w, y),
                    Point::new(x + w - slant, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::FlowchartPredefinedProcess => {
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(x + w, y + h),
                    Point::new(x, y + h),
                ]
            }
            ShapeGeometryType::FlowchartTerminator => {
                let r = h / 2.0;
                let mut pts = Vec::new();
                for i in 0..=12 {
                    let a = -PI / 2.0 + PI * (i as f64 / 12.0);
                    pts.push(Point::new(x + w - r + r * a.cos(), cy + r * a.sin()));
                }
                for i in 0..=12 {
                    let a = PI / 2.0 + PI * (i as f64 / 12.0);
                    pts.push(Point::new(x + r + r * a.cos(), cy + r * a.sin()));
                }
                pts
            }
            ShapeGeometryType::FlowchartPreparation => {
                let cut = 0.2 * w;
                vec![
                    Point::new(x + cut, y),
                    Point::new(x + w - cut, y),
                    Point::new(x + w, cy),
                    Point::new(x + w - cut, y + h),
                    Point::new(x + cut, y + h),
                    Point::new(x, cy),
                ]
            }
            ShapeGeometryType::FlowchartManualOperation => {
                let cut = 0.15 * w;
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(x + w - cut, y + h),
                    Point::new(x + cut, y + h),
                ]
            }
            ShapeGeometryType::FlowchartMerge => {
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(cx, y + h),
                ]
            }
            ShapeGeometryType::FlowchartDelay => {
                let r = h / 2.0;
                let mut pts = vec![
                    Point::new(x, y),
                    Point::new(x + w - r, y),
                ];
                for i in 0..=12 {
                    let a = -PI / 2.0 + PI * (i as f64 / 12.0);
                    pts.push(Point::new(x + w - r + r * a.cos(), cy + r * a.sin()));
                }
                pts.push(Point::new(x, y + h));
                pts
            }
            ShapeGeometryType::Freeform => {
                vec![
                    Point::new(x, y),
                    Point::new(x + w, y),
                    Point::new(x + w, y + h),
                    Point::new(x, y + h),
                ]
            }
        }
    }

    fn regular_polygon_points(cx: f64, cy: f64, rx: f64, ry: f64, sides: usize) -> Vec<Point> {
        let mut pts = Vec::with_capacity(sides);
        let offset = -PI / 2.0;
        for i in 0..sides {
            let a = offset + 2.0 * PI * (i as f64 / sides as f64);
            pts.push(Point::new(cx + rx * a.cos(), cy + ry * a.sin()));
        }
        pts
    }

    fn star_points(cx: f64, cy: f64, rx: f64, ry: f64, points: usize, inner_frac: f64) -> Vec<Point> {
        let total_verts = points * 2;
        let mut pts = Vec::with_capacity(total_verts);
        let offset = -PI / 2.0;
        let irx = rx * inner_frac.clamp(0.1, 0.9);
        let iry = ry * inner_frac.clamp(0.1, 0.9);

        for i in 0..total_verts {
            let a = offset + 2.0 * PI * (i as f64 / total_verts as f64);
            if i % 2 == 0 {
                pts.push(Point::new(cx + rx * a.cos(), cy + ry * a.sin()));
            } else {
                pts.push(Point::new(cx + irx * a.cos(), cy + iry * a.sin()));
            }
        }
        pts
    }

    /// Point-in-polygon ray-casting hit test
    pub fn hit_test(&self, p: &Point) -> bool {
        if !self.bounds.contains_point(p) {
            return false;
        }
        let poly = self.generate_outline_points();
        if poly.is_empty() {
            return false;
        }
        let mut inside = false;
        let mut j = poly.len() - 1;
        for i in 0..poly.len() {
            let pi = &poly[i];
            let pj = &poly[j];
            if ((pi.y > p.y) != (pj.y > p.y))
                && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Render shape vector geometry and text frame to DisplayList
    pub fn render_to_display_list(&self, dl: &mut DisplayList) {
        let fill_color = match &self.fill {
            ShapeFill::Solid(c) => Some(*c),
            ShapeFill::LinearGradient { stops, .. } => stops.first().map(|s| s.1),
            ShapeFill::None => None,
        };
        let stroke_color = self.stroke.as_ref().map(|s| s.color);
        let stroke_width = self.stroke.as_ref().map(|s| s.width_px).unwrap_or(0.0);

        let poly = self.generate_outline_points();
        if !poly.is_empty() {
            dl.push(DrawCommand::DrawPath {
                points: poly,
                fill_color,
                stroke_color,
                stroke_width,
                closed: true,
            });
        }

        // Render Text Frame with Auto-fit
        if let Some(frame) = &self.text_frame {
            let mut cur_y = self.bounds.y + self.bounds.height / 2.0;
            for para in &frame.paragraphs {
                let text_str = para.to_plain_string();
                if text_str.is_empty() {
                    continue;
                }
                let first_run = para.runs.first();
                let font_family = first_run.map(|r| r.font_family.clone()).unwrap_or_else(|| "Inter".to_string());
                let font_size = first_run.map(|r| r.font_size_pt * 1.333).unwrap_or(16.0);
                let color = first_run.map(|r| r.color).unwrap_or(Color::WHITE);
                let bold = first_run.map(|r| r.bold).unwrap_or(false);
                let italic = first_run.map(|r| r.italic).unwrap_or(false);

                // Center origin estimate
                let approx_w = text_str.chars().count() as f64 * font_size * 0.55;
                let origin_x = match para.alignment {
                    TextAlignment::Left => self.bounds.x + frame.margin_pt * 1.333,
                    TextAlignment::Center => (self.bounds.x + self.bounds.width / 2.0) - (approx_w / 2.0),
                    TextAlignment::Right => (self.bounds.x + self.bounds.width) - approx_w - (frame.margin_pt * 1.333),
                    TextAlignment::Justify => self.bounds.x + frame.margin_pt * 1.333,
                };

                dl.push(DrawCommand::DrawTextRun {
                    text: text_str,
                    font_family,
                    font_size_px: font_size,
                    bold,
                    italic,
                    color,
                    origin: Point::new(origin_x.max(self.bounds.x + 4.0), cur_y + font_size * 0.35),
                });
                cur_y += font_size * para.line_spacing;
            }
        }
    }
}
