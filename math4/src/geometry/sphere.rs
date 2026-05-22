use crate::geometry::point::Point3D;

pub struct Sphere {
    pub center: Point3D,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Point3D, radius: f64) -> Self {
        Sphere { center, radius }
    }

    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.radius * self.radius * self.radius
    }

    pub fn surface_area(&self) -> f64 {
        4.0 * std::f64::consts::PI * self.radius * self.radius
    }

    pub fn contains_point(&self, p: &Point3D) -> bool {
        self.center.distance(p) <= self.radius + 1e-10
    }

    pub fn intersect_line(&self, p1: &Point3D, p2: &Point3D) -> Option<(Point3D, Point3D)> {
        let cx = self.center.x;
        let cy = self.center.y;
        let cz = self.center.z;
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let dz = p2.z - p1.z;
        let ox = p1.x - cx;
        let oy = p1.y - cy;
        let oz = p1.z - cz;

        let a = dx * dx + dy * dy + dz * dz;
        let b = 2.0 * (dx * ox + dy * oy + dz * oz);
        let c = ox * ox + oy * oy + oz * oz - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        let point1 = Point3D::new(p1.x + t1 * dx, p1.y + t1 * dy, p1.z + t1 * dz);
        let point2 = Point3D::new(p1.x + t2 * dx, p1.y + t2 * dy, p1.z + t2 * dz);

        Some((point1, point2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_volume() {
        let s = Sphere::new(Point3D::new(0.0, 0.0, 0.0), 1.0);
        let expected = (4.0 / 3.0) * std::f64::consts::PI;
        assert!((s.volume() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_surface_area() {
        let s = Sphere::new(Point3D::new(0.0, 0.0, 0.0), 1.0);
        let expected = 4.0 * std::f64::consts::PI;
        assert!((s.surface_area() - expected).abs() < 1e-10);
    }
}