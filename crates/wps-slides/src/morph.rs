use crate::shape::{ParametricShape, ShapeFill, ShapeGeometryType, ShapeStroke};
use serde::{Deserialize, Serialize};
use wps_core::{Color, Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MorphType {
    #[default]
    ByObject,
    ByWord,
    ByCharacter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideTransition {
    pub name: String, // "Morph", "Fade", "Push", "Wipe", "Split", "Zoom"
    pub duration_ms: u32,
    pub morph_type: MorphType,
}

impl Default for SlideTransition {
    fn default() -> Self {
        Self {
            name: "Morph".to_string(),
            duration_ms: 1500,
            morph_type: MorphType::ByObject,
        }
    }
}

pub struct MorphInterpolator;

impl MorphInterpolator {
    /// Discretizes a polygon into N equidistant points along its perimeter
    pub fn resample_polygon(points: &[Point], n: usize) -> Vec<Point> {
        if points.is_empty() || n == 0 {
            return Vec::new();
        }
        if points.len() == 1 {
            return vec![points[0]; n];
        }

        // Calculate perimeter segment lengths
        let mut lengths = Vec::with_capacity(points.len());
        let mut total_length = 0.0;
        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            let d = p1.distance_to(&p2);
            lengths.push(d);
            total_length += d;
        }

        if total_length < 1e-6 {
            return vec![points[0]; n];
        }

        let step = total_length / n as f64;
        let mut resampled = Vec::with_capacity(n);
        let mut cur_seg = 0;
        let mut seg_progress = 0.0;

        for i in 0..n {
            let target_dist = i as f64 * step;
            while cur_seg < lengths.len() && seg_progress + lengths[cur_seg] < target_dist {
                seg_progress += lengths[cur_seg];
                cur_seg += 1;
            }

            if cur_seg >= points.len() {
                resampled.push(points[0]);
                continue;
            }

            let p1 = points[cur_seg];
            let p2 = points[(cur_seg + 1) % points.len()];
            let seg_len = lengths[cur_seg];

            if seg_len < 1e-6 {
                resampled.push(p1);
            } else {
                let local_t = (target_dist - seg_progress) / seg_len;
                let interp_p = Point::new(
                    p1.x + local_t * (p2.x - p1.x),
                    p1.y + local_t * (p2.y - p1.y),
                );
                resampled.push(interp_p);
            }
        }

        resampled
    }

    /// Interpolates matching shapes across Slide A and Slide B at time progress t (0.0 to 1.0)
    /// Supports the ECMA-376 `!!name` correspondence matching protocol
    pub fn interpolate_shapes(
        shapes_a: &[ParametricShape],
        shapes_b: &[ParametricShape],
        t: f64,
    ) -> Vec<ParametricShape> {
        let t_clamped = t.clamp(0.0, 1.0);
        let mut result = Vec::new();
        let mut matched_b_ids = std::collections::HashSet::new();

        for shape_a in shapes_a {
            // Priority 1: Match by '!!name' (e.g. !!Logo matches !!Logo across different geometries)
            let match_b = if shape_a.name.starts_with("!!") {
                shapes_b.iter().find(|sb| sb.name == shape_a.name)
            } else {
                // Priority 2: Match by exact ID
                // Priority 3: Match by non-empty Name
                // Priority 4: Match by exact Geometry if only one such geometry exists
                shapes_b
                    .iter()
                    .find(|sb| sb.id == shape_a.id)
                    .or_else(|| shapes_b.iter().find(|sb| !sb.name.is_empty() && sb.name == shape_a.name))
                    .or_else(|| {
                        let matching_geom_b: Vec<_> = shapes_b.iter().filter(|sb| sb.geometry == shape_a.geometry).collect();
                        if matching_geom_b.len() == 1 {
                            Some(matching_geom_b[0])
                        } else {
                            None
                        }
                    })
            };

            if let Some(shape_b) = match_b {
                matched_b_ids.insert(&shape_b.id);

                // Interpolate bounds
                let interp_rect = Rect::new(
                    shape_a.bounds.x + (shape_b.bounds.x - shape_a.bounds.x) * t_clamped,
                    shape_a.bounds.y + (shape_b.bounds.y - shape_a.bounds.y) * t_clamped,
                    shape_a.bounds.width + (shape_b.bounds.width - shape_a.bounds.width) * t_clamped,
                    shape_a.bounds.height + (shape_b.bounds.height - shape_a.bounds.height) * t_clamped,
                );

                // Interpolate rotation
                let interp_rot = shape_a.rotation_deg + (shape_b.rotation_deg - shape_a.rotation_deg) * t_clamped;

                // Interpolate fill color
                let interp_fill = match (&shape_a.fill, &shape_b.fill) {
                    (ShapeFill::Solid(c1), ShapeFill::Solid(c2)) => {
                        ShapeFill::Solid(Self::lerp_color(c1, c2, t_clamped))
                    }
                    _ => shape_b.fill.clone(),
                };

                // Interpolate stroke
                let interp_stroke = match (&shape_a.stroke, &shape_b.stroke) {
                    (Some(s1), Some(s2)) => Some(ShapeStroke {
                        color: Self::lerp_color(&s1.color, &s2.color, t_clamped),
                        width_px: s1.width_px + (s2.width_px - s1.width_px) * t_clamped,
                        dash_style: if t_clamped < 0.5 { s1.dash_style.clone() } else { s2.dash_style.clone() },
                    }),
                    (Some(s1), None) => {
                        let mut s = s1.clone();
                        s.width_px *= 1.0 - t_clamped;
                        Some(s)
                    }
                    (None, Some(s2)) => {
                        let mut s = s2.clone();
                        s.width_px *= t_clamped;
                        Some(s)
                    }
                    (None, None) => None,
                };

                // Interpolate geometry path: resample both to 64 points if geometries differ
                let mut morphed = if shape_a.geometry == shape_b.geometry {
                    let mut s = shape_a.clone();
                    // Also interpolate adjustment values
                    let max_adj = shape_a.adjustment_values.len().max(shape_b.adjustment_values.len());
                    let mut interp_adj = Vec::with_capacity(max_adj);
                    for i in 0..max_adj {
                        let v1 = shape_a.adjustment_values.get(i).copied().unwrap_or(0.0);
                        let v2 = shape_b.adjustment_values.get(i).copied().unwrap_or(0.0);
                        interp_adj.push(v1 + (v2 - v1) * t_clamped);
                    }
                    s.adjustment_values = interp_adj;
                    s
                } else {
                    // Smooth path morphing across different geometries!
                    let sample_count = 64;
                    let pts_a = Self::resample_polygon(&shape_a.generate_outline_points(), sample_count);
                    let pts_b = Self::resample_polygon(&shape_b.generate_outline_points(), sample_count);

                    let mut interp_points = Vec::with_capacity(sample_count);
                    for i in 0..sample_count {
                        let pa = pts_a[i];
                        let pb = pts_b[i];
                        interp_points.push(Point::new(
                            pa.x + (pb.x - pa.x) * t_clamped,
                            pa.y + (pb.y - pa.y) * t_clamped,
                        ));
                    }

                    let mut s = ParametricShape::new(&shape_a.id, ShapeGeometryType::Freeform, interp_rect);
                    s.freeform_points = Some(interp_points);
                    s
                };

                morphed.bounds = interp_rect;
                morphed.rotation_deg = interp_rot;
                morphed.fill = interp_fill;
                morphed.stroke = interp_stroke;
                morphed.name = shape_b.name.clone();
                morphed.text_frame = if t_clamped < 0.5 { shape_a.text_frame.clone() } else { shape_b.text_frame.clone() };

                result.push(morphed);
            } else {
                // Shape only in Slide A: fade out and shrink during t in [0.0, 0.5]
                let fade_t = (t_clamped * 2.0).min(1.0);
                let scale = 1.0 - fade_t;
                let mut dying = shape_a.clone();
                let cx = dying.bounds.center().x;
                let cy = dying.bounds.center().y;
                let nw = dying.bounds.width * scale;
                let nh = dying.bounds.height * scale;
                dying.bounds = Rect::new(cx - nw / 2.0, cy - nh / 2.0, nw, nh);

                if let ShapeFill::Solid(mut c) = dying.fill {
                    c.a = ((c.a as f64) * scale).round() as u8;
                    dying.fill = ShapeFill::Solid(c);
                }
                if scale > 0.01 {
                    result.push(dying);
                }
            }
        }

        // Shapes only in Slide B: fade in and scale up during t in [0.5, 1.0]
        for shape_b in shapes_b {
            if !matched_b_ids.contains(&shape_b.id) {
                let fade_t = ((t_clamped - 0.5) * 2.0).clamp(0.0, 1.0);
                let scale = fade_t;
                let mut born = shape_b.clone();
                let cx = born.bounds.center().x;
                let cy = born.bounds.center().y;
                let nw = born.bounds.width * scale;
                let nh = born.bounds.height * scale;
                born.bounds = Rect::new(cx - nw / 2.0, cy - nh / 2.0, nw, nh);

                if let ShapeFill::Solid(mut c) = born.fill {
                    c.a = ((c.a as f64) * scale).round() as u8;
                    born.fill = ShapeFill::Solid(c);
                }
                if scale > 0.01 {
                    result.push(born);
                }
            }
        }

        result
    }

    fn lerp_color(c1: &Color, c2: &Color, t: f64) -> Color {
        let r = (c1.r as f64 + (c2.r as f64 - c1.r as f64) * t).round() as u8;
        let g = (c1.g as f64 + (c2.g as f64 - c1.g as f64) * t).round() as u8;
        let b = (c1.b as f64 + (c2.b as f64 - c1.b as f64) * t).round() as u8;
        let a = (c1.a as f64 + (c2.a as f64 - c1.a as f64) * t).round() as u8;
        Color::rgba(r, g, b, a)
    }
}
