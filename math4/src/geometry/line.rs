use crate::geometry::point::Point2D;
use crate::geometry::vector::Vec2;

pub struct Line2D {
    pub p1: Point2D,
    pub p2: Point2D,
}

impl Line2D {
    pub fn new(p1: Point2D, p2: Point2D) -> Self {
        Line2D { p1, p2 }
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        let v1 = vec![self.p2.x - self.p1.x, self.p2.y - self.p1.y];
        let v2 = vec![p.x - self.p1.x, p.y - self.p1.y];
        let cross = v1[0] * v2[1] - v1[1] * v2[0];
        cross.abs() < 1e-10
    }

    pub fn distance_to(&self, p: &Point2D) -> f64 {
        let a = self.p2.y - self.p1.y;
        let b = self.p1.x - self.p2.x;
        let c = -a * self.p1.x - b * self.p1.y;
        (a * p.x + b * p.y + c).abs() / (a * a + b * b).sqrt()
    }

    pub fn intersect(&self, other: &Line2D) -> Option<Point2D> {
        let x1 = self.p1.x;
        let y1 = self.p1.y;
        let x2 = self.p2.x;
        let y2 = self.p2.y;
        let x3 = other.p1.x;
        let y3 = other.p1.y;
        let x4 = other.p2.x;
        let y4 = other.p2.y;

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if denom.abs() < 1e-10 {
            return None;
        }

        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
        Some(Point2D::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1)))
    }

    pub fn direction(&self) -> Vec2 {
        Vec2::new(self.p2.x - self.p1.x, self.p2.y - self.p1.y)
    }
}

pub struct Line3D {
    pub p1: Point3D,
    pub p2: Point3D,
}

impl Line3D {
    pub fn new(p1: Point3D, p2: Point3D) -> Self {
        Line3D { p1, p2 }
    }

    pub fn direction(&self) -> Vec3 {
        Vec3::new(
            self.p2.x - self.p1.x,
            self.p2.y - self.p1.y,
            self.p2.z - self.p1.z,
        )
    }

    pub fn distance_to_point(&self, p: &Point3D) -> f64 {
        let d = self.direction();
        let v = Vec3::new(p.x - self.p1.x, p.y - self.p1.y, p.z - self.p1.z);
        let cross = d.cross(&v);
        cross.length() / d.length()
    }
}

pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3D { x, y, z }
    }

    pub fn distance(&self, other: &Point3D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn cross(&self, v: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }

    pub fn dot(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        }
    }

    pub fn mul(&self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn sub(&self, v: &Vec3) -> Vec3 {
        Vec3::new(self.x - v.x, self.y - v.y, self.z - v.z)
    }
}

pub struct Ray {
    pub origin: Point2D,
    pub direction: Vec2,
}

impl Ray {
    pub fn new(origin: Point2D, direction: Vec2) -> Self {
        Ray { origin, direction }
    }

    pub fn point(&self, t: f64) -> Point2D {
        Point2D::new(
            self.origin.x + t * self.direction.x,
            self.origin.y + t * self.direction.y,
        )
    }

    pub fn intersect_circle(&self, center: &Point2D, radius: f64) -> Option<IntersectResult> {
        let oc = Vec2::new(center.x - self.origin.x, center.y - self.origin.y);
        let a = self.direction.dot(&self.direction);
        let b = 2.0 * oc.dot(&self.direction);
        let c = oc.dot(&oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        Some(IntersectResult { t1, t2 })
    }
}

#[derive(Debug)]
pub struct IntersectResult {
    pub t1: f64,
    pub t2: f64,
}

pub struct Segment {
    pub p1: Point2D,
    pub p2: Point2D,
}

impl Segment {
    pub fn new(p1: Point2D, p2: Point2D) -> Self {
        Segment { p1, p2 }
    }

    pub fn length(&self) -> f64 {
        self.p1.distance(&self.p2)
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        let d1 = self.p1.distance(p);
        let d2 = self.p2.distance(p);
        let total = self.length();
        (d1 + d2 - total).abs() < 1e-10
    }

    pub fn midpoint(&self) -> Point2D {
        self.p1.midpoint(&self.p2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line2d_contains() {
        let line = Line2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0));
        assert!(line.contains_point(&Point2D::new(0.5, 0.5)));
    }

    #[test]
    fn test_segment_length() {
        let seg = Segment::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0));
        assert!((seg.length() - 5.0).abs() < 1e-10);
    }
}