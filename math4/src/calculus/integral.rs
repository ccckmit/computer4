pub fn trapezoid<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let h = (b - a) / n as f64;
    let mut sum = 0.5 * (f(a) + f(b));
    for i in 1..n {
        sum += f(a + i as f64 * h);
    }
    sum * h
}

pub fn simpson<F>(f: F, a: f64, b: f64, mut n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    if n % 2 != 0 {
        n += 1;
    }
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        sum += if i % 2 == 0 { 2.0 } else { 4.0 } * f(x);
    }
    (h / 3.0) * sum
}

pub fn romberg<F>(f: F, a: f64, b: f64, tol: f64, max_iter: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let mut r: Vec<Vec<f64>> = vec![vec![0.0; max_iter]; max_iter];

    r[0][0] = trapezoid(&f, a, b, 1);

    for i in 1..max_iter {
        let n = 2usize.pow(i as u32);
        r[i][0] = trapezoid(&f, a, b, n);
        for j in 1..=i {
            let k = 4.0_f64.powi(j as i32);
            r[i][j] = (k * r[i][j - 1] - r[i - 1][j - 1]) / (k - 1.0);
        }
        if i > 0 && (r[i][i] - r[i - 1][i - 1]).abs() < tol {
            return r[i][i];
        }
    }
    r[max_iter - 1][max_iter - 1]
}

pub fn adaptive<F>(f: F, a: f64, b: f64, tol: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    fn recursive<F>(f: &F, a: f64, b: f64, fa: f64, fb: f64, tol: f64) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let c = (a + b) / 2.0;
        let fc = f(c);
        let left = (b - a) / 6.0 * (fa + 4.0 * fc + fb);

        let d = (a + c) / 2.0;
        let fd = f(d);
        let e = (c + b) / 2.0;
        let fe = f(e);
        let right = (c - a) / 6.0 * (fa + 4.0 * fd + fc) + (b - c) / 6.0 * (fc + 4.0 * fe + fb);

        if (left - right).abs() < tol || b - a < 1e-14 {
            return right;
        }
        recursive(f, a, c, fa, fc, tol / 2.0) + recursive(f, c, b, fc, fb, tol / 2.0)
    }

    recursive(&f, a, b, f(a), f(b), tol)
}

pub fn monte_carlo<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    use crate::statistics::random::random;
    let mut sum = 0.0;
    for _ in 0..n {
        let x = a + random() * (b - a);
        sum += f(x);
    }
    sum * (b - a) / n as f64
}

pub fn gauss_legendre<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let (nodes, weights): (Vec<f64>, Vec<f64>) = match n {
        1 => (vec![0.0], vec![2.0]),
        2 => (
            vec![-0.577350269189626, 0.577350269189626],
            vec![1.0, 1.0],
        ),
        3 => (
            vec![0.0, -0.774596669240483, 0.774596669240483],
            vec![0.888888888888889, 0.555555555555556, 0.555555555555556],
        ),
        4 => (
            vec![
                -0.861136311594043,
                -0.339981043584856,
                0.339981043584856,
                0.861136311594043,
            ],
            vec![0.347854845137454, 0.652145154862546, 0.652145154862546, 0.347854845137454],
        ),
        5 => (
            vec![
                0.0,
                -0.538469310105683,
                0.538469310105683,
                -0.906179845938664,
                0.906179845938664,
            ],
            vec![
                0.568888888888889,
                0.478628670499366,
                0.478628670499366,
                0.236927758151136,
                0.236927758151136,
            ],
        ),
        _ => (vec![0.0], vec![2.0]),
    };

    let map = |t: f64| ((b + a) / 2.0) + ((b - a) / 2.0) * t;
    let mut sum = 0.0;
    for i in 0..n {
        sum += weights[i] * f(map(nodes[i]));
    }
    sum * (b - a) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trapezoid_x_squared() {
        let f = |x: f64| x * x;
        let result = trapezoid(f, 0.0, 1.0, 1000);
        assert!((result - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_trapezoid_sin() {
        let f = |x: f64| x.sin();
        let result = trapezoid(f, 0.0, std::f64::consts::PI, 1000);
        assert!((result - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_trapezoid_constant() {
        let f = |_: f64| 5.0;
        let result = trapezoid(f, 0.0, 10.0, 100);
        assert!((result - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_simpson_x_squared() {
        let f = |x: f64| x * x;
        let result = simpson(f, 0.0, 1.0, 100);
        assert!((result - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_simpson_x_cubed() {
        let f = |x: f64| x * x * x;
        let result = simpson(f, 0.0, 2.0, 100);
        assert!((result - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_romberg_x_squared() {
        let f = |x: f64| x * x;
        let result = romberg(f, 0.0, 1.0, 1e-10, 10);
        assert!((result - 1.0 / 3.0).abs() < 1e-8);
    }

    #[test]
    fn test_romberg_exp() {
        let f = |x: f64| x.exp();
        let result = romberg(f, 0.0, 1.0, 1e-10, 10);
        assert!((result - std::f64::consts::E + 1.0).abs() < 1e-8);
    }

    #[test]
    fn test_gauss_legendre_x_squared() {
        let f = |x: f64| x * x;
        let result = gauss_legendre(f, -1.0, 1.0, 3);
        assert!((result - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_gauss_legendre_sin() {
        let f = |x: f64| x.sin();
        let result = gauss_legendre(f, 0.0, std::f64::consts::PI, 5);
        assert!((result - 2.0).abs() < 0.001);
    }
}