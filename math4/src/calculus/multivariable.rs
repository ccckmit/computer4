pub fn grad<F>(f: F, h: f64) -> impl Fn(&[f64]) -> Vec<f64>
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

pub fn divergence<F>(f: F, h: f64) -> impl Fn(&[f64]) -> f64
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    move |x: &[f64]| {
        let mut sum = 0.0;
        for i in 0..x.len() {
            let fi = |y: &[f64]| f(y)[i];
            let mut x_plus = x.to_vec();
            let mut x_minus = x.to_vec();
            x_plus[i] += h;
            x_minus[i] -= h;
            sum += (fi(&x_plus) - fi(&x_minus)) / (2.0 * h);
        }
        sum
    }
}

pub fn curl_2d<F>(f: F) -> impl Fn(&[f64]) -> f64
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    move |x: &[f64]| {
        let eps = 1e-8;
        let d_f1dx2 = |x: &[f64]| {
            let mut x_plus = x.to_vec();
            x_plus[1] += eps;
            (f(&x_plus)[0] - f(x)[0]) / eps
        };
        let d_f2dx1 = |x: &[f64]| {
            let mut x_plus = x.to_vec();
            x_plus[0] += eps;
            (f(&x_plus)[1] - f(x)[1]) / eps
        };
        d_f1dx2(x) - d_f2dx1(x)
    }
}

pub fn curl_3d<F>(f: F) -> impl Fn(&[f64]) -> Vec<f64>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    move |x: &[f64]| {
        let eps = 1e-8;
        let fx = |i: usize, offset: &[f64]| {
            (f(offset)[i] - f(&[
                x[0] - if offset[0] > x[0] { 2.0 * eps } else { 0.0 },
                x[1],
                x[2],
            ])[i])
                / (2.0 * eps)
        };
        let fy = |i: usize, offset: &[f64]| {
            (f(offset)[i] - f(&[
                x[0],
                x[1] - if offset[1] > x[1] { 2.0 * eps } else { 0.0 },
                x[2],
            ])[i])
                / (2.0 * eps)
        };
        let fz = |i: usize, offset: &[f64]| {
            (f(offset)[i] - f(&[
                x[0],
                x[1],
                x[2] - if offset[2] > x[2] { 2.0 * eps } else { 0.0 },
            ])[i])
                / (2.0 * eps)
        };

        let x_plus_eps = |i| {
            let mut v = x.to_vec();
            v[i] += eps;
            v
        };

        let f_0 = f(x);
        vec![
            fy(2, &x_plus_eps(1)) - fz(1, &x_plus_eps(2)),
            fz(0, &x_plus_eps(2)) - fx(2, &x_plus_eps(0)),
            fx(1, &x_plus_eps(0)) - fy(0, &x_plus_eps(1)),
        ]
    }
}

pub fn laplacian<F>(f: F, h: f64) -> impl Fn(&[f64]) -> f64
where
    F: Fn(&[f64]) -> f64,
{
    move |x: &[f64]| {
        let mut sum = 0.0;
        let f0 = f(x);
        for i in 0..x.len() {
            let mut x_plus = x.to_vec();
            let mut x_minus = x.to_vec();
            x_plus[i] += h;
            x_minus[i] -= h;
            sum += (f(&x_plus) - 2.0 * f0 + f(&x_minus)) / (h * h);
        }
        sum
    }
}

pub fn hessian<F>(f: F, h: f64) -> impl Fn(&[f64]) -> Vec<Vec<f64>>
where
    F: Fn(&[f64]) -> f64,
{
    move |x: &[f64]| {
        let n = x.len();
        let mut result = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    let mut x_plus = x.to_vec();
                    let mut x_minus = x.to_vec();
                    x_plus[i] += h;
                    x_minus[i] -= h;
                    result[i][j] = (f(&x_plus) - 2.0 * f(x) + f(&x_minus)) / (h * h);
                } else {
                    let mut xpp = x.to_vec();
                    let mut xpm = x.to_vec();
                    let mut xmp = x.to_vec();
                    let mut xmm = x.to_vec();
                    xpp[i] += h;
                    xpp[j] += h;
                    xpm[i] += h;
                    xpm[j] -= h;
                    xmp[i] -= h;
                    xmp[j] += h;
                    xmm[i] -= h;
                    xmm[j] -= h;
                    result[i][j] = (f(&xpp) - f(&xpm) - f(&xmp) + f(&xmm)) / (4.0 * h * h);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grad() {
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let grad_fn = grad(f, 1e-8);
        let result = grad_fn(&[1.0, 2.0]);
        assert!((result[0] - 2.0).abs() < 0.001);
        assert!((result[1] - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_laplacian() {
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let lap = laplacian(f, 1e-4);
        let result = lap(&[1.0, 2.0]);
        assert!((result - 4.0).abs() < 0.1);
    }
}