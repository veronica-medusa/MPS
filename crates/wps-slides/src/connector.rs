use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorRouting {
    Straight,
    Orthogonal,
    Curved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartConnector {
    pub id: String,
    pub start_shape_id: String,
    pub start_port: ConnectionPort,
    pub end_shape_id: String,
    pub end_port: ConnectionPort,
    pub routing: ConnectorRouting,
    pub color: Color,
    pub stroke_width: f64,
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
            color: Color::MERCURY_INK,
            stroke_width: 2.0,
        }
    }

    pub fn port_coordinate(bounds: &Rect, port: ConnectionPort) -> Point {
        match port {
            ConnectionPort::Top => Point::new(bounds.x + bounds.width / 2.0, bounds.y),
            ConnectionPort::Right => Point::new(bounds.x + bounds.width, bounds.y + bounds.height / 2.0),
            ConnectionPort::Bottom => Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height),
            ConnectionPort::Left => Point::new(bounds.x, bounds.y + bounds.height / 2.0),
            ConnectionPort::Center => bounds.center(),
        }
    }

    /// Computes waypoints for orthogonal routing avoiding direct shape collisions
    pub fn compute_waypoints(&self, start_bounds: &Rect, end_bounds: &Rect) -> Vec<Point> {
        let p_start = Self::port_coordinate(start_bounds, self.start_port);
        let p_end = Self::port_coordinate(end_bounds, self.end_port);

        match self.routing {
            ConnectorRouting::Straight => vec![p_start, p_end],
            ConnectorRouting::Orthogonal => {
                let mid_x = (p_start.x + p_end.x) / 2.0;
                let w1 = Point::new(mid_x, p_start.y);
                let w2 = Point::new(mid_x, p_end.y);
                vec![p_start, w1, w2, p_end]
            }
            ConnectorRouting::Curved => {
                vec![p_start, p_end]
            }
        }
    }

    pub fn render_to_display_list(&self, start_bounds: &Rect, end_bounds: &Rect, dl: &mut DisplayList) {
        let waypoints = self.compute_waypoints(start_bounds, end_bounds);
        for i in 0..waypoints.len() - 1 {
            dl.push(DrawCommand::DrawLine {
                from: waypoints[i],
                to: waypoints[i + 1],
                color: self.color,
                width: self.stroke_width,
            });
        }
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

        assert_eq!(waypoints.len(), 4);
        assert_eq!(waypoints[0], Point::new(100.0, 25.0)); // Right port of r1
        assert_eq!(waypoints[3], Point::new(200.0, 125.0)); // Left port of r2
    }
}
