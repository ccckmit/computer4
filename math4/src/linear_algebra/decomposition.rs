use super::Matrix;

#[derive(Clone)]
pub struct LUResult {
    pub l: Matrix,
    pub u: Matrix,
    pub p: Matrix,
}

#[derive(Clone)]
pub struct QRResult {
    pub q: Matrix,
    pub r: Matrix,
}

#[derive(Clone)]
pub struct SVDResult {
    pub u: Matrix,
    pub s: Vec<f64>,
    pub v: Matrix,
}

pub fn lu(a: &Matrix) -> LUResult {
    let n = a.rows();
    let mut l = Matrix::identity(n);
    let mut u = a.clone();
    let mut p = Matrix::identity(n);
    for j in 0..n {
        let mut max_row = j;
        for i in (j + 1)..n {
            if u[[i, j]].abs() > u[[max_row, j]].abs() {
                max_row = i;
            }
        }
        if u[[max_row, j]].abs() < 1e-12 {
            continue;
        }
        if max_row != j {
            for k in 0..n {
                let tmp = u[[j, k]];
                u[[j, k]] = u[[max_row, k]];
                u[[max_row, k]] = tmp;
            }
            for k in 0..n {
                let tmp = p[[j, k]];
                p[[j, k]] = p[[max_row, k]];
                p[[max_row, k]] = tmp;
            }
        }
        for i in (j + 1)..n {
            if u[[j, j]].abs() < 1e-12 {
                continue;
            }
            let factor = u[[i, j]] / u[[j, j]];
            l[[i, j]] = factor;
            for k in j..n {
                u[[i, k]] -= factor * u[[j, k]];
            }
        }
    }
    LUResult { l, u, p }
}

pub fn qr(a: &Matrix) -> QRResult {
    let (m, n) = a.shape();
    let mut q = Matrix::zeros(m, n);
    let mut r = Matrix::zeros(n, n);
    for j in 0..n {
        let mut v: Vec<f64> = (0..m).map(|i| a[[i, j]]).collect();
        for k in 0..j {
            let dot: f64 = (0..m).map(|i| q[[i, k]] * v[i]).sum();
            r[[k, j]] = dot;
            for i in 0..m {
                v[i] -= dot * q[[i, k]];
            }
        }
        let norm = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
        r[[j, j]] = norm;
        if norm > 1e-12 {
            for i in 0..m {
                q[[i, j]] = v[i] / norm;
            }
        }
        for i in 0..j {
            r[[j, i]] = 0.0;
        }
    }
    QRResult { q, r }
}

pub fn svd(a: &Matrix) -> SVDResult {
    let (m, n) = a.shape();
    let ata = a.transpose().matmul(a);
    let eig = super::eig_sym(&ata);

    let mut svals: Vec<f64> = eig.values.iter().map(|&ev| ev.abs().sqrt()).collect();
    let mut pairs: Vec<(usize, f64)> = svals.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut perm = vec![0usize; svals.len()];
    let sorted_svals: Vec<f64> = pairs.iter().map(|&(_, v)| v).collect();
    for (new_idx, &(old_idx, _)) in pairs.iter().enumerate() {
        perm[old_idx] = new_idx;
    }

    let mut v_sorted = Matrix::zeros(n, n);
    for j in 0..n {
        let src_col = perm[j];
        for i in 0..n {
            v_sorted[[i, j]] = eig.vectors[[i, src_col]];
        }
    }

    let k = m.min(n);
    let mut u = Matrix::zeros(m, k);
    for j in 0..k {
        if sorted_svals[j] < 1e-10 {
            continue;
        }
        let sigma = sorted_svals[j];
        for i in 0..m {
            let dot: f64 = (0..n).map(|jj| a[[i, jj]] * v_sorted[[jj, j]]).sum();
            u[[i, j]] = dot / sigma;
        }
    }

    SVDResult {
        u,
        s: sorted_svals,
        v: v_sorted.transpose(),
    }
}

pub fn cholesky(a: &Matrix) -> Result<Matrix, &'static str> {
    let n = a.rows();
    if a.rows() != a.cols() {
        return Err("cholesky: not square");
    }
    let mut l = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..=i {
            let sum: f64 = (0..j).map(|k| l[[i, k]] * l[[j, k]]).sum();
            if i == j {
                let val = a[[i, i]] - sum;
                if val <= 0.0 {
                    return Err("cholesky: matrix not positive definite");
                }
                l[[i, j]] = val.sqrt();
            } else {
                l[[i, j]] = (a[[i, j]] - sum) / l[[j, j]];
            }
        }
    }
    Ok(l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lu() {
        let a = Matrix::new(vec![vec![4.0, 3.0], vec![6.0, 3.0]]);
        let lu = lu(&a);
        let pa = lu.p.matmul(&a);
        let recon = lu.l.matmul(&lu.u);
        for i in 0..2 {
            for j in 0..2 {
                assert!((recon[[i, j]] - pa[[i, j]]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_cholesky() {
        let a = Matrix::new(vec![vec![4.0, 2.0], vec![2.0, 3.0]]);
        let l = cholesky(&a).unwrap();
        let recon = l.matmul(&l.transpose());
        for i in 0..2 {
            for j in 0..2 {
                assert!((recon[[i, j]] - a[[i, j]]).abs() < 1e-8);
            }
        }
    }
}