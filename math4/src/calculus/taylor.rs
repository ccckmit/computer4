fn factorial(n: usize) -> f64 {
    if n == 0 || n == 1 {
        return 1.0;
    }
    let mut result = 1.0;
    for i in 2..=n {
        result *= i as f64;
    }
    result
}

pub fn taylor<F>(f: F, a: f64, n: usize, h: f64) -> impl Fn(f64) -> f64
where
    F: Fn(f64) -> f64,
{
    let derivatives: Vec<f64> = (0..=n)
        .map(|order| nth_derivative(&f, order, a, h))
        .collect();

    move |x: f64| {
        let mut sum = 0.0;
        for i in 0..=n {
            let term = derivatives[i] / factorial(i) * (x - a).powi(i as i32);
            sum += term;
        }
        sum
    }
}

fn nth_derivative<F>(f: &F, order: usize, x: f64, h: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    if order == 0 {
        return f(x);
    }
    let eps = h;
    (nth_derivative(f, order - 1, x + eps, h) - nth_derivative(f, order - 1, x - eps, h)) / (2.0 * eps)
}

pub fn maclaurin<F>(f: F, n: usize, h: f64) -> impl Fn(f64) -> f64
where
    F: Fn(f64) -> f64,
{
    taylor(f, 0.0, n, h)
}

pub fn series_sum<F>(f: F, a: usize, n: usize) -> f64
where
    F: Fn(usize) -> f64,
{
    (a..=n).map(f).sum()
}

pub fn power_series_coeffs<F>(f: F, n: usize, center: f64, h: f64) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    (0..=n)
        .map(|i| nth_derivative(&f, i, center, h) / factorial(i))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_sum() {
        let f = |n: usize| n as f64;
        let result = series_sum(f, 1, 5);
        assert!((result - 15.0).abs() < 1e-10);
    }
}