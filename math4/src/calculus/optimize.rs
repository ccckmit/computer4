pub fn golden_section<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let mut x1 = b - (b - a) / phi;
    let mut x2 = a + (b - a) / phi;
    let mut f1 = f(x1);
    let mut f2 = f(x2);

    for _ in 0..max_iter {
        if b - a < tol {
            return (a + b) / 2.0;
        }

        if f1 < f2 {
            b = x2;
            x2 = x1;
            f2 = f1;
            x1 = b - (b - a) / phi;
            f1 = f(x1);
        } else {
            a = x1;
            x1 = x2;
            f1 = f2;
            x2 = a + (b - a) / phi;
            f2 = f(x2);
        }
    }
    (a + b) / 2.0
}

pub fn gradient_descent<F, G>(
    f: F,
    df: G,
    x0: &[f64],
    lr: f64,
    tol: f64,
    max_iter: usize,
) -> OptimizeResult
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64]) -> Vec<f64>,
{
    let mut x = x0.to_vec();

    for i in 0..max_iter {
        let grad = df(&x);
        let grad_norm = grad.iter().map(|&g| g * g).sum::<f64>().sqrt();

        if grad_norm < tol {
            return OptimizeResult {
                x,
                iterations: i,
                converged: true,
            };
        }

        x = x
            .iter()
            .zip(grad.iter())
            .map(|(&xi, &gj)| xi - lr * gj)
            .collect();
    }

    OptimizeResult {
        x,
        iterations: max_iter,
        converged: false,
    }
}

#[derive(Debug, Clone)]
pub struct OptimizeResult {
    pub x: Vec<f64>,
    pub iterations: usize,
    pub converged: bool,
}

pub fn newton_method<F, DF, DDF>(
    f: F,
    df: DF,
    ddf: DDF,
    x0: f64,
    tol: f64,
    max_iter: usize,
) -> NewtonResult
where
    F: Fn(f64) -> f64,
    DF: Fn(f64) -> f64,
    DDF: Fn(f64) -> f64,
{
    let mut x = x0;

    for i in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);

        if dfx.abs() < 1e-14 {
            return NewtonResult {
                root: x,
                iterations: i,
                converged: false,
            };
        }

        let x1 = x - fx / dfx;
        if (x1 - x).abs() < tol {
            return NewtonResult {
                root: x1,
                iterations: i + 1,
                converged: true,
            };
        }
        x = x1;
    }

    NewtonResult {
        root: x,
        iterations: max_iter,
        converged: false,
    }
}

#[derive(Debug, Clone)]
pub struct NewtonResult {
    pub root: f64,
    pub iterations: usize,
    pub converged: bool,
}

pub fn conjugate_gradient(
    a: &[Vec<f64>],
    b: &[f64],
    x0: &[f64],
    tol: f64,
    max_iter: usize,
) -> OptimizeResult {
    let n = b.len();
    let mut x = x0.to_vec();
    let mut r: Vec<f64> = (0..n)
        .map(|i| b[i] - a[i].iter().zip(x.iter()).map(|(aj, xj)| aj * xj).sum::<f64>())
        .collect();
    let mut p = r.clone();
    let mut rsold = r.iter().map(|&ri| ri * ri).sum::<f64>();

    for i in 0..max_iter {
        let ap: Vec<f64> = (0..n)
            .map(|j| a[j].iter().zip(p.iter()).map(|(akj, pk)| akj * pk).sum::<f64>())
            .collect();
        let pap = p.iter().zip(ap.iter()).map(|(pj, apj)| pj * apj).sum::<f64>();

        if pap.abs() < 1e-14 {
            return OptimizeResult {
                x,
                iterations: i,
                converged: false,
            };
        }

        let alpha = rsold / pap;
        x = x
            .iter()
            .zip(p.iter())
            .map(|(&xj, &pj)| xj + alpha * pj)
            .collect();
        r = r
            .iter()
            .zip(ap.iter())
            .map(|(&ri, &api)| ri - alpha * api)
            .collect();

        let rsnew = r.iter().map(|&ri| ri * ri).sum::<f64>();
        if rsnew.sqrt() < tol {
            return OptimizeResult {
                x,
                iterations: i + 1,
                converged: true,
            };
        }

        let beta = rsnew / rsold;
        p = r
            .iter()
            .zip(p.iter())
            .map(|(&ri, &pj)| ri + beta * pj)
            .collect();
        rsold = rsnew;
    }

    OptimizeResult {
        x,
        iterations: max_iter,
        converged: false,
    }
}

pub fn momentum_gradient_descent<F, G>(
    f: F,
    df: G,
    x0: &[f64],
    lr: f64,
    momentum: f64,
    tol: f64,
    max_iter: usize,
) -> OptimizeResult
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64]) -> Vec<f64>,
{
    let mut x = x0.to_vec();
    let mut v = vec![0.0; x.len()];

    for i in 0..max_iter {
        let grad = df(&x);
        let grad_norm = grad.iter().map(|&g| g * g).sum::<f64>().sqrt();

        if grad_norm < tol {
            return OptimizeResult {
                x,
                iterations: i,
                converged: true,
            };
        }

        v = v.iter()
            .zip(grad.iter())
            .map(|(&vi, &gj)| momentum * vi + lr * gj)
            .collect();
        x = x.iter().zip(v.iter()).map(|(&xj, &vj)| xj - vj).collect();
    }

    OptimizeResult {
        x,
        iterations: max_iter,
        converged: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_section_x_squared() {
        let f = |x: f64| x * x;
        let result = golden_section(f, -10.0, 10.0, 1e-10, 100);
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_golden_section_x_minus_3() {
        let f = |x: f64| (x - 3.0) * (x - 3.0);
        let result = golden_section(f, 0.0, 10.0, 1e-10, 100);
        assert!((result - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_gradient_descent() {
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let df = |x: &[f64]| vec![2.0 * x[0], 2.0 * x[1]];
        let result = gradient_descent(f, df, &[5.0, 5.0], 0.1, 1e-10, 1000);
        assert!(result.converged);
        assert!(result.x[0].abs() < 0.1);
        assert!(result.x[1].abs() < 0.1);
    }

    #[test]
    fn test_newton_method() {
        let f = |x: f64| x * x - 2.0;
        let df = |x: f64| 2.0 * x;
        let ddf = |_: f64| 2.0;
        let result = newton_method(f, df, ddf, 1.0, 1e-10, 50);
        assert!(result.converged);
        assert!((result.root - 2.0_f64.sqrt()).abs() < 1e-5);
    }
}