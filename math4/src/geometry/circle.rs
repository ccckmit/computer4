use crate::geometry::point::Point2D;
use crate::geometry::line::Line2D;

pub struct Circle {
    pub center: Point2D,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point2D, radius: f64) -> Self {
        Circle { center, radius }
    }

    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        self.center.distance(p) <= self.radius + 1e-10
    }

    pub fn tangent(&self, p: &Point2D) -> Option<Line2D> {
        let dist = self.center.distance(p);
        if (dist - self.radius).abs() > 1e-10 {
            return None;
        }
        let dx = p.x - self.center.x;
        let dy = p.y - self.center.y;
        Some(Line2D::new(p.clone(), Point2D::new(p.x - dy, p.y + dx)))
    }
}

pub struct Arc {
    pub center: Point2D,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

impl Arc {
    pub fn new(center: Point2D, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        Arc { center, radius, start_angle, end_angle }
    }

    pub fn length(&self) -> f64 {
        let d_angle = self.end_angle - self.start_angle;
        self.radius * d_angle.abs()
    }

    pub fn area(&self) -> f64 {
        let d_angle = self.end_angle - self.start_angle;
        (self.radius * self.radius * d_angle.abs()) / 2.0
    }

    pub fn point(&self, angle: f64) -> Point2D {
        Point2D::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        let dist = self.center.distance(p);
        if dist > self.radius + 1e-10 {
            return false;
        }
        let angle = (p.y - self.center.y).atan2(p.x - self.center.x);
        angle >= self.start_angle && angle <= self.end_angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_area() {
        let c = Circle::new(Point2D::new(0.0, 0.0), 1.0);
        assert!((c.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_circle_circumference() {
        let c = Circle::new(Point2D::new(0.0, 0.0), 1.0);
        assert!((c.circumference() - 2.0 * std::f64::consts::PI).abs() < 1e-10);
    }
}