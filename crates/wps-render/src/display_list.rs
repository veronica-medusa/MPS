use serde::{Deserialize, Serialize};
use wps_core::{Color, Point, Rect};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DrawCommand {
    FillRect {
        rect: Rect,
        color: Color,
    },
    StrokeRect {
        rect: Rect,
        color: Color,
        width: f64,
    },
    DrawLine {
        from: Point,
        to: Point,
        color: Color,
        width: f64,
    },
    DrawTextRun {
        text: String,
        font_family: String,
        font_size_px: f64,
        bold: bool,
        italic: bool,
        color: Color,
        origin: Point, // baseline start
    },
    DrawPath {
        points: Vec<Point>,
        fill_color: Option<Color>,
        stroke_color: Option<Color>,
        stroke_width: f64,
        closed: bool,
    },
    DrawImage {
        image_id: String,
        dest_rect: Rect,
        clip: Option<Rect>,
    },
    PushClip {
        rect: Rect,
    },
    PopClip,
}

impl DrawCommand {
    pub fn bounding_box(&self) -> Option<Rect> {
        match self {
            DrawCommand::FillRect { rect, .. } => Some(*rect),
            DrawCommand::StrokeRect { rect, width, .. } => {
                let half = *width / 2.0;
                Some(Rect::new(
                    rect.x - half,
                    rect.y - half,
                    rect.width + width,
                    rect.height + width,
                ))
            }
            DrawCommand::DrawLine { from, to, width, .. } => {
                let min_x = from.x.min(to.x) - width / 2.0;
                let min_y = from.y.min(to.y) - width / 2.0;
                let max_x = from.x.max(to.x) + width / 2.0;
                let max_y = from.y.max(to.y) + width / 2.0;
                Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
            }
            DrawCommand::DrawTextRun { origin, font_size_px, text, .. } => {
                // Approximate width based on average character aspect ratio 0.6
                let approx_width = text.chars().count() as f64 * font_size_px * 0.6;
                Some(Rect::new(
                    origin.x,
                    origin.y - font_size_px,
                    approx_width,
                    *font_size_px * 1.2,
                ))
            }
            DrawCommand::DrawPath { points, stroke_width, .. } => {
                if points.is_empty() {
                    return None;
                }
                let mut min_x = points[0].x;
                let mut max_x = points[0].x;
                let mut min_y = points[0].y;
                let mut max_y = points[0].y;
                for p in points {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }
                let pad = stroke_width / 2.0;
                Some(Rect::new(
                    min_x - pad,
                    min_y - pad,
                    (max_x - min_x) + stroke_width,
                    (max_y - min_y) + stroke_width,
                ))
            }
            DrawCommand::DrawImage { dest_rect, .. } => Some(*dest_rect),
            DrawCommand::PushClip { rect } => Some(*rect),
            DrawCommand::PopClip => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DisplayList {
    pub commands: Vec<DrawCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Frustum Culling: Filters commands to only those intersecting the viewport
    pub fn cull_to_viewport(&self, viewport: &Rect) -> DisplayList {
        let mut culled = DisplayList::new();
        let mut clip_depth = 0;

        for cmd in &self.commands {
            match cmd {
                DrawCommand::PushClip { .. } => {
                    clip_depth += 1;
                    culled.push(cmd.clone());
                }
                DrawCommand::PopClip => {
                    if clip_depth > 0 {
                        clip_depth -= 1;
                        culled.push(cmd.clone());
                    }
                }
                _ => {
                    if let Some(bbox) = cmd.bounding_box() {
                        if bbox.intersects(viewport) {
                            culled.push(cmd.clone());
                        }
                    } else {
                        // Commands without direct bounding boxes (e.g. clips) are retained
                        culled.push(cmd.clone());
                    }
                }
            }
        }

        culled
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_list_culling() {
        let mut dl = DisplayList::new();
        // Visible rect
        dl.push(DrawCommand::FillRect {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::MERCURY_PRIMARY,
        });
        // Offscreen rect
        dl.push(DrawCommand::FillRect {
            rect: Rect::new(1000.0, 1000.0, 50.0, 50.0),
            color: Color::ACCENT_SHEETS,
        });

        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        let culled = dl.cull_to_viewport(&viewport);

        assert_eq!(culled.len(), 1);
    }
}
