use serde::{Deserialize, Serialize};
use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use wps_render::{DisplayList, DrawCommand};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationKind {
    Highlight {
        color: Color,
        opacity: f32,
    },
    Underline {
        color: Color,
    },
    Squiggly {
        color: Color,
    },
    Strikethrough {
        color: Color,
    },
    StickyNote {
        author: String,
        content: String,
        color: Color,
    },
    FreehandInk {
        points: Vec<Point>,
        stroke_color: Color,
        stroke_width: f64,
    },
    TextBox {
        text: String,
        font_size: f64,
        text_color: Color,
        background: Color,
        border_color: Color,
    },
    Callout {
        text: String,
        anchor_point: Point,
        box_rect: Rect,
        color: Color,
    },
    Shape {
        shape_type: ShapeType,
        stroke_color: Color,
        fill_color: Option<Color>,
        stroke_width: f64,
    },
    Stamp {
        stamp_type: StampType,
        author: String,
        date_str: String,
    },
    Measurement {
        start: Point,
        end: Point,
        scale_ratio: f64, // e.g. 1 pt = 0.352778 mm
        unit: String,     // "mm", "cm", "in"
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StampType {
    Approved,
    Confidential,
    Draft,
    Rejected,
    Final,
    Received,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub page_index: usize,
    pub bounds: Rect,
    pub kind: AnnotationKind,
    pub created_at_sec: u64,
}

impl Annotation {
    pub fn highlight(id: &str, page_index: usize, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            page_index,
            bounds,
            kind: AnnotationKind::Highlight {
                color: Color::rgb(255, 235, 59), // Yellow highlighter
                opacity: 0.45,
            },
            created_at_sec: 0,
        }
    }

    pub fn sticky_note(id: &str, page_index: usize, pos: Point, author: &str, content: &str) -> Self {
        Self {
            id: id.to_string(),
            page_index,
            bounds: Rect::new(pos.x, pos.y, 24.0, 24.0),
            kind: AnnotationKind::StickyNote {
                author: author.to_string(),
                content: content.to_string(),
                color: Color::rgb(255, 213, 79),
            },
            created_at_sec: 0,
        }
    }

    pub fn stamp(id: &str, page_index: usize, bounds: Rect, stamp_type: StampType, author: &str, date_str: &str) -> Self {
        Self {
            id: id.to_string(),
            page_index,
            bounds,
            kind: AnnotationKind::Stamp {
                stamp_type,
                author: author.to_string(),
                date_str: date_str.to_string(),
            },
            created_at_sec: 0,
        }
    }

    pub fn measurement(id: &str, page_index: usize, start: Point, end: Point, scale_ratio: f64, unit: &str) -> Self {
        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);

        Self {
            id: id.to_string(),
            page_index,
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(10.0), (max_y - min_y).max(10.0)),
            kind: AnnotationKind::Measurement {
                start,
                end,
                scale_ratio,
                unit: unit.to_string(),
            },
            created_at_sec: 0,
        }
    }

    pub fn hit_test(&self, p: Point) -> bool {
        p.x >= self.bounds.x && p.x <= self.bounds.x + self.bounds.width &&
        p.y >= self.bounds.y && p.y <= self.bounds.y + self.bounds.height
    }

    pub fn render_to_display_list(&self, dl: &mut DisplayList) {
        match &self.kind {
            AnnotationKind::Highlight { color, opacity } => {
                let mut highlight_color = *color;
                highlight_color.a = (*opacity * 255.0) as u8;
                dl.push(DrawCommand::FillRect {
                    rect: self.bounds,
                    color: highlight_color,
                });
            }
            AnnotationKind::Underline { color } => {
                let baseline_y = self.bounds.y + self.bounds.height;
                dl.push(DrawCommand::DrawLine {
                    from: Point::new(self.bounds.x, baseline_y),
                    to: Point::new(self.bounds.x + self.bounds.width, baseline_y),
                    color: *color,
                    width: 1.5,
                });
            }
            AnnotationKind::Squiggly { color } => {
                let baseline_y = self.bounds.y + self.bounds.height;
                let step = 4.0;
                let mut cur_x = self.bounds.x;
                let mut up = true;
                while cur_x < self.bounds.x + self.bounds.width {
                    let next_x = (cur_x + step).min(self.bounds.x + self.bounds.width);
                    let target_y = if up { baseline_y - 2.0 } else { baseline_y + 2.0 };
                    dl.push(DrawCommand::DrawLine {
                        from: Point::new(cur_x, baseline_y),
                        to: Point::new(next_x, target_y),
                        color: *color,
                        width: 1.5,
                    });
                    cur_x = next_x;
                    up = !up;
                }
            }
            AnnotationKind::Strikethrough { color } => {
                let mid_y = self.bounds.y + self.bounds.height * 0.5;
                dl.push(DrawCommand::DrawLine {
                    from: Point::new(self.bounds.x, mid_y),
                    to: Point::new(self.bounds.x + self.bounds.width, mid_y),
                    color: *color,
                    width: 1.5,
                });
            }
            AnnotationKind::StickyNote { color, .. } => {
                dl.push(DrawCommand::FillRect {
                    rect: self.bounds,
                    color: *color,
                });
                dl.push(DrawCommand::StrokeRect {
                    rect: self.bounds,
                    color: Color::rgb(200, 160, 40),
                    width: 1.0,
                });
            }
            AnnotationKind::FreehandInk { points, stroke_color, stroke_width } => {
                for i in 1..points.len() {
                    dl.push(DrawCommand::DrawLine {
                        from: points[i - 1],
                        to: points[i],
                        color: *stroke_color,
                        width: *stroke_width,
                    });
                }
            }
            AnnotationKind::TextBox { text, font_size, text_color, background, border_color } => {
                dl.push(DrawCommand::FillRect {
                    rect: self.bounds,
                    color: *background,
                });
                dl.push(DrawCommand::StrokeRect {
                    rect: self.bounds,
                    color: *border_color,
                    width: 1.0,
                });
                dl.push(DrawCommand::DrawTextRun {
                    text: text.clone(),
                    font_family: "Inter".to_string(),
                    font_size_px: *font_size,
                    bold: false,
                    italic: false,
                    color: *text_color,
                    origin: Point::new(self.bounds.x + 4.0, self.bounds.y + *font_size),
                });
            }
            AnnotationKind::Callout { text, anchor_point, box_rect, color } => {
                // Leader line
                dl.push(DrawCommand::DrawLine {
                    from: *anchor_point,
                    to: Point::new(box_rect.x, box_rect.y + box_rect.height * 0.5),
                    color: *color,
                    width: 1.5,
                });
                // Box
                dl.push(DrawCommand::FillRect {
                    rect: *box_rect,
                    color: Color::WHITE,
                });
                dl.push(DrawCommand::StrokeRect {
                    rect: *box_rect,
                    color: *color,
                    width: 1.5,
                });
                dl.push(DrawCommand::DrawTextRun {
                    text: text.clone(),
                    font_family: "Inter".to_string(),
                    font_size_px: 11.0,
                    bold: false,
                    italic: false,
                    color: *color,
                    origin: Point::new(box_rect.x + 6.0, box_rect.y + 14.0),
                });
            }
            AnnotationKind::Shape { shape_type, stroke_color, fill_color, stroke_width } => {
                match shape_type {
                    ShapeType::Rectangle => {
                        if let Some(fill) = fill_color {
                            dl.push(DrawCommand::FillRect {
                                rect: self.bounds,
                                color: *fill,
                            });
                        }
                        dl.push(DrawCommand::StrokeRect {
                            rect: self.bounds,
                            color: *stroke_color,
                            width: *stroke_width,
                        });
                    }
                    ShapeType::Ellipse => {
                        if let Some(fill) = fill_color {
                            dl.push(DrawCommand::FillRect {
                                rect: self.bounds,
                                color: *fill,
                            });
                        }
                        dl.push(DrawCommand::StrokeRect {
                            rect: self.bounds,
                            color: *stroke_color,
                            width: *stroke_width,
                        });
                    }
                    ShapeType::Line | ShapeType::Arrow => {
                        dl.push(DrawCommand::DrawLine {
                            from: Point::new(self.bounds.x, self.bounds.y),
                            to: Point::new(self.bounds.x + self.bounds.width, self.bounds.y + self.bounds.height),
                            color: *stroke_color,
                            width: *stroke_width,
                        });
                    }
                }
            }
            AnnotationKind::Stamp { stamp_type, author, date_str } => {
                let stamp_text = match stamp_type {
                    StampType::Approved => "APPROVED",
                    StampType::Confidential => "CONFIDENTIAL",
                    StampType::Draft => "DRAFT",
                    StampType::Rejected => "REJECTED",
                    StampType::Final => "FINAL",
                    StampType::Received => "RECEIVED",
                };
                let border_color = match stamp_type {
                    StampType::Approved | StampType::Final => Color::rgb(46, 125, 50),
                    StampType::Confidential | StampType::Rejected => Color::rgb(198, 40, 40),
                    StampType::Draft | StampType::Received => Color::rgb(21, 101, 192),
                };

                dl.push(DrawCommand::StrokeRect {
                    rect: self.bounds,
                    color: border_color,
                    width: 3.0,
                });
                dl.push(DrawCommand::DrawTextRun {
                    text: stamp_text.to_string(),
                    font_family: "Inter".to_string(),
                    font_size_px: 16.0,
                    bold: true,
                    italic: false,
                    color: border_color,
                    origin: Point::new(self.bounds.x + 12.0, self.bounds.y + 22.0),
                });
                dl.push(DrawCommand::DrawTextRun {
                    text: format!("{} • {}", author, date_str),
                    font_family: "Inter".to_string(),
                    font_size_px: 9.0,
                    bold: false,
                    italic: false,
                    color: border_color,
                    origin: Point::new(self.bounds.x + 12.0, self.bounds.y + 36.0),
                });
            }
            AnnotationKind::Measurement { start, end, scale_ratio, unit } => {
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let pt_distance = (dx * dx + dy * dy).sqrt();
                let scaled_distance = pt_distance * scale_ratio;

                let measurement_color = Color::rgb(0, 122, 255);
                dl.push(DrawCommand::DrawLine {
                    from: *start,
                    to: *end,
                    color: measurement_color,
                    width: 2.0,
                });

                let mid_x = (start.x + end.x) * 0.5;
                let mid_y = (start.y + end.y) * 0.5 - 6.0;
                dl.push(DrawCommand::DrawTextRun {
                    text: format!("{:.2} {}", scaled_distance, unit),
                    font_family: "Inter".to_string(),
                    font_size_px: 11.0,
                    bold: false,
                    italic: false,
                    color: measurement_color,
                    origin: Point::new(mid_x, mid_y),
                });
            }
        }
    }
}
