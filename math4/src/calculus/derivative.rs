fn normalize(v: &[f64]) -> Vec<f64> {
    let len = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
    if len == 0.0 {
        return v.to_vec();
    }
    v.iter().map(|&x| x / len).collect()
}

pub fn derivative<F>(f: F, h: f64) -> impl Fn(f64) -> f64
where
    F: Fn(f64) -> f64,
{
    move |x| (f(x + h) - f(x - h)) / (2.0 * h)
}

pub fn second_derivative<F>(f: F, h: f64) -> impl Fn(f64) -> f64
where
    F: Fn(f64) -> f64,
{
    move |x| (f(x + h) - 2.0 * f(x) + f(x - h)) / (h * h)
}

pub fn partial<F>(f: F, i: usize, h: f64) -> impl Fn(&[f64]) -> f64
where
    F: Fn(&[f64]) -> f64,
{
    move |x: &[f64]| {
        let mut x_plus = x.to_vec();
        let mut x_minus = x.to_vec();
        x_plus[i] += h;
        x_minus[i] -= h;
        (f(&x_plus) - f(&x_minus)) / (2.0 * h)
    }
}

pub fn gradient<F>(f: F, h: f64) -> impl Fn(&[f64]) -> Vec<f64>
where
    F: Fn(&[f64]) -> f64,
{
    move |x: &[f64]| {
        (0..x.len())
            .map(|i| {
                let mut x_plus = x.to_vec();
                let mut x_minus = x.to_vec();
                x_plus[i] += h;
                x_minus[i] -= h;
                (f(&x_plus) - f(&x_minus)) / (2.0 * h)
            })
            .collect()
    }
}

pub fn jacobian<F>(f: F, h: f64) -> impl Fn(&[f64]) -> Vec<Vec<f64>>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    move |x: &[f64]| {
        let fx = f(x);
        (0..fx.len())
            .map(|i| {
                let fi = |y: &[f64]| f(y)[i];
                (0..x.len())
                    .map(|j| {
                        let mut x_plus = x.to_vec();
                        let mut x_minus = x.to_vec();
                        x_plus[j] += h;
                        x_minus[j] -= h;
                        (fi(&x_plus) - fi(&x_minus)) / (2.0 * h)
                    })
                    .collect()
            })
            .collect()
    }
}

pub fn directional_derivative<F>(f: F, x: &[f64], direction: &[f64]) -> f64
where
    F: Fn(&[f64]) -> f64,
{
    let grad = gradient(&f, 1e-8)(x);
    let dir = normalize(direction);
    grad.iter().zip(dir.iter()).map(|(g, d)| g * d).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivative_x_squared() {
        let f = |x: f64| x * x;
        let df = derivative(f, 1e-8);
        assert!((df(2.0) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_derivative_sin() {
        let f = |x: f64| x.sin();
        let df = derivative(f, 1e-8);
        assert!((df(std::f64::consts::FRAC_PI_2) - 0.0).abs() < 0.001);
        assert!((df(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_second_derivative() {
        let f = |x: f64| x * x * x;
        let ddf = second_derivative(f, 1e-4);
        assert!((ddf(1.0) - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_gradient() {
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let grad = gradient(f, 1e-8);
        let result = grad(&[1.0, 2.0]);
        assert!((result[0] - 2.0).abs() < 0.001);
        assert!((result[1] - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_jacobian() {
        let f = |x: &[f64]| vec![x[0] + x[1], x[0] - x[1]];
        let jac = jacobian(f, 1e-8);
        let result = jac(&[1.0, 2.0]);
        assert!((result[0][0] - 1.0).abs() < 0.001);
        assert!((result[0][1] - 1.0).abs() < 0.001);
        assert!((result[1][0] - 1.0).abs() < 0.001);
        assert!((result[1][1] - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_directional_derivative() {
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let result = directional_derivative(f, &[1.0, 1.0], &[1.0, 0.0]);
        assert!((result - 2.0).abs() < 0.01);
    }
}