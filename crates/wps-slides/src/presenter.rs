use serde::{Deserialize, Serialize};
use wps_core::{Color, Point, Rect};
use wps_render::{DisplayList, DrawCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DisplayMode {
    #[default]
    Normal,
    Blackout, // 'B' key
    Whiteout, // 'W' key
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaserPointer {
    pub active: bool,
    pub position: Option<Point>,
    pub color: Color,
    pub radius_px: f64,
}

impl Default for LaserPointer {
    fn default() -> Self {
        Self {
            active: false,
            position: None,
            color: Color::rgb(255, 0, 51),
            radius_px: 5.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InkStroke {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f64,
    pub is_highlighter: bool,
}

impl InkStroke {
    pub fn new_pen(color: Color, width: f64) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
            is_highlighter: false,
        }
    }

    pub fn new_highlighter(mut color: Color, width: f64) -> Self {
        color.a = 90; // Translucent alpha (approx 0.35)
        Self {
            points: Vec::new(),
            color,
            width,
            is_highlighter: true,
        }
    }

    pub fn push_point(&mut self, pt: Point) {
        self.points.push(pt);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideshowSession {
    pub current_slide_idx: usize,
    pub total_slides: usize,
    pub current_build_step: usize,
    pub total_steps_in_slide: usize,
    pub elapsed_seconds: u32,
    pub is_timer_running: bool,
    pub display_mode: DisplayMode,
    pub laser_pointer: LaserPointer,
    pub ink_strokes: Vec<InkStroke>,
    pub active_stroke: Option<InkStroke>,
    pub speaker_notes: String,
}

impl SlideshowSession {
    pub fn new(total_slides: usize) -> Self {
        Self {
            current_slide_idx: 0,
            total_slides: total_slides.max(1),
            current_build_step: 0,
            total_steps_in_slide: 0,
            elapsed_seconds: 0,
            is_timer_running: true,
            display_mode: DisplayMode::Normal,
            laser_pointer: LaserPointer::default(),
            ink_strokes: Vec::new(),
            active_stroke: None,
            speaker_notes: String::new(),
        }
    }

    pub fn next_step(&mut self) -> bool {
        if self.current_build_step < self.total_steps_in_slide {
            self.current_build_step += 1;
            true
        } else if self.current_slide_idx + 1 < self.total_slides {
            self.current_slide_idx += 1;
            self.current_build_step = 0;
            self.clear_ink();
            true
        } else {
            false // End of slideshow
        }
    }

    pub fn prev_step(&mut self) -> bool {
        if self.current_build_step > 0 {
            self.current_build_step -= 1;
            true
        } else if self.current_slide_idx > 0 {
            self.current_slide_idx -= 1;
            self.current_build_step = 0;
            self.clear_ink();
            true
        } else {
            false
        }
    }

    pub fn goto_slide(&mut self, slide_idx: usize) {
        if slide_idx < self.total_slides {
            self.current_slide_idx = slide_idx;
            self.current_build_step = 0;
            self.clear_ink();
        }
    }

    pub fn toggle_blackout(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Blackout => DisplayMode::Normal,
            _ => DisplayMode::Blackout,
        };
    }

    pub fn toggle_whiteout(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Whiteout => DisplayMode::Normal,
            _ => DisplayMode::Whiteout,
        };
    }

    pub fn tick_timer(&mut self) {
        if self.is_timer_running {
            self.elapsed_seconds += 1;
        }
    }

    pub fn toggle_timer(&mut self) {
        self.is_timer_running = !self.is_timer_running;
    }

    pub fn reset_timer(&mut self) {
        self.elapsed_seconds = 0;
        self.is_timer_running = true;
    }

    pub fn set_laser_position(&mut self, pt: Option<Point>) {
        self.laser_pointer.position = pt;
        self.laser_pointer.active = pt.is_some();
    }

    pub fn start_pen_stroke(&mut self, color: Color, width: f64) {
        self.active_stroke = Some(InkStroke::new_pen(color, width));
    }

    pub fn start_highlighter_stroke(&mut self, color: Color, width: f64) {
        self.active_stroke = Some(InkStroke::new_highlighter(color, width));
    }

    pub fn add_ink_point(&mut self, pt: Point) {
        if let Some(stroke) = &mut self.active_stroke {
            stroke.push_point(pt);
        }
    }

    pub fn finish_ink_stroke(&mut self) {
        if let Some(stroke) = self.active_stroke.take() {
            if stroke.points.len() >= 2 {
                self.ink_strokes.push(stroke);
            }
        }
    }

    pub fn clear_ink(&mut self) {
        self.ink_strokes.clear();
        self.active_stroke = None;
    }

    pub fn render_overlay_to_display_list(&self, viewport: &Rect, dl: &mut DisplayList) {
        // 1. Blackout or Whiteout screen cover
        match self.display_mode {
            DisplayMode::Blackout => {
                dl.push(DrawCommand::FillRect {
                    rect: *viewport,
                    color: Color::BLACK,
                });
                return;
            }
            DisplayMode::Whiteout => {
                dl.push(DrawCommand::FillRect {
                    rect: *viewport,
                    color: Color::WHITE,
                });
                return;
            }
            DisplayMode::Normal => {}
        }

        // 2. Render committed ink strokes
        for stroke in &self.ink_strokes {
            Self::render_stroke(stroke, dl);
        }

        // 3. Render currently active ink stroke
        if let Some(stroke) = &self.active_stroke {
            Self::render_stroke(stroke, dl);
        }

        // 4. Render Laser Pointer with glowing aura
        if self.laser_pointer.active {
            if let Some(pos) = self.laser_pointer.position {
                // Outer glow
                dl.push(DrawCommand::FillRect {
                    rect: Rect::new(pos.x - 10.0, pos.y - 10.0, 20.0, 20.0),
                    color: Color::rgba(255, 0, 51, 60),
                });
                // Core dot
                dl.push(DrawCommand::FillRect {
                    rect: Rect::new(pos.x - 3.5, pos.y - 3.5, 7.0, 7.0),
                    color: self.laser_pointer.color,
                });
            }
        }
    }

    fn render_stroke(stroke: &InkStroke, dl: &mut DisplayList) {
        if stroke.points.len() < 2 {
            return;
        }
        for i in 0..stroke.points.len() - 1 {
            dl.push(DrawCommand::DrawLine {
                from: stroke.points[i],
                to: stroke.points[i + 1],
                color: stroke.color,
                width: stroke.width,
            });
        }
    }
}
