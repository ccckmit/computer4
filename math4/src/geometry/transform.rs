use crate::geometry::point::{Point2D, Point3D};

#[derive(Clone)]
pub struct Transform2D {
    pub matrix: [[f64; 3]; 3],
}

impl Transform2D {
    pub fn identity() -> Self {
        Transform2D {
            matrix: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(x: f64, y: f64) -> Self {
        Transform2D {
            matrix: [
                [1.0, 0.0, x],
                [0.0, 1.0, y],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scaling(sx: f64, sy: f64) -> Self {
        Transform2D {
            matrix: [
                [sx, 0.0, 0.0],
                [0.0, sy, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Transform2D {
            matrix: [
                [c, -s, 0.0],
                [s, c, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn apply(&self, p: &Point2D) -> Point2D {
        let x = self.matrix[0][0] * p.x + self.matrix[0][1] * p.y + self.matrix[0][2];
        let y = self.matrix[1][0] * p.x + self.matrix[1][1] * p.y + self.matrix[1][2];
        Point2D::new(x, y)
    }

    pub fn compose(&self, other: &Transform2D) -> Transform2D {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.matrix[i][k] * other.matrix[k][j];
                }
            }
        }
        Transform2D { matrix: result }
    }
}

#[derive(Clone)]
pub struct Transform3D {
    pub matrix: [[f64; 4]; 4],
}

impl Transform3D {
    pub fn identity() -> Self {
        Transform3D {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        Transform3D {
            matrix: [
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, y],
                [0.0, 0.0, 1.0, z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scaling(sx: f64, sy: f64, sz: f64) -> Self {
        Transform3D {
            matrix: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [0.0, 0.0, sz, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_x(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Transform3D {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -s, 0.0],
                [0.0, s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Transform3D {
            matrix: [
                [c, 0.0, s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_z(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Transform3D {
            matrix: [
                [c, -s, 0.0, 0.0],
                [s, c, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn apply(&self, p: &Point3D) -> Point3D {
        let x = self.matrix[0][0] * p.x + self.matrix[0][1] * p.y + self.matrix[0][2] * p.z + self.matrix[0][3];
        let y = self.matrix[1][0] * p.x + self.matrix[1][1] * p.y + self.matrix[1][2] * p.z + self.matrix[1][3];
        let z = self.matrix[2][0] * p.x + self.matrix[2][1] * p.y + self.matrix[2][2] * p.z + self.matrix[2][3];
        Point3D::new(x, y, z)
    }

    pub fn compose(&self, other: &Transform3D) -> Transform3D {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self.matrix[i][k] * other.matrix[k][j];
                }
            }
        }
        Transform3D { matrix: result }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let t = Transform2D::identity();
        let p = Point2D::new(1.0, 2.0);
        let result = t.apply(&p);
        assert!((result.x - 1.0).abs() < 1e-10);
        assert!((result.y - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_translation() {
        let t = Transform2D::translation(2.0, 3.0);
        let p = Point2D::new(1.0, 1.0);
        let result = t.apply(&p);
        assert!((result.x - 3.0).abs() < 1e-10);
        assert!((result.y - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation() {
        let t = Transform2D::rotation(std::f64::consts::PI / 2.0);
        let p = Point2D::new(1.0, 0.0);
        let result = t.apply(&p);
        assert!((result.x - 0.0).abs() < 1e-10);
        assert!((result.y - 1.0).abs() < 1e-10);
    }
}