use crate::scene_graph::Slide;
use crate::shape::ParametricShape;
use serde::{Deserialize, Serialize};
use wps_core::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationEffect {
    Fade,
    FlyIn,
    Zoom,
    Wipe,
    Spin,
    Morph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    OnClick,
    WithPrevious,
    AfterPrevious,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeAnimation {
    pub shape_id: String,
    pub effect: AnimationEffect,
    pub trigger: TriggerType,
    pub duration_ms: u32,
    pub delay_ms: u32,
}

pub struct MorphInterpolator;

impl MorphInterpolator {
    /// Interpolates matching shapes across Slide A and Slide B at time progress t (0.0 to 1.0)
    pub fn interpolate_slides(slide_a: &Slide, slide_b: &Slide, t: f64) -> Vec<ParametricShape> {
        let t_clamped = t.clamp(0.0, 1.0);
        let mut result = Vec::new();

        for shape_a in &slide_a.shapes {
            // Find corresponding shape in slide_b by name or ID
            if let Some(shape_b) = slide_b.shapes.iter().find(|s| s.name == shape_a.name || s.id == shape_a.id) {
                // Interpolate bounds
                let interp_rect = Rect::new(
                    shape_a.bounds.x + (shape_b.bounds.x - shape_a.bounds.x) * t_clamped,
                    shape_a.bounds.y + (shape_b.bounds.y - shape_a.bounds.y) * t_clamped,
                    shape_a.bounds.width + (shape_b.bounds.width - shape_a.bounds.width) * t_clamped,
                    shape_a.bounds.height + (shape_b.bounds.height - shape_a.bounds.height) * t_clamped,
                );

                let interp_rot = shape_a.rotation_deg + (shape_b.rotation_deg - shape_a.rotation_deg) * t_clamped;

                let mut morphed = shape_a.clone();
                morphed.bounds = interp_rect;
                morphed.rotation_deg = interp_rot;
                result.push(morphed);
            } else {
                // Shape only in slide_a: fade out (scale down)
                let mut dying = shape_a.clone();
                dying.bounds = Rect::new(
                    dying.bounds.x,
                    dying.bounds.y,
                    dying.bounds.width * (1.0 - t_clamped),
                    dying.bounds.height * (1.0 - t_clamped),
                );
                result.push(dying);
            }
        }

        // Shapes only in slide_b: fade in
        for shape_b in &slide_b.shapes {
            if !slide_a.shapes.iter().any(|s| s.name == shape_b.name || s.id == shape_b.id) {
                let mut born = shape_b.clone();
                born.bounds = Rect::new(
                    born.bounds.x,
                    born.bounds.y,
                    born.bounds.width * t_clamped,
                    born.bounds.height * t_clamped,
                );
                result.push(born);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::ShapeGeometryType;

    #[test]
    fn test_morph_transition_interpolation() {
        let mut slide1 = Slide::new("s1", "Title 1", "layout-1");
        let mut s1 = ParametricShape::new("circle", ShapeGeometryType::Ellipse, Rect::new(50.0, 50.0, 100.0, 100.0));
        s1.name = "MyCircle".to_string();
        slide1.shapes.push(s1);

        let mut slide2 = Slide::new("s2", "Title 2", "layout-1");
        let mut s2 = ParametricShape::new("circle", ShapeGeometryType::Ellipse, Rect::new(250.0, 150.0, 200.0, 200.0));
        s2.name = "MyCircle".to_string();
        slide2.shapes.push(s2);

        // At midpoint t = 0.5
        let mid_shapes = MorphInterpolator::interpolate_slides(&slide1, &slide2, 0.5);
        assert_eq!(mid_shapes.len(), 1);
        assert_eq!(mid_shapes[0].bounds, Rect::new(150.0, 100.0, 150.0, 150.0));
    }
}
