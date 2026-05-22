use crate::geometry::point::{Point2D, Point3D};
use crate::geometry::line::Line2D;

pub fn point_to_point_2d(p1: &Point2D, p2: &Point2D) -> f64 {
    p1.distance(p2)
}

pub fn point_to_point_3d(p1: &Point3D, p2: &Point3D) -> f64 {
    p1.distance(p2)
}

pub fn point_to_line_2d(p: &Point2D, line: &Line2D) -> f64 {
    let a = line.p2.y - line.p1.y;
    let b = line.p1.x - line.p2.x;
    let c = line.p2.x * line.p1.y - line.p1.x * line.p2.y;
    let denom = (a * a + b * b).sqrt();
    if denom < 1e-10 {
        return 0.0;
    }
    (a * p.x + b * p.y + c).abs() / denom
}

pub fn line_to_line_2d(l1: &Line2D, l2: &Line2D) -> f64 {
    let a1 = l1.p2.y - l1.p1.y;
    let b1 = l1.p1.x - l1.p2.x;
    let c1 = l1.p2.x * l1.p1.y - l1.p1.x * l1.p2.y;
    let a2 = l2.p2.y - l2.p1.y;
    let b2 = l2.p1.x - l2.p2.x;
    let c2 = l2.p2.x * l2.p1.y - l2.p1.x * l2.p2.y;

    let denom = a1 * b2 - a2 * b1;
    if denom.abs() < 1e-10 {
        let dist1 = point_to_line_2d(&l1.p1, l2);
        let dist2 = point_to_line_2d(&l1.p2, l2);
        return dist1.min(dist2);
    }

    let x = (b1 * c2 - b2 * c1) / denom;
    let y = (c1 * a2 - c2 * a1) / denom;
    let intersection = Point2D::new(x, y);
    let d1 = l1.p1.distance(&intersection);
    let d2 = l1.p2.distance(&intersection);
    let segment_dist = if d1 <= d2 { d1 } else { d2 };

    let on_l1 = intersection_on_segment(&intersection, l1);
    let on_l2 = intersection_on_segment(&intersection, l2);

    if on_l1 && on_l2 {
        0.0
    } else {
        segment_dist
    }
}

fn intersection_on_segment(p: &Point2D, line: &Line2D) -> bool {
    let min_x = line.p1.x.min(line.p2.x) - 1e-10;
    let max_x = line.p1.x.max(line.p2.x) + 1e-10;
    let min_y = line.p1.y.min(line.p2.y) - 1e-10;
    let max_y = line.p1.y.max(line.p2.y) + 1e-10;
    p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_to_point() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);
        assert!((point_to_point_2d(&p1, &p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_to_line() {
        let p = Point2D::new(1.0, 1.0);
        let line = Line2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0));
        assert!((point_to_line_2d(&p, &line) - 1.0).abs() < 1e-10);
    }
}