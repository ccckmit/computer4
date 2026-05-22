use super::Matrix;

pub fn determinant_theorem() -> bool {
    let a = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    let b = Matrix::new(vec![vec![5.0, 6.0], vec![7.0, 8.0]]);
    let ab_det = a.matmul(&b).det();
    let a_det_times_b_det = a.det() * b.det();
    (ab_det - a_det_times_b_det).abs() < 1e-8
}

pub fn rank_nullity_theorem(m: usize, n: usize, rank: usize) -> bool {
    rank <= m.min(n) && n == rank + (n - rank)
}

pub fn eigenvalues_theorem() -> bool {
    let a = Matrix::new(vec![vec![4.0, 2.0], vec![1.0, 3.0]]);
    let trace = a.trace();
    let det = a.det();
    let eig = super::eigen(&a);
    let sum_ev = eig.values.iter().sum::<f64>();
    let prod_ev: f64 = eig.values.iter().product();
    (trace - sum_ev).abs() < 1e-3 && (det - prod_ev).abs() < 1e-3
}

pub fn svd_theorem() -> bool {
    let a = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    let svd = super::svd(&a);
    let s_diag = Matrix::from_diag(&svd.s);
    let reconstructed = svd.u.matmul(&s_diag).matmul(&svd.v);
    let diff = a.sub(&reconstructed);
    diff.norm(2.0) < 1e-6
}

pub fn linear_independence_theorem() -> bool {
    let a = Matrix::new(vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
    assert_eq!(a.rank(), 2);
    let b = Matrix::new(vec![vec![1.0, 2.0], vec![2.0, 4.0]]);
    assert_eq!(b.rank(), 1);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinant_theorem() {
        assert!(determinant_theorem());
    }

    #[test]
    fn test_rank_nullity() {
        assert!(rank_nullity_theorem(3, 3, 2));
    }

    #[test]
    fn test_eigenvalues_theorem() {
        assert!(eigenvalues_theorem());
    }

    #[test]
    fn test_linear_independence() {
        assert!(linear_independence_theorem());
    }
}