use std::fmt;

fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

#[derive(Clone)]
pub struct Rational {
    num: i64,
    den: i64,
}

impl Rational {
    pub fn new(num: i64, den: i64) -> Self {
        if den == 0 {
            panic!("Denominator cannot be zero");
        }
        let g = gcd(num, den);
        let sign = if den < 0 { -1 } else { 1 };
        Rational {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    pub fn num(&self) -> i64 {
        self.num
    }

    pub fn den(&self) -> i64 {
        self.den
    }

    pub fn add(&self, other: &Rational) -> Rational {
        Rational::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn sub(&self, other: &Rational) -> Rational {
        Rational::new(
            self.num * other.den - other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn mul(&self, other: &Rational) -> Rational {
        Rational::new(self.num * other.num, self.den * other.den)
    }

    pub fn div(&self, other: &Rational) -> Rational {
        if other.num == 0 {
            panic!("Cannot divide by zero");
        }
        Rational::new(self.num * other.den, self.den * other.num)
    }

    pub fn simplify(&self) -> Rational {
        let g = gcd(self.num, self.den);
        let sign = if self.den < 0 { -1 } else { 1 };
        Rational::new(sign * self.num / g, sign * self.den / g)
    }

    pub fn to_number(&self) -> f64 {
        self.num as f64 / self.den as f64
    }

    pub fn equals(&self, other: &Rational) -> bool {
        self.num == other.num && self.den == other.den
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

pub fn parse_rational(s: &str) -> Rational {
    let parts: Vec<&str> = s.trim().split('/').collect();
    if parts.len() == 1 {
        Rational::new(parts[0].parse().unwrap_or(0), 1)
    } else {
        Rational::new(
            parts[0].parse().unwrap_or(0),
            parts[1].parse().unwrap_or(1),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let r = Rational::new(2, 4);
        assert_eq!(r.num(), 1);
        assert_eq!(r.den(), 2);
    }

    #[test]
    fn test_negative() {
        let r = Rational::new(-3, -6);
        assert_eq!(r.num(), 1);
        assert_eq!(r.den(), 2);
    }

    #[test]
    fn test_zero() {
        let r = Rational::new(0, 5);
        assert_eq!(r.num(), 0);
        assert_eq!(r.den(), 1);
    }

    #[test]
    fn test_add() {
        let r1 = Rational::new(1, 2);
        let r2 = Rational::new(1, 3);
        let result = r1.add(&r2);
        assert_eq!(result.num(), 5);
        assert_eq!(result.den(), 6);
    }

    #[test]
    fn test_sub() {
        let r1 = Rational::new(1, 2);
        let r2 = Rational::new(1, 3);
        let result = r1.sub(&r2);
        assert_eq!(result.num(), 1);
        assert_eq!(result.den(), 6);
    }

    #[test]
    fn test_mul() {
        let r1 = Rational::new(2, 3);
        let r2 = Rational::new(3, 4);
        let result = r1.mul(&r2);
        assert_eq!(result.num(), 1);
        assert_eq!(result.den(), 2);
    }

    #[test]
    fn test_div() {
        let r1 = Rational::new(1, 2);
        let r2 = Rational::new(2, 1);
        let result = r1.div(&r2);
        assert_eq!(result.num(), 1);
        assert_eq!(result.den(), 4);
    }

    #[test]
    fn test_to_number() {
        let r = Rational::new(1, 2);
        assert!((r.to_number() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_equals() {
        let r1 = Rational::new(1, 2);
        let r2 = Rational::new(2, 4);
        assert!(r1.equals(&r2));
    }

    #[test]
    fn test_parse_rational_integer() {
        let r = parse_rational("5");
        assert_eq!(r.num(), 5);
        assert_eq!(r.den(), 1);
    }

    #[test]
    fn test_parse_rational_fraction() {
        let r = parse_rational("3/4");
        assert_eq!(r.num(), 3);
        assert_eq!(r.den(), 4);
    }
}