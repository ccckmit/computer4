use std::fmt;

#[derive(Clone)]
pub struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    pub fn re(&self) -> f64 {
        self.re
    }

    pub fn im(&self) -> f64 {
        self.im
    }

    pub fn add(&self, other: &Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }

    pub fn sub(&self, other: &Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }

    pub fn mul(&self, other: &Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    pub fn div(&self, other: &Complex) -> Complex {
        let denom = other.re * other.re + other.im * other.im;
        if denom == 0.0 {
            panic!("Cannot divide by zero");
        }
        Complex::new(
            (self.re * other.re + self.im * other.im) / denom,
            (self.im * other.re - self.re * other.im) / denom,
        )
    }

    pub fn conj(&self) -> Complex {
        Complex::new(self.re, -self.im)
    }

    pub fn abs(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn sqrt(&self) -> Complex {
        let r = self.abs();
        let theta = self.arg();
        let sqrt_r = r.sqrt();
        Complex::new(sqrt_r * (theta / 2.0).cos(), sqrt_r * (theta / 2.0).sin())
    }

    pub fn exp(&self) -> Complex {
        let er = self.re.exp();
        Complex::new(er * self.im.cos(), er * self.im.sin())
    }

    pub fn log(&self) -> Complex {
        Complex::new(self.abs().ln(), self.arg())
    }

    pub fn pow(&self, n: f64) -> Complex {
        let r = self.abs();
        let theta = self.arg();
        let rn = r.powf(n);
        Complex::new(rn * (n * theta).cos(), rn * (n * theta).sin())
    }

    pub fn sin(&self) -> Complex {
        Complex::new(
            self.re.sin() * (self.im).cosh(),
            self.re.cos() * (self.im).sinh(),
        )
    }

    pub fn cos(&self) -> Complex {
        Complex::new(
            self.re.cos() * (self.im).cosh(),
            -self.re.sin() * (self.im).sinh(),
        )
    }

    pub fn tan(&self) -> Complex {
        self.sin().div(&self.cos())
    }

    pub fn equals(&self, other: &Complex) -> bool {
        self.re == other.re && self.im == other.im
    }

    pub fn to_polar(&self) -> (f64, f64) {
        (self.abs(), self.arg())
    }

    pub fn from_polar(r: f64, theta: f64) -> Complex {
        Complex::new(r * theta.cos(), r * theta.sin())
    }

    pub fn i() -> Complex {
        Complex::new(0.0, 1.0)
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im == 0.0 {
            write!(f, "{}", self.re)
        } else if self.re == 0.0 {
            if (self.im - 1.0).abs() < 1e-10 {
                write!(f, "i")
            } else if (self.im - (-1.0)).abs() < 1e-10 {
                write!(f, "-i")
            } else {
                write!(f, "{}i", self.im)
            }
        } else {
            let sign = if self.im >= 0.0 { "+" } else { "-" };
            write!(f, "{} {} {}i", self.re, sign, self.im.abs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let c = Complex::new(3.0, 4.0);
        assert!((c.re() - 3.0).abs() < 1e-10);
        assert!((c.im() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_create_zero() {
        let c = Complex::new(0.0, 0.0);
        assert!((c.re() - 0.0).abs() < 1e-10);
        assert!((c.im() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_add() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1.add(&c2);
        assert!((result.re() - 4.0).abs() < 1e-10);
        assert!((result.im() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_sub() {
        let c1 = Complex::new(5.0, 6.0);
        let c2 = Complex::new(2.0, 3.0);
        let result = c1.sub(&c2);
        assert!((result.re() - 3.0).abs() < 1e-10);
        assert!((result.im() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_mul() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);
        let result = c1.mul(&c2);
        assert!((result.re() - (-5.0)).abs() < 1e-10);
        assert!((result.im() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_div() {
        let c1 = Complex::new(1.0, 1.0);
        let c2 = Complex::new(1.0, 1.0);
        let result = c1.div(&c2);
        assert!((result.re() - 1.0).abs() < 1e-10);
        assert!((result.im() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_conj() {
        let c = Complex::new(3.0, 4.0);
        let result = c.conj();
        assert!((result.re() - 3.0).abs() < 1e-10);
        assert!((result.im() - (-4.0)).abs() < 1e-10);
    }

    #[test]
    fn test_abs() {
        let c = Complex::new(3.0, 4.0);
        assert!((c.abs() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_arg() {
        let c = Complex::new(1.0, 1.0);
        assert!((c.arg() - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
    }

    #[test]
    fn test_sqrt() {
        let c = Complex::new(-1.0, 0.0);
        let result = c.sqrt();
        assert!((result.re() - 0.0).abs() < 1e-10);
        assert!((result.im() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_exp() {
        let c = Complex::new(0.0, std::f64::consts::PI);
        let result = c.exp();
        assert!((result.re() - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_log() {
        let c = Complex::new(1.0, 0.0);
        let result = c.log();
        assert!((result.re() - 0.0).abs() < 1e-10);
        assert!((result.im() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_pow() {
        let c = Complex::new(1.0, 0.0);
        let result = c.pow(3.0);
        assert!((result.re() - 1.0).abs() < 1e-10);
        assert!((result.im() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sin() {
        let c = Complex::new(0.0, 0.0);
        let result = c.sin();
        assert!((result.re() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cos() {
        let c = Complex::new(0.0, 0.0);
        let result = c.cos();
        assert!((result.re() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_equals() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(1.0, 2.0);
        assert!(c1.equals(&c2));
    }

    #[test]
    fn test_from_polar() {
        let c = Complex::from_polar(1.0, std::f64::consts::FRAC_PI_2);
        assert!((c.re() - 0.0).abs() < 1e-10);
        assert!((c.im() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_i() {
        let c = Complex::i();
        assert!((c.re() - 0.0).abs() < 1e-10);
        assert!((c.im() - 1.0).abs() < 1e-10);
    }
}