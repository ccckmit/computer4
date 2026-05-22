use crate::geometry::vector::Vec3;
use crate::geometry::point::Point3D;

pub struct Plane {
    pub normal: Vec3,
    pub point: Point3D,
}

impl Plane {
    pub fn new(normal: Vec3, point: Point3D) -> Self {
        Plane { normal, point }
    }

    pub fn from_points(p1: &Point3D, p2: &Point3D, p3: &Point3D) -> Plane {
        let v1 = Vec3::new(p2.x - p1.x, p2.y - p1.y, p2.z - p1.z);
        let v2 = Vec3::new(p3.x - p1.x, p3.y - p1.y, p3.z - p1.z);
        let normal = v1.cross(&v2).normalize();
        Plane::new(normal, p1.clone())
    }

    pub fn distance_to(&self, point: &Point3D) -> f64 {
        let v = Vec3::new(point.x - self.point.x, point.y - self.point.y, point.z - self.point.z);
        v.dot(&self.normal).abs()
    }

    pub fn contains_point(&self, p: &Point3D) -> bool {
        self.distance_to(p) < 1e-10
    }

    pub fn intersect_line(&self, p1: &Point3D, p2: &Point3D) -> Option<Point3D> {
        let d = Vec3::new(p2.x - p1.x, p2.y - p1.y, p2.z - p1.z);
        let denom = self.normal.dot(&d);
        if denom.abs() < 1e-10 {
            return None;
        }
        let v = Vec3::new(self.point.x - p1.x, self.point.y - p1.y, self.point.z - p1.z);
        let t = v.dot(&self.normal) / denom;
        if t < 0.0 || t > 1.0 {
            return None;
        }
        Some(Point3D::new(p1.x + t * d.x, p1.y + t * d.y, p1.z + t * d.z))
    }

    pub fn project(&self, point: &Point3D) -> Point3D {
        let v = Vec3::new(point.x - self.point.x, point.y - self.point.y, point.z - self.point.z);
        let dist = v.dot(&self.normal);
        let proj = v.sub(&self.normal.mul(dist));
        Point3D::new(self.point.x + proj.x, self.point.y + proj.y, self.point.z + proj.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_distance() {
        let plane = Plane::new(Vec3::new(0.0, 0.0, 1.0), Point3D::new(0.0, 0.0, 0.0));
        let p = Point3D::new(0.0, 0.0, 5.0);
        assert!((plane.distance_to(&p) - 5.0).abs() < 1e-10);
    }
}