use crate::geometry::point::Point2D;

pub struct Polygon {
    pub vertices: Vec<Point2D>,
}

impl Polygon {
    pub fn new(vertices: Vec<Point2D>) -> Self {
        Polygon { vertices }
    }

    pub fn area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }
        let mut sum = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            sum += self.vertices[i].x * self.vertices[j].y;
            sum -= self.vertices[j].x * self.vertices[i].y;
        }
        sum.abs() / 2.0
    }

    pub fn perimeter(&self) -> f64 {
        let n = self.vertices.len();
        if n < 2 {
            return 0.0;
        }
        let mut sum = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            sum += self.vertices[i].distance(&self.vertices[j]);
        }
        sum
    }

    pub fn contains_point(&self, p: &Point2D) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];
            if ((vi.y > p.y) != (vj.y > p.y)) && (p.x < (vj.x - vi.x) * (p.y - vi.y) / (vj.y - vi.y) + vi.x) {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    pub fn centroid(&self) -> Option<Point2D> {
        let n = self.vertices.len();
        if n == 0 {
            return None;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        for v in &self.vertices {
            cx += v.x;
            cy += v.y;
        }
        cx /= n as f64;
        cy /= n as f64;
        Some(Point2D::new(cx, cy))
    }
}

pub fn triangle_area(p1: &Point2D, p2: &Point2D, p3: &Point2D) -> f64 {
    Polygon::new(vec![p1.clone(), p2.clone(), p3.clone()]).area()
}

pub fn quadrilateral_area(p1: &Point2D, p2: &Point2D, p3: &Point2D, p4: &Point2D) -> f64 {
    Polygon::new(vec![p1.clone(), p2.clone(), p3.clone(), p4.clone()]).area()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 0.0);
        let p3 = Point2D::new(0.0, 4.0);
        assert!((triangle_area(&p1, &p2, &p3) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_polygon_area() {
        let vertices = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(4.0, 0.0),
            Point2D::new(4.0, 3.0),
            Point2D::new(0.0, 3.0),
        ];
        let poly = Polygon::new(vertices);
        assert!((poly.area() - 12.0).abs() < 1e-10);
    }
}