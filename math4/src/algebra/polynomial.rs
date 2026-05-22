use std::fmt;

#[derive(Clone)]
pub struct Polynomial {
    coeffs: Vec<f64>,
}

impl Polynomial {
    pub fn new(coeffs: Vec<f64>) -> Self {
        let mut p = Polynomial { coeffs };
        p.trim();
        p
    }

    fn trim(&mut self) {
        while self.coeffs.len() > 1 {
            let last = self.coeffs[self.coeffs.len() - 1];
            if (last - 0.0).abs() < 1e-10 {
                self.coeffs.pop();
            } else {
                break;
            }
        }
    }

    pub fn coeffs(&self) -> Vec<f64> {
        self.coeffs.clone()
    }

    pub fn degree(&self) -> i32 {
        if self.coeffs.is_empty() {
            0
        } else {
            self.coeffs.len() as i32 - 1
        }
    }

    pub fn eval(&self, x: f64) -> f64 {
        let mut result = 0.0;
        for i in (0..self.coeffs.len()).rev() {
            result = result * x + self.coeffs[i];
        }
        result
    }

    pub fn add(&self, other: &Polynomial) -> Polynomial {
        let max_len = self.coeffs.len().max(other.coeffs.len());
        let mut result = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let a = self.coeffs.get(i).unwrap_or(&0.0);
            let b = other.coeffs.get(i).unwrap_or(&0.0);
            result.push(a + b);
        }
        Polynomial::new(result)
    }

    pub fn sub(&self, other: &Polynomial) -> Polynomial {
        let max_len = self.coeffs.len().max(other.coeffs.len());
        let mut result = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let a = self.coeffs.get(i).unwrap_or(&0.0);
            let b = other.coeffs.get(i).unwrap_or(&0.0);
            result.push(a - b);
        }
        Polynomial::new(result)
    }

    pub fn mul(&self, other: &Polynomial) -> Polynomial {
        if self.coeffs.is_empty() || other.coeffs.is_empty() {
            return Polynomial::new(vec![0.0]);
        }
        let len = self.coeffs.len() + other.coeffs.len() - 1;
        let mut result = vec![0.0; len];
        for i in 0..self.coeffs.len() {
            for j in 0..other.coeffs.len() {
                result[i + j] += self.coeffs[i] * other.coeffs[j];
            }
        }
        Polynomial::new(result)
    }

    pub fn scalar_mul(&self, s: f64) -> Polynomial {
        Polynomial::new(self.coeffs.iter().map(|c| c * s).collect())
    }

    pub fn derivative(&self) -> Polynomial {
        if self.coeffs.len() <= 1 {
            return Polynomial::new(vec![0.0]);
        }
        let mut result = Vec::new();
        for i in 1..self.coeffs.len() {
            result.push(i as f64 * self.coeffs[i]);
        }
        Polynomial::new(result)
    }

    pub fn integral(&self, c: f64) -> Polynomial {
        let mut result = vec![c];
        for i in 0..self.coeffs.len() {
            result.push(self.coeffs[i] / (i as f64 + 1.0));
        }
        Polynomial::new(result)
    }

    pub fn compose(&self, other: &Polynomial) -> Polynomial {
        let mut result = Polynomial::new(vec![0.0]);
        for i in (0..self.coeffs.len()).rev() {
            result = result.mul(other);
            result = result.add(&Polynomial::new(vec![self.coeffs[i]]));
        }
        result
    }

    pub fn divide(&self, d: &Polynomial) -> (Polynomial, Polynomial) {
        if d.degree() < 0 {
            panic!("Cannot divide by zero polynomial");
        }
        let mut remainder = Polynomial::new(self.coeffs.clone());
        let divisor_deg = d.degree() as usize;
        let divisor_lead = d.coeffs[divisor_deg];
        let quotient_len = if self.degree() as i32 - divisor_deg as i32 + 1 > 0 {
            (self.degree() as i32 - divisor_deg as i32 + 1) as usize
        } else {
            0
        };
        let mut quotient_coeffs = vec![0.0; quotient_len];

        while remainder.degree() >= divisor_deg as i32 && remainder.degree() > 0 {
            let deg_diff = (remainder.degree() - divisor_deg as i32) as usize;
            let lead_coeff = remainder.coeffs[remainder.degree() as usize] / divisor_lead;
            quotient_coeffs[deg_diff] = lead_coeff;
            let mut term_coeffs = vec![0.0; deg_diff + 1];
            term_coeffs[deg_diff] = lead_coeff;
            let term = Polynomial::new(term_coeffs);
            remainder = remainder.sub(&term.mul(d));
        }

        (Polynomial::new(quotient_coeffs), remainder)
    }

    pub fn r#mod(&self, d: &Polynomial) -> Polynomial {
        self.divide(d).1
    }

    pub fn gcd(&self, other: &Polynomial) -> Polynomial {
        let mut a = Polynomial::new(self.coeffs.clone());
        let mut b = Polynomial::new(other.coeffs.clone());
        let mut iterations = 0;
        let max_iter = 10000;

        while iterations < max_iter && (b.degree() > 0 || (b.coeffs.len() > 0 && (b.coeffs[0] - 0.0).abs() > 1e-10)) {
            let r = a.r#mod(&b);
            a = b;
            b = r;
            iterations += 1;
        }

        if iterations >= max_iter {
            return Polynomial::new(vec![1.0]);
        }

        let lead = a.coeffs[a.degree() as usize];
        if lead != 1.0 && lead != -1.0 && lead != 0.0 {
            return a.scalar_mul(1.0 / lead);
        }
        a
    }

    pub fn equals(&self, other: &Polynomial) -> bool {
        if self.degree() != other.degree() {
            return false;
        }
        for i in 0..=self.degree() as usize {
            if (self.coeffs[i] - other.coeffs[i]).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.coeffs.is_empty() {
            return write!(f, "0");
        }
        let mut terms = Vec::new();
        for i in (0..self.coeffs.len()).rev() {
            let c = self.coeffs[i];
            if c.abs() < 1e-10 {
                continue;
            }
            if i == 0 {
                terms.push(format!("{}", c));
            } else if i == 1 {
                if (c - 1.0).abs() < 1e-10 {
                    terms.push("x".to_string());
                } else if (c - (-1.0)).abs() < 1e-10 {
                    terms.push("-x".to_string());
                } else {
                    terms.push(format!("{}x", c));
                }
            } else {
                if (c - 1.0).abs() < 1e-10 {
                    terms.push(format!("x^{}", i));
                } else if (c - (-1.0)).abs() < 1e-10 {
                    terms.push(format!("-x^{}", i));
                } else {
                    terms.push(format!("{}x^{}", c, i));
                }
            }
        }
        if terms.is_empty() {
            write!(f, "0")
        } else {
            let s = terms.join(" + ").replace("+ -", "- ");
            write!(f, "{}", s)
        }
    }
}

pub fn horner(coeffs: &[f64], x: f64) -> f64 {
    let mut result = coeffs[coeffs.len() - 1];
    for i in (0..coeffs.len() - 1).rev() {
        result = result * x + coeffs[i];
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_create() {
        let p = Polynomial::new(vec![1.0, -5.0, 6.0]);
        assert_eq!(p.coeffs(), vec![1.0, -5.0, 6.0]);
    }

    #[test]
    fn test_polynomial_degree() {
        let p = Polynomial::new(vec![1.0, -5.0, 6.0]);
        assert_eq!(p.degree(), 2);
        assert_eq!(Polynomial::new(vec![5.0]).degree(), 0);
    }

    #[test]
    fn test_polynomial_eval() {
        let p = Polynomial::new(vec![6.0, -5.0, 1.0]);
        assert!((p.eval(1.0) - 2.0).abs() < 1e-10);
        assert!((p.eval(2.0) - 0.0).abs() < 1e-10);
        assert!((p.eval(3.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_polynomial_add() {
        let p1 = Polynomial::new(vec![1.0, 2.0]);
        let p2 = Polynomial::new(vec![1.0, -2.0, 1.0]);
        let result = p1.add(&p2);
        assert_eq!(result.coeffs(), vec![2.0, 0.0, 1.0]);
    }

    #[test]
    fn test_polynomial_sub() {
        let p1 = Polynomial::new(vec![3.0, 4.0]);
        let p2 = Polynomial::new(vec![1.0, 2.0]);
        let result = p1.sub(&p2);
        assert_eq!(result.coeffs(), vec![2.0, 2.0]);
    }

    #[test]
    fn test_polynomial_mul() {
        let p1 = Polynomial::new(vec![1.0, 1.0]);
        let p2 = Polynomial::new(vec![1.0, 1.0]);
        let result = p1.mul(&p2);
        assert_eq!(result.coeffs(), vec![1.0, 2.0, 1.0]);
    }

    #[test]
    fn test_polynomial_scalar_mul() {
        let p = Polynomial::new(vec![1.0, 2.0]);
        let result = p.scalar_mul(3.0);
        assert_eq!(result.coeffs(), vec![3.0, 6.0]);
    }

    #[test]
    fn test_polynomial_derivative() {
        let p = Polynomial::new(vec![1.0, 3.0, 3.0, 1.0]);
        let dp = p.derivative();
        assert_eq!(dp.coeffs(), vec![3.0, 6.0, 3.0]);
    }

    #[test]
    fn test_polynomial_integral() {
        let p = Polynomial::new(vec![2.0, 0.0]);
        let ip = p.integral(0.0);
        assert!((ip.coeffs()[0] - 0.0).abs() < 1e-10);
        assert!((ip.coeffs()[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_polynomial_divide() {
        let p1 = Polynomial::new(vec![1.0, 0.0, -1.0]);
        let p2 = Polynomial::new(vec![1.0, 1.0]);
        let (quotient, remainder) = p1.divide(&p2);
        assert_eq!(quotient.coeffs(), vec![1.0, -1.0]);
        assert_eq!(remainder.coeffs(), vec![0.0]);
    }

    #[test]
    fn test_horner() {
        let coeffs = vec![1.0, -5.0, 6.0];
        assert!((horner(&coeffs, 2.0) - 15.0).abs() < 1e-10);
        assert!((horner(&coeffs, 1.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_polynomial_to_string() {
        let p = Polynomial::new(vec![0.0, 1.0, 2.0]);
        let s = p.to_string();
        assert!(s.contains("x^2"));
    }
}