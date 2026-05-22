use super::{Matrix, LUResult};

pub fn solve(a: &Matrix, b: &[f64]) -> Result<Vec<f64>, &'static str> {
    let n = a.rows();
    if a.cols() != n {
        return Err("solve: matrix must be square");
    }
    if b.len() != n {
        return Err("solve: dimension mismatch");
    }
    let lu = crate::linear_algebra::decomposition::lu(a);
    solve_lu(&lu, b)
}

pub fn solve_lu(lu: &LUResult, b: &[f64]) -> Result<Vec<f64>, &'static str> {
    let n = lu.l.rows();
    let mut y = vec![0.0; n];
    for i in 0..n {
        let sum: f64 = (0..i).map(|k| lu.l.data[i][k] * y[k]).sum();
        y[i] = b[i] - sum;
    }
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        if lu.u.data[i][i].abs() < 1e-12 {
            return Err("solve_lu: zero pivot");
        }
        let sum: f64 = (i + 1..n).map(|k| lu.u.data[i][k] * x[k]).sum();
        x[i] = (y[i] - sum) / lu.u.data[i][i];
    }
    Ok(x)
}

pub struct LeastSquaresResult {
    pub x: Vec<f64>,
    pub residuals: Vec<f64>,
}

pub fn lstsq(a: &Matrix, b: &[f64]) -> Result<LeastSquaresResult, &'static str> {
    let (m, n) = a.shape();
    if b.len() != m {
        return Err("lstsq: dimension mismatch");
    }
    let at = a.transpose();
    let ata = at.matmul(a);
    let atb: Vec<f64> = (0..n).map(|j| {
        (0..m).map(|i| a.data[i][j] * b[i]).sum()
    }).collect();
    let ata_inv = match ata.inv() {
        Ok(inv) => inv,
        Err(_) => {
            let qr = crate::linear_algebra::decomposition::qr(a);
            let qtb: Vec<f64> = (0..n).map(|j| {
                (0..m).map(|i| qr.q.data[i][j] * b[i]).sum()
            }).collect();
            let mut x = vec![0.0; n];
            for j in (0..n).rev() {
                if qr.r.data[j][j].abs() < 1e-12 {
                    x[j] = 0.0;
                } else {
                    let sum: f64 = (j + 1..n).map(|k| qr.r.data[j][k] * x[k]).sum();
                    x[j] = (qtb[j] - sum) / qr.r.data[j][j];
                }
            }
            let residuals: Vec<f64> = (0..m).map(|i| {
                let pred: f64 = (0..n).map(|j| a.data[i][j] * x[j]).sum();
                b[i] - pred
            }).collect();
            return Ok(LeastSquaresResult { x, residuals });
        }
    };
    let x: Vec<f64> = (0..n).map(|j| {
        (0..n).map(|i| ata_inv.data[j][i] * atb[i]).sum()
    }).collect();
    let residuals: Vec<f64> = (0..m).map(|i| {
        let pred: f64 = (0..n).map(|j| a.data[i][j] * x[j]).sum();
        b[i] - pred
    }).collect();
    Ok(LeastSquaresResult { x, residuals })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve() {
        let a = Matrix::new(vec![vec![1.0, 1.0], vec![1.0, -1.0]]);
        let b = vec![3.0, 1.0];
        let x = solve(&a, &b).unwrap();
        assert!((x[0] - 2.0).abs() < 1e-8);
        assert!((x[1] - 1.0).abs() < 1e-8);
    }

    #[test]
    fn test_lstsq() {
        let a = Matrix::new(vec![vec![1.0, 1.0], vec![1.0, 2.0], vec![1.0, 3.0]]);
        let b = vec![1.0, 2.0, 3.0];
        let result = lstsq(&a, &b).unwrap();
        assert!(result.x.len() == 2);
    }
}