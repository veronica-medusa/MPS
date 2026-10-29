use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use wps_core::{Color, Point, Rect};
use wps_render::{DisplayList, DrawCommand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionPort {
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

impl ConnectionPort {
    /// Returns normalized (u, v) in range [0, 1] and outward unit normal (nx, ny)
    pub fn normalized_info(&self) -> (f64, f64, f64, f64) {
        match self {
            ConnectionPort::Top => (0.5, 0.0, 0.0, -1.0),
            ConnectionPort::Right => (1.0, 0.5, 1.0, 0.0),
            ConnectionPort::Bottom => (0.5, 1.0, 0.0, 1.0),
            ConnectionPort::Left => (0.0, 0.5, -1.0, 0.0),
            ConnectionPort::Center => (0.5, 0.5, 0.0, 0.0),
        }
    }

    pub fn port_coordinate(&self, bounds: &Rect) -> Point {
        let (u, v, _, _) = self.normalized_info();
        Point::new(bounds.x + u * bounds.width, bounds.y + v * bounds.height)
    }

    pub fn outward_normal(&self) -> (f64, f64) {
        let (_, _, nx, ny) = self.normalized_info();
        (nx, ny)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectorRouting {
    Straight,
    #[default]
    Orthogonal,
    Curved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArrowheadType {
    #[default]
    None,
    Triangle,
    Stealth,
    Oval,
    Diamond,
    Open,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartConnector {
    pub id: String,
    pub start_shape_id: String,
    pub start_port: ConnectionPort,
    pub end_shape_id: String,
    pub end_port: ConnectionPort,
    pub routing: ConnectorRouting,
    pub start_arrow: ArrowheadType,
    pub end_arrow: ArrowheadType,
    pub color: Color,
    pub stroke_width: f64,
    pub lane_offset: f64, // Draggable midpoint adjustment for orthogonal paths
}

impl SmartConnector {
    pub fn new(id: &str, start_id: &str, end_id: &str) -> Self {
        Self {
            id: id.to_string(),
            start_shape_id: start_id.to_string(),
            start_port: ConnectionPort::Right,
            end_shape_id: end_id.to_string(),
            end_port: ConnectionPort::Left,
            routing: ConnectorRouting::Orthogonal,
            start_arrow: ArrowheadType::None,
            end_arrow: ArrowheadType::Triangle,
            color: Color::MERCURY_INK,
            stroke_width: 2.0,
            lane_offset: 0.0,
        }
    }

    pub fn port_coordinate(bounds: &Rect, port: ConnectionPort) -> Point {
        port.port_coordinate(bounds)
    }

    /// Computes collision-free waypoints for the connector connecting start_bounds to end_bounds
    pub fn compute_waypoints(&self, start_bounds: &Rect, end_bounds: &Rect) -> Vec<Point> {
        self.compute_waypoints_with_obstacles(start_bounds, end_bounds, &[])
    }

    /// A* Orthogonal router with obstacle avoidance and corner bend minimizer
    pub fn compute_waypoints_with_obstacles(
        &self,
        start_bounds: &Rect,
        end_bounds: &Rect,
        other_obstacles: &[Rect],
    ) -> Vec<Point> {
        let p_start = self.start_port.port_coordinate(start_bounds);
        let p_end = self.end_port.port_coordinate(end_bounds);

        match self.routing {
            ConnectorRouting::Straight => vec![p_start, p_end],
            ConnectorRouting::Curved => {
                // Cubic Bezier spline discretized into smooth waypoints
                let (snx, sny) = self.start_port.outward_normal();
                let (enx, eny) = self.end_port.outward_normal();
                let dist = p_start.distance_to(&p_end).max(40.0) * 0.45;

                let cp1 = Point::new(p_start.x + snx * dist, p_start.y + sny * dist);
                let cp2 = Point::new(p_end.x + enx * dist, p_end.y + eny * dist);

                let steps = 24;
                let mut pts = Vec::with_capacity(steps + 1);
                for i in 0..=steps {
                    let t = i as f64 / steps as f64;
                    let u = 1.0 - t;
                    let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                    let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                    pts.push(Point::new(x, y));
                }
                pts
            }
            ConnectorRouting::Orthogonal => {
                // A* Orthogonal Grid Pathfinding
                let router = AStarOrthogonalRouter::new(start_bounds, end_bounds, other_obstacles, self.lane_offset);
                router.route(p_start, p_end, self.start_port, self.end_port)
            }
        }
    }

    pub fn render_to_display_list(
        &self,
        start_bounds: &Rect,
        end_bounds: &Rect,
        dl: &mut DisplayList,
    ) {
        let waypoints = self.compute_waypoints(start_bounds, end_bounds);
        if waypoints.len() < 2 {
            return;
        }

        // Draw path lines
        for i in 0..waypoints.len() - 1 {
            dl.push(DrawCommand::DrawLine {
                from: waypoints[i],
                to: waypoints[i + 1],
                color: self.color,
                width: self.stroke_width,
            });
        }

        // Render End Arrowhead if requested
        if self.end_arrow != ArrowheadType::None && waypoints.len() >= 2 {
            let tip = waypoints[waypoints.len() - 1];
            let prev = waypoints[waypoints.len() - 2];
            let angle = (tip.y - prev.y).atan2(tip.x - prev.x);
            Self::render_arrowhead(tip, angle, self.end_arrow, self.color, self.stroke_width, dl);
        }

        // Render Start Arrowhead if requested
        if self.start_arrow != ArrowheadType::None && waypoints.len() >= 2 {
            let tip = waypoints[0];
            let next = waypoints[1];
            let angle = (tip.y - next.y).atan2(tip.x - next.x);
            Self::render_arrowhead(tip, angle, self.start_arrow, self.color, self.stroke_width, dl);
        }
    }

    fn render_arrowhead(
        tip: Point,
        angle: f64,
        arrow_type: ArrowheadType,
        color: Color,
        width: f64,
        dl: &mut DisplayList,
    ) {
        let size = (width * 3.5).max(8.0);
        let left_angle = angle + std::f64::consts::PI * 0.85;
        let right_angle = angle - std::f64::consts::PI * 0.85;

        match arrow_type {
            ArrowheadType::Triangle | ArrowheadType::Stealth => {
                let p1 = Point::new(tip.x + size * left_angle.cos(), tip.y + size * left_angle.sin());
                let p2 = Point::new(tip.x + size * right_angle.cos(), tip.y + size * right_angle.sin());
                dl.push(DrawCommand::DrawPath {
                    points: vec![tip, p1, p2],
                    fill_color: Some(color),
                    stroke_color: Some(color),
                    stroke_width: 1.0,
                    closed: true,
                });
            }
            ArrowheadType::Open => {
                let p1 = Point::new(tip.x + size * left_angle.cos(), tip.y + size * left_angle.sin());
                let p2 = Point::new(tip.x + size * right_angle.cos(), tip.y + size * right_angle.sin());
                dl.push(DrawCommand::DrawLine { from: tip, to: p1, color, width });
                dl.push(DrawCommand::DrawLine { from: tip, to: p2, color, width });
            }
            ArrowheadType::Diamond => {
                let back = Point::new(tip.x - size * 1.5 * angle.cos(), tip.y - size * 1.5 * angle.sin());
                let mid = Point::new(tip.x - size * 0.75 * angle.cos(), tip.y - size * 0.75 * angle.sin());
                let p1 = Point::new(mid.x + (size * 0.5) * (angle + std::f64::consts::PI / 2.0).cos(), mid.y + (size * 0.5) * (angle + std::f64::consts::PI / 2.0).sin());
                let p2 = Point::new(mid.x + (size * 0.5) * (angle - std::f64::consts::PI / 2.0).cos(), mid.y + (size * 0.5) * (angle - std::f64::consts::PI / 2.0).sin());
                dl.push(DrawCommand::DrawPath {
                    points: vec![tip, p1, back, p2],
                    fill_color: Some(color),
                    stroke_color: Some(color),
                    stroke_width: 1.0,
                    closed: true,
                });
            }
            ArrowheadType::Oval => {
                let center = Point::new(tip.x - (size / 2.0) * angle.cos(), tip.y - (size / 2.0) * angle.sin());
                let r = size * 0.45;
                dl.push(DrawCommand::FillRect {
                    rect: Rect::new(center.x - r, center.y - r, r * 2.0, r * 2.0),
                    color,
                });
            }
            ArrowheadType::None => {}
        }
    }
}

// ----------------------------------------------------------------------------
// A* Orthogonal Collision-Free Router
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct GridNode {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq)]
struct AStarState {
    cost: f64,
    node: GridNode,
    dir: (i32, i32), // direction of arrival
}

impl Eq for AStarState {}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invert for min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct AStarOrthogonalRouter {
    obstacles: Vec<Rect>,
    lane_offset: f64,
}

impl AStarOrthogonalRouter {
    pub fn new(start_bounds: &Rect, end_bounds: &Rect, other_obstacles: &[Rect], lane_offset: f64) -> Self {
        let pad = 12.0;
        let mut obstacles = Vec::new();
        obstacles.push(start_bounds.inflate(&wps_core::Insets::uniform(pad)));
        obstacles.push(end_bounds.inflate(&wps_core::Insets::uniform(pad)));
        for o in other_obstacles {
            obstacles.push(o.inflate(&wps_core::Insets::uniform(pad)));
        }
        Self {
            obstacles,
            lane_offset,
        }
    }

    pub fn route(
        &self,
        start: Point,
        end: Point,
        start_port: ConnectionPort,
        end_port: ConnectionPort,
    ) -> Vec<Point> {
        // Initial departure stub along port outward normal
        let (snx, sny) = start_port.outward_normal();
        let (enx, eny) = end_port.outward_normal();
        let stub_len = 20.0;

        let start_lead = Point::new(start.x + snx * stub_len, start.y + sny * stub_len);
        let end_lead = Point::new(end.x + enx * stub_len, end.y + eny * stub_len);

        // Discrete grid resolution (20.0 points per grid unit)
        let grid_size = 20.0;
        let to_grid = |p: Point| GridNode {
            x: (p.x / grid_size).round() as i32,
            y: (p.y / grid_size).round() as i32,
        };
        let to_point = |n: GridNode| Point::new(n.x as f64 * grid_size, n.y as f64 * grid_size);

        let start_node = to_grid(start_lead);
        let target_node = to_grid(end_lead);

        // A* search
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<GridNode, GridNode> = HashMap::new();
        let mut g_score: HashMap<GridNode, f64> = HashMap::new();

        g_score.insert(start_node, 0.0);
        open_set.push(AStarState {
            cost: Self::heuristic(start_node, target_node),
            node: start_node,
            dir: (snx.round() as i32, sny.round() as i32),
        });

        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut found = false;
        let mut iterations = 0;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if current.node == target_node || iterations > 800 {
                found = true;
                break;
            }

            let cur_g = *g_score.get(&current.node).unwrap_or(&f64::INFINITY);

            for &(dx, dy) in &dirs {
                let next_node = GridNode {
                    x: current.node.x + dx,
                    y: current.node.y + dy,
                };
                let next_pt = to_point(next_node);

                // Obstacle collision test (allow target node)
                if next_node != target_node && self.is_obstacle_hit(&next_pt) {
                    continue;
                }

                // Turn penalty encourages straight orthogonal runs
                let is_turn = current.dir != (0, 0) && current.dir != (dx, dy);
                let turn_penalty = if is_turn { 25.0 } else { 0.0 };
                let step_cost = grid_size + turn_penalty;

                let tentative_g = cur_g + step_cost;
                if tentative_g < *g_score.get(&next_node).unwrap_or(&f64::INFINITY) {
                    g_score.insert(next_node, tentative_g);
                    came_from.insert(next_node, current.node);
                    let f = tentative_g + Self::heuristic(next_node, target_node);
                    open_set.push(AStarState {
                        cost: f,
                        node: next_node,
                        dir: (dx, dy),
                    });
                }
            }
        }

        // Reconstruct path
        let mut raw_path = Vec::new();
        if found && came_from.contains_key(&target_node) {
            let mut curr = target_node;
            raw_path.push(to_point(curr));
            while let Some(&prev) = came_from.get(&curr) {
                raw_path.push(to_point(prev));
                curr = prev;
            }
            raw_path.reverse();
        } else {
            // Deterministic 3-segment orthogonal fallback with lane offset
            let mid_x = (start_lead.x + end_lead.x) / 2.0 + self.lane_offset;
            raw_path = vec![
                start_lead,
                Point::new(mid_x, start_lead.y),
                Point::new(mid_x, end_lead.y),
                end_lead,
            ];
        }

        // Simplify collinear waypoints to retain only 90-degree corner vertices
        let mut simplified = vec![start];
        if !raw_path.is_empty() {
            simplified.push(raw_path[0]);
            for i in 1..raw_path.len() - 1 {
                let p_prev = raw_path[i - 1];
                let p_curr = raw_path[i];
                let p_next = raw_path[i + 1];

                let is_collinear_x = (p_prev.x - p_curr.x).abs() < 1e-4 && (p_curr.x - p_next.x).abs() < 1e-4;
                let is_collinear_y = (p_prev.y - p_curr.y).abs() < 1e-4 && (p_curr.y - p_next.y).abs() < 1e-4;

                if !(is_collinear_x || is_collinear_y) {
                    simplified.push(p_curr);
                }
            }
            simplified.push(*raw_path.last().unwrap());
        }
        simplified.push(end);

        // Final duplicate check
        simplified.dedup_by(|a, b| a.distance_to(b) < 1.0);
        simplified
    }

    fn heuristic(a: GridNode, b: GridNode) -> f64 {
        ((a.x - b.x).abs() + (a.y - b.y).abs()) as f64 * 20.0
    }

    fn is_obstacle_hit(&self, pt: &Point) -> bool {
        for obs in &self.obstacles {
            if obs.contains_point(pt) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_connector_waypoints() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 50.0);
        let r2 = Rect::new(200.0, 100.0, 100.0, 50.0);

        let connector = SmartConnector::new("c1", "s1", "s2");
        let waypoints = connector.compute_waypoints(&r1, &r2);

        assert!(waypoints.len() >= 4);
        assert_eq!(waypoints[0], Point::new(100.0, 25.0)); // Right port of r1
        assert_eq!(*waypoints.last().unwrap(), Point::new(200.0, 125.0)); // Left port of r2
    }
}
