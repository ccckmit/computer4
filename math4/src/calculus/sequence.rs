pub fn sequence<F>(f: F, n0: usize, n: usize) -> Vec<f64>
where
    F: Fn(usize) -> f64,
{
    (n0..=n).map(f).collect()
}

pub fn series<F>(f: F, a: usize, b: usize) -> f64
where
    F: Fn(usize) -> f64,
{
    (a..=b).map(f).sum()
}

pub fn converge<F>(f: F, x0: f64, tol: f64, max_iter: usize) -> ConvergeResult
where
    F: Fn(f64) -> f64,
{
    let mut x = x0;
    for i in 0..max_iter {
        let x1 = f(x);
        if (x1 - x).abs() < tol {
            return ConvergeResult {
                root: x1,
                iterations: i + 1,
                converged: true,
            };
        }
        x = x1;
    }
    ConvergeResult {
        root: x,
        iterations: max_iter,
        converged: false,
    }
}

#[derive(Debug, Clone)]
pub struct ConvergeResult {
    pub root: f64,
    pub iterations: usize,
    pub converged: bool,
}

pub fn limit<F>(
    f: F,
    a: f64,
    direction: &str,
    tol: f64,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    let h = 1e-10;

    if direction == "left" || direction == "both" {
        let left = f(a - h);
        if direction == "left" {
            return Some(left);
        }
        let right = f(a + h);
        if (left - right).abs() < tol {
            return Some((left + right) / 2.0);
        }
        return Some(left);
    }

    if direction == "right" {
        return Some(f(a + h));
    }

    None
}

pub fn infinite_series<F>(f: F, tol: f64, max_terms: usize) -> SeriesResult
where
    F: Fn(usize) -> f64,
{
    let mut sum = 0.0;
    for n in 0..max_terms {
        let term = f(n);
        sum += term;
        if term.abs() < tol {
            return SeriesResult {
                sum,
                terms: n + 1,
                converged: true,
            };
        }
    }
    SeriesResult {
        sum,
        terms: max_terms,
        converged: false,
    }
}

pub fn alternating_series<F>(f: F, tol: f64, max_terms: usize) -> SeriesResult
where
    F: Fn(usize) -> f64,
{
    let mut sum = 0.0;
    let mut prev_sum = sum + 1.0;
    for n in 0..max_terms {
        let term = f(n) * if n % 2 == 0 { 1.0 } else { -1.0 };
        sum += term;
        if (sum - prev_sum).abs() < tol {
            return SeriesResult {
                sum,
                terms: n + 1,
                converged: true,
            };
        }
        prev_sum = sum;
    }
    SeriesResult {
        sum,
        terms: max_terms,
        converged: false,
    }
}

#[derive(Debug, Clone)]
pub struct SeriesResult {
    pub sum: f64,
    pub terms: usize,
    pub converged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence() {
        let f = |n: usize| n as f64 * 2.0;
        let result = sequence(f, 1, 5);
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0, 10.0]);
    }

    #[test]
    fn test_series() {
        let f = |n: usize| n as f64;
        let result = series(f, 1, 5);
        assert!((result - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_converge() {
        let f = |x: f64| x / 2.0;
        let result = converge(f, 1.0, 1e-10, 100);
        assert!(result.converged);
        assert!((result.root - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_limit() {
        let f = |x: f64| x * x;
        let result = limit(f, 2.0, "both", 1e-10);
        assert!(result.is_some());
        assert!((result.unwrap() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_infinite_series() {
        let f = |n: usize| 1.0 / (2.0_f64.powi(n as i32));
        let result = infinite_series(f, 1e-10, 100);
        assert!(result.converged);
        assert!((result.sum - 2.0).abs() < 0.001);
    }
}