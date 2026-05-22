use std::fmt;

#[derive(Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    pub fn add(&self, v: &Vec2) -> Vec2 {
        Vec2::new(self.x + v.x, self.y + v.y)
    }

    pub fn sub(&self, v: &Vec2) -> Vec2 {
        Vec2::new(self.x - v.x, self.y - v.y)
    }

    pub fn mul(&self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }

    pub fn dot(&self, v: &Vec2) -> f64 {
        self.x * v.x + self.y * v.y
    }

    pub fn cross(&self, v: &Vec2) -> f64 {
        self.x * v.y - self.y * v.x
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len == 0.0 {
            Vec2::new(0.0, 0.0)
        } else {
            Vec2::new(self.x / len, self.y / len)
        }
    }

    pub fn project(&self, v: &Vec2) -> Vec2 {
        let dot = self.dot(v);
        let len_sq = v.length().powi(2);
        if len_sq == 0.0 {
            Vec2::new(0.0, 0.0)
        } else {
            let t = dot / len_sq;
            v.mul(t)
        }
    }

    pub fn reflect(&self, normal: &Vec2) -> Vec2 {
        let n = normal.normalize();
        let dot = self.dot(&n);
        self.sub(&n.mul(2.0 * dot))
    }

    pub fn to_array(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn add(&self, v: &Vec3) -> Vec3 {
        Vec3::new(self.x + v.x, self.y + v.y, self.z + v.z)
    }

    pub fn sub(&self, v: &Vec3) -> Vec3 {
        Vec3::new(self.x - v.x, self.y - v.y, self.z - v.z)
    }

    pub fn mul(&self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn dot(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn cross(&self, v: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
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

    pub fn project(&self, v: &Vec3) -> Vec3 {
        let dot = self.dot(v);
        let len_sq = v.length().powi(2);
        if len_sq == 0.0 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            let t = dot / len_sq;
            v.mul(t)
        }
    }

    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        let n = normal.normalize();
        let dot = self.dot(&n);
        self.sub(&n.mul(2.0 * dot))
    }

    pub fn to_array(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z]
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Clone)]
pub struct VecN {
    coords: Vec<f64>,
}

impl VecN {
    pub fn new(coords: Vec<f64>) -> Self {
        VecN { coords }
    }

    pub fn from_slice(arr: &[f64]) -> VecN {
        VecN::new(arr.to_vec())
    }

    pub fn add(&self, v: &VecN) -> VecN {
        let result: Vec<f64> = self
            .coords
            .iter()
            .zip(v.coords.iter())
            .map(|(c1, c2)| c1 + c2)
            .collect();
        VecN::new(result)
    }

    pub fn sub(&self, v: &VecN) -> VecN {
        let result: Vec<f64> = self
            .coords
            .iter()
            .zip(v.coords.iter())
            .map(|(c1, c2)| c1 - c2)
            .collect();
        VecN::new(result)
    }

    pub fn mul(&self, s: f64) -> VecN {
        VecN::new(self.coords.iter().map(|c| c * s).collect())
    }

    pub fn dot(&self, v: &VecN) -> f64 {
        self.coords
            .iter()
            .zip(v.coords.iter())
            .map(|(c1, c2)| c1 * c2)
            .sum()
    }

    pub fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&self) -> VecN {
        let len = self.length();
        if len == 0.0 {
            VecN::new(self.coords.iter().map(|_| 0.0).collect())
        } else {
            self.mul(1.0 / len)
        }
    }

    pub fn dim(&self) -> usize {
        self.coords.len()
    }

    pub fn to_vec(&self) -> Vec<f64> {
        self.coords.clone()
    }
}

impl fmt::Display for VecN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.coords.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vec3() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        let w = Vec3::new(0.0, 1.0, 0.0);
        let cross = v.cross(&w);
        assert!((cross.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vecn() {
        let v = VecN::new(vec![1.0, 2.0, 3.0]);
        assert!((v.length() - 14.0_f64.sqrt()).abs() < 1e-10);
    }
}