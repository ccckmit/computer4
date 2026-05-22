pub fn bisection<F>(f: F, a: f64, b: f64, tol: f64, max_iter: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    if f(a) * f(b) > 0.0 {
        panic!("Function must have opposite signs at endpoints");
    }
    let mut lo = a;
    let mut hi = b;
    for _ in 0..max_iter {
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid);
        if f_mid.abs() < tol || (hi - lo) / 2.0 < tol {
            return mid;
        }
        if f_mid * f(lo) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

pub fn newton<F, DF>(f: F, df: DF, x0: f64, tol: f64, max_iter: usize) -> f64
where
    F: Fn(f64) -> f64,
    DF: Fn(f64) -> f64,
{
    let mut x = x0;
    for _ in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);
        if dfx.abs() < 1e-14 {
            panic!("Derivative too small");
        }
        let x1 = x - fx / dfx;
        if (x1 - x).abs() < tol {
            return x1;
        }
        x = x1;
    }
    x
}

pub fn secant<F>(f: F, x0: f64, x1: f64, tol: f64, max_iter: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let mut x_prev = x0;
    let mut x_curr = x1;
    for _ in 0..max_iter {
        let fx_prev = f(x_prev);
        let fx_curr = f(x_curr);
        if (fx_curr - fx_prev).abs() < 1e-14 {
            panic!("Function values too close");
        }
        let x_next = x_curr - fx_curr * (x_curr - x_prev) / (fx_curr - fx_prev);
        if (x_next - x_curr).abs() < tol {
            return x_next;
        }
        x_prev = x_curr;
        x_curr = x_next;
    }
    x_curr
}

pub fn horner(coeffs: &[f64], x: f64) -> f64 {
    let mut result = coeffs[0];
    for i in 1..coeffs.len() {
        result = result * x + coeffs[i];
    }
    result
}

pub fn horner_derivative(coeffs: &[f64], x: f64) -> f64 {
    let mut result = 0.0;
    let mut power = 1.0;
    for i in 1..coeffs.len() {
        result += i as f64 * coeffs[i] * power;
        power *= x;
    }
    result
}

pub fn find_all_roots<F>(f: F, coeffs: &[f64], tol: f64, max_iter: usize) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    let mut roots = Vec::new();
    let eps = 1e-6;
    let domains: [[f64; 2]; 3] = [[-100.0, -eps], [-eps, eps], [eps, 100.0]];

    for domain in &domains {
        let a = domain[0];
        let b = domain[1];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            bisection(&f, a, b, tol, max_iter)
        }));
        if let Ok(root) = result {
            if root.is_finite() {
                roots.push(root);
            }
        }
    }
    roots
}

pub fn deflation(coeffs: &[f64], root: f64) -> Vec<f64> {
    let n = coeffs.len() - 1;
    let mut reversed: Vec<f64> = coeffs.iter().rev().cloned().collect();
    let mut result = vec![0.0; n];
    result[0] = reversed[0];
    for i in 1..n {
        result[i] = reversed[i] + root * result[i - 1];
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisection_sqrt2() {
        let f = |x: f64| x * x - 2.0;
        let root = bisection(f, 0.0, 2.0, 1e-10, 100);
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_bisection_cubic() {
        let f = |x: f64| x * x * x - x - 2.0;
        let root = bisection(f, 1.0, 2.0, 1e-10, 100);
        assert!((root - 1.5213797).abs() < 1e-5);
    }

    #[test]
    fn test_newton_sqrt2() {
        let f = |x: f64| x * x - 2.0;
        let df = |x: f64| 2.0 * x;
        let root = newton(f, df, 1.5, 1e-10, 50);
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_newton_cube_root() {
        let f = |x: f64| x * x * x - 27.0;
        let df = |x: f64| 3.0 * x * x;
        let root = newton(f, df, 4.0, 1e-10, 50);
        assert!((root - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_secant_sqrt2() {
        let f = |x: f64| x * x - 2.0;
        let root = secant(f, 1.0, 2.0, 1e-10, 50);
        assert!((root - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_horner() {
        let coeffs = vec![1.0, -5.0, 6.0];
        assert!((horner(&coeffs, 2.0) - 0.0).abs() < 1e-10);
        assert!((horner(&coeffs, 3.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_horner_derivative() {
        let coeffs = vec![1.0, -5.0, 6.0];
        assert!((horner_derivative(&coeffs, 2.0) - 19.0).abs() < 1e-10);
    }

    #[test]
    fn test_deflation() {
        let coeffs = vec![6.0, -5.0, 1.0];
        let new_coeffs = deflation(&coeffs, 2.0);
        assert!((horner(&new_coeffs, 3.0) - 0.0).abs() < 1e-10);
    }
}