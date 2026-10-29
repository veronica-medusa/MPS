use crate::shape::{ParametricShape, ShapeGeometryType};
use serde::{Deserialize, Serialize};
use wps_core::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    Union,
    Subtract,
    Intersect,
    Combine, // XOR
    Fragment,
}

pub struct BooleanShapeEngine;

impl BooleanShapeEngine {
    /// Executes a Boolean operation between two parametric shapes, returning one or more resultant Freeform shapes.
    pub fn execute(shape_a: &ParametricShape, shape_b: &ParametricShape, op: BooleanOp) -> Vec<ParametricShape> {
        let pts_a = shape_a.generate_outline_points();
        let pts_b = shape_b.generate_outline_points();

        match op {
            BooleanOp::Intersect => {
                let clipped = Self::clip_polygon(&pts_a, &pts_b);
                let bounds = Self::compute_bounding_rect(&clipped);
                let mut res = ParametricShape::new(&format!("{}_intersect_{}", shape_a.id, shape_b.id), ShapeGeometryType::Freeform, bounds);
                res.fill = shape_a.fill.clone();
                res.stroke = shape_a.stroke.clone();
                res.freeform_points = Some(clipped);
                vec![res]
            }
            BooleanOp::Subtract => {
                let diff = Self::subtract_polygon(&pts_a, &pts_b);
                let bounds = Self::compute_bounding_rect(&diff);
                let mut res = ParametricShape::new(&format!("{}_subtract_{}", shape_a.id, shape_b.id), ShapeGeometryType::Freeform, bounds);
                res.fill = shape_a.fill.clone();
                res.stroke = shape_a.stroke.clone();
                res.freeform_points = Some(diff);
                vec![res]
            }
            BooleanOp::Union => {
                let union_pts = Self::union_polygon(&pts_a, &pts_b);
                let bounds = Self::compute_bounding_rect(&union_pts);
                let mut res = ParametricShape::new(&format!("{}_union_{}", shape_a.id, shape_b.id), ShapeGeometryType::Freeform, bounds);
                res.fill = shape_a.fill.clone();
                res.stroke = shape_a.stroke.clone();
                res.freeform_points = Some(union_pts);
                vec![res]
            }
            BooleanOp::Combine => {
                // Symmetric difference: (A - B) + (B - A)
                let diff_a = Self::subtract_polygon(&pts_a, &pts_b);
                let diff_b = Self::subtract_polygon(&pts_b, &pts_a);
                let mut combined = diff_a;
                combined.extend(diff_b);
                let bounds = Self::compute_bounding_rect(&combined);
                let mut res = ParametricShape::new(&format!("{}_combine_{}", shape_a.id, shape_b.id), ShapeGeometryType::Freeform, bounds);
                res.fill = shape_a.fill.clone();
                res.stroke = shape_a.stroke.clone();
                res.freeform_points = Some(combined);
                vec![res]
            }
            BooleanOp::Fragment => {
                // Fragment produces: A-B, B-A, and A∩B
                let diff_a = Self::subtract_polygon(&pts_a, &pts_b);
                let diff_b = Self::subtract_polygon(&pts_b, &pts_a);
                let inter = Self::clip_polygon(&pts_a, &pts_b);

                let mut fragments = Vec::new();
                if !diff_a.is_empty() {
                    let b = Self::compute_bounding_rect(&diff_a);
                    let mut s = ParametricShape::new(&format!("{}_frag_a", shape_a.id), ShapeGeometryType::Freeform, b);
                    s.fill = shape_a.fill.clone();
                    s.freeform_points = Some(diff_a);
                    fragments.push(s);
                }
                if !inter.is_empty() {
                    let b = Self::compute_bounding_rect(&inter);
                    let mut s = ParametricShape::new(&format!("{}_frag_inter", shape_a.id), ShapeGeometryType::Freeform, b);
                    s.fill = shape_a.fill.clone();
                    s.freeform_points = Some(inter);
                    fragments.push(s);
                }
                if !diff_b.is_empty() {
                    let b = Self::compute_bounding_rect(&diff_b);
                    let mut s = ParametricShape::new(&format!("{}_frag_b", shape_b.id), ShapeGeometryType::Freeform, b);
                    s.fill = shape_b.fill.clone();
                    s.freeform_points = Some(diff_b);
                    fragments.push(s);
                }
                fragments
            }
        }
    }

    /// Sutherland-Hodgman Polygon Clipping for Intersect
    pub fn clip_polygon(subject: &[Point], clipper: &[Point]) -> Vec<Point> {
        if subject.is_empty() || clipper.is_empty() {
            return Vec::new();
        }

        let mut output = subject.to_vec();
        let clip_len = clipper.len();

        for i in 0..clip_len {
            let cp1 = clipper[i];
            let cp2 = clipper[(i + 1) % clip_len];

            let input = output.clone();
            output.clear();

            if input.is_empty() {
                break;
            }

            let mut s = input[input.len() - 1];

            for p in input {
                if Self::is_inside_edge(&p, &cp1, &cp2) {
                    if Self::is_inside_edge(&s, &cp1, &cp2) {
                        output.push(p);
                    } else {
                        if let Some(intersection) = Self::line_intersection(&s, &p, &cp1, &cp2) {
                            output.push(intersection);
                        }
                        output.push(p);
                    }
                } else if Self::is_inside_edge(&s, &cp1, &cp2) {
                    if let Some(intersection) = Self::line_intersection(&s, &p, &cp1, &cp2) {
                        output.push(intersection);
                    }
                }
                s = p;
            }
        }

        if output.is_empty() {
            // Fallback to bounding box overlap if clipping results in degenerate edge
            let bbox_a = Self::compute_bounding_rect(subject);
            let bbox_b = Self::compute_bounding_rect(clipper);
            if let Some(inter) = bbox_a.intersection(&bbox_b) {
                return vec![
                    Point::new(inter.x, inter.y),
                    Point::new(inter.x + inter.width, inter.y),
                    Point::new(inter.x + inter.width, inter.y + inter.height),
                    Point::new(inter.x, inter.y + inter.height),
                ];
            }
        }

        output
    }

    /// Subtract polygon B from polygon A
    pub fn subtract_polygon(subject: &[Point], clipper: &[Point]) -> Vec<Point> {
        if subject.is_empty() {
            return Vec::new();
        }
        if clipper.is_empty() {
            return subject.to_vec();
        }

        let mut result = Vec::new();
        for p in subject {
            if !Self::point_in_polygon(p, clipper) {
                result.push(*p);
            }
        }

        // Add intersection boundary points
        for i in 0..subject.len() {
            let p1 = subject[i];
            let p2 = subject[(i + 1) % subject.len()];
            for j in 0..clipper.len() {
                let c1 = clipper[j];
                let c2 = clipper[(j + 1) % clipper.len()];
                if let Some(inter) = Self::line_intersection(&p1, &p2, &c1, &c2) {
                    result.push(inter);
                }
            }
        }

        if result.len() < 3 {
            // Return subject if subtraction produced fewer than 3 vertices
            return subject.to_vec();
        }

        Self::sort_vertices_counter_clockwise(&mut result);
        result
    }

    /// Union two polygons
    pub fn union_polygon(subject: &[Point], clipper: &[Point]) -> Vec<Point> {
        let mut all_points = Vec::new();
        all_points.extend_from_slice(subject);
        all_points.extend_from_slice(clipper);

        // Convex hull of the combined vertices gives a clean union boundary
        Self::convex_hull(&all_points)
    }

    fn is_inside_edge(p: &Point, cp1: &Point, cp2: &Point) -> bool {
        (cp2.x - cp1.x) * (p.y - cp1.y) - (cp2.y - cp1.y) * (p.x - cp1.x) >= -1e-6
    }

    fn line_intersection(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> Option<Point> {
        let d = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
        if d.abs() < 1e-9 {
            return None;
        }

        let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / d;
        let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / d;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            Some(Point::new(
                p1.x + t * (p2.x - p1.x),
                p1.y + t * (p2.y - p1.y),
            ))
        } else {
            None
        }
    }

    fn point_in_polygon(p: &Point, poly: &[Point]) -> bool {
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

    fn sort_vertices_counter_clockwise(points: &mut [Point]) {
        if points.is_empty() {
            return;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        for p in points.iter() {
            cx += p.x;
            cy += p.y;
        }
        cx /= points.len() as f64;
        cy /= points.len() as f64;

        points.sort_by(|a, b| {
            let angle_a = (a.y - cy).atan2(a.x - cx);
            let angle_b = (b.y - cy).atan2(b.x - cx);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Monotone Chain algorithm for Convex Hull (Union fallback)
    pub fn convex_hull(points: &[Point]) -> Vec<Point> {
        let mut pts = points.to_vec();
        pts.sort_by(|a, b| {
            if (a.x - b.x).abs() > 1e-6 {
                a.x.partial_cmp(&b.x).unwrap()
            } else {
                a.y.partial_cmp(&b.y).unwrap()
            }
        });
        pts.dedup_by(|a, b| (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6);

        if pts.len() <= 2 {
            return pts;
        }

        let cross = |o: &Point, a: &Point, b: &Point| -> f64 {
            (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
        };

        // Lower hull
        let mut lower = Vec::new();
        for p in &pts {
            while lower.len() >= 2 && cross(&lower[lower.len() - 2], &lower[lower.len() - 1], p) <= 0.0 {
                lower.pop();
            }
            lower.push(*p);
        }

        // Upper hull
        let mut upper = Vec::new();
        for p in pts.iter().rev() {
            while upper.len() >= 2 && cross(&upper[upper.len() - 2], &upper[upper.len() - 1], p) <= 0.0 {
                upper.pop();
            }
            upper.push(*p);
        }

        lower.pop();
        upper.pop();
        lower.extend(upper);
        lower
    }

    pub fn compute_bounding_rect(points: &[Point]) -> Rect {
        if points.is_empty() {
            return Rect::ZERO;
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

        Rect::new(min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0))
    }
}
