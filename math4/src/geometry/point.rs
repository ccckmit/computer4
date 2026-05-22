use std::fmt;

#[derive(Clone)]
pub struct Point1D {
    pub x: f64,
}

impl Point1D {
    pub fn new(x: f64) -> Self {
        Point1D { x }
    }

    pub fn distance(&self, other: &Point1D) -> f64 {
        (self.x - other.x).abs()
    }

    pub fn midpoint(&self, other: &Point1D) -> Point1D {
        Point1D::new((self.x + other.x) / 2.0)
    }

    pub fn translate(&self, v: f64) -> Point1D {
        Point1D::new(self.x + v)
    }

    pub fn scale(&self, factor: f64) -> Point1D {
        Point1D::new(self.x * factor)
    }

    pub fn to_array(&self) -> Vec<f64> {
        vec![self.x]
    }
}

impl fmt::Display for Point1D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.x)
    }
}

#[derive(Clone)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Point2D { x, y }
    }

    pub fn distance(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, other: &Point2D) -> Point2D {
        Point2D::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
    }

    pub fn translate(&self, v: &[f64; 2]) -> Point2D {
        Point2D::new(self.x + v[0], self.y + v[1])
    }

    pub fn scale(&self, factor: f64) -> Point2D {
        Point2D::new(self.x * factor, self.y * factor)
    }

    pub fn to_array(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
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

    pub fn midpoint(&self, other: &Point3D) -> Point3D {
        Point3D::new(
            (self.x + other.x) / 2.0,
            (self.y + other.y) / 2.0,
            (self.z + other.z) / 2.0,
        )
    }

    pub fn translate(&self, v: &[f64; 3]) -> Point3D {
        Point3D::new(self.x + v[0], self.y + v[1], self.z + v[2])
    }

    pub fn scale(&self, factor: f64) -> Point3D {
        Point3D::new(self.x * factor, self.y * factor, self.z * factor)
    }

    pub fn to_array(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z]
    }
}

impl fmt::Display for Point3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Clone)]
pub struct PointND {
    pub coords: Vec<f64>,
}

impl PointND {
    pub fn new(coords: Vec<f64>) -> Self {
        PointND { coords }
    }

    pub fn from_array(arr: &[f64]) -> PointND {
        PointND::new(arr.to_vec())
    }

    pub fn distance(&self, other: &PointND) -> f64 {
        let sum: f64 = self
            .coords
            .iter()
            .zip(other.coords.iter())
            .map(|(c1, c2)| {
                let d = c1 - c2;
                d * d
            })
            .sum();
        sum.sqrt()
    }

    pub fn midpoint(&self, other: &PointND) -> PointND {
        let result: Vec<f64> = self
            .coords
            .iter()
            .zip(other.coords.iter())
            .map(|(c1, c2)| (c1 + c2) / 2.0)
            .collect();
        PointND::new(result)
    }

    pub fn translate(&self, v: &[f64]) -> PointND {
        let result: Vec<f64> = self
            .coords
            .iter()
            .enumerate()
            .map(|(i, c)| c + v.get(i).unwrap_or(&0.0))
            .collect();
        PointND::new(result)
    }

    pub fn scale(&self, factor: f64) -> PointND {
        PointND::new(self.coords.iter().map(|c| c * factor).collect())
    }

    pub fn dim(&self) -> usize {
        self.coords.len()
    }

    pub fn to_array(&self) -> Vec<f64> {
        self.coords.clone()
    }
}

impl fmt::Display for PointND {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.coords.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point1d() {
        let p = Point1D::new(3.0);
        assert!((p.x - 3.0).abs() < 1e-10);
        assert!((p.distance(&Point1D::new(5.0)) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_point2d() {
        let p = Point2D::new(3.0, 4.0);
        assert!((p.distance(&Point2D::new(0.0, 0.0)) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point3d() {
        let p = Point3D::new(1.0, 2.0, 2.0);
        let other = Point3D::new(1.0, 2.0, 5.0);
        assert!((p.distance(&other) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_pointnd() {
        let p1 = PointND::new(vec![1.0, 2.0, 3.0]);
        let p2 = PointND::new(vec![4.0, 6.0, 9.0]);
        let dist = p1.distance(&p2);
        assert!(dist > 0.0);
    }
}