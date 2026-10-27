use serde::{Deserialize, Serialize};
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
    Heart,
    LightningBolt,
    Sun,
    Moon,
    Cloud,

    // Stars
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
    UturnArrow,

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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeFill {
    None,
    Solid(Color),
    LinearGradient { angle_deg: f64, stops: Vec<(f64, Color)> },
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParametricShape {
    pub id: String,
    pub name: String,
    pub geometry: ShapeGeometryType,
    pub bounds: Rect,
    pub rotation_deg: f64,
    pub fill: ShapeFill,
    pub stroke: Option<ShapeStroke>,
    pub adjustment_values: Vec<f64>, // e.g. corner radius, star inner radius
    pub text: Option<String>,
}

impl ParametricShape {
    pub fn new(id: &str, geometry: ShapeGeometryType, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            name: format!("{:?}", geometry),
            geometry,
            bounds,
            rotation_deg: 0.0,
            fill: ShapeFill::Solid(Color::MERCURY_PRIMARY),
            stroke: Some(ShapeStroke::default()),
            adjustment_values: vec![0.2],
            text: None,
        }
    }

    /// Generates vector draw commands for this shape
    pub fn render_to_display_list(&self, dl: &mut DisplayList) {
        let fill_color = match &self.fill {
            ShapeFill::Solid(c) => Some(*c),
            ShapeFill::LinearGradient { stops, .. } => stops.first().map(|s| s.1),
            ShapeFill::None => None,
        };
        let stroke_color = self.stroke.as_ref().map(|s| s.color);
        let stroke_width = self.stroke.as_ref().map(|s| s.width_px).unwrap_or(0.0);

        match self.geometry {
            ShapeGeometryType::Rectangle => {
                if let Some(c) = fill_color {
                    dl.push(DrawCommand::FillRect { rect: self.bounds, color: c });
                }
                if let Some(c) = stroke_color {
                    dl.push(DrawCommand::StrokeRect { rect: self.bounds, color: c, width: stroke_width });
                }
            }
            ShapeGeometryType::Triangle => {
                let p1 = Point::new(self.bounds.x + self.bounds.width / 2.0, self.bounds.y);
                let p2 = Point::new(self.bounds.x + self.bounds.width, self.bounds.y + self.bounds.height);
                let p3 = Point::new(self.bounds.x, self.bounds.y + self.bounds.height);
                dl.push(DrawCommand::DrawPath {
                    points: vec![p1, p2, p3],
                    fill_color,
                    stroke_color,
                    stroke_width,
                    closed: true,
                });
            }
            ShapeGeometryType::Diamond => {
                let top = Point::new(self.bounds.x + self.bounds.width / 2.0, self.bounds.y);
                let right = Point::new(self.bounds.x + self.bounds.width, self.bounds.y + self.bounds.height / 2.0);
                let bottom = Point::new(self.bounds.x + self.bounds.width / 2.0, self.bounds.y + self.bounds.height);
                let left = Point::new(self.bounds.x, self.bounds.y + self.bounds.height / 2.0);
                dl.push(DrawCommand::DrawPath {
                    points: vec![top, right, bottom, left],
                    fill_color,
                    stroke_color,
                    stroke_width,
                    closed: true,
                });
            }
            _ => {
                // Generic parametric polygon
                if let Some(c) = fill_color {
                    dl.push(DrawCommand::FillRect { rect: self.bounds, color: c });
                }
                if let Some(c) = stroke_color {
                    dl.push(DrawCommand::StrokeRect { rect: self.bounds, color: c, width: stroke_width });
                }
            }
        }

        // Render text label if present
        if let Some(t) = &self.text {
            dl.push(DrawCommand::DrawTextRun {
                text: t.clone(),
                font_family: "Inter".into(),
                font_size_px: 14.0,
                bold: true,
                italic: false,
                color: Color::WHITE,
                origin: Point::new(self.bounds.x + 10.0, self.bounds.y + self.bounds.height / 2.0 + 5.0),
            });
        }
    }
}
