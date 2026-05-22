use super::Matrix;

#[derive(Clone)]
pub struct EigResult {
    pub values: Vec<f64>,
    pub vectors: Matrix,
}

pub fn eigenvalues(a: &Matrix) -> Vec<f64> {
    super::eigen(a).values
}

pub fn eigen(a: &Matrix) -> EigResult {
    let n = a.rows();
    if n != a.cols() {
        panic!("eigen: matrix must be square");
    }
    if n == 1 {
        return EigResult { values: vec![a.data[0][0]], vectors: Matrix::identity(1) };
    }
    let mut a_mut = a.clone();
    let mut eigenvectors = Matrix::identity(n);
    for _iter in 0..100 {
        let qr = crate::linear_algebra::decomposition::qr(&a_mut);
        a_mut = qr.r.matmul(&qr.q);
        eigenvectors = eigenvectors.matmul(&qr.q);
    }
    let mut vals = Vec::new();
    for i in 0..n {
        vals.push(a_mut.data[i][i]);
    }
    EigResult { values: vals, vectors: eigenvectors }
}

pub fn eig_sym(a: &Matrix) -> EigResult {
    let n = a.rows();
    if n != a.cols() {
        panic!("eig_sym: matrix must be square");
    }
    let mut a_mut = a.clone();
    let mut eigenvectors = Matrix::identity(n);
    for _iter in 0..200 {
        let qr = crate::linear_algebra::decomposition::qr(&a_mut);
        a_mut = qr.r.matmul(&qr.q);
        eigenvectors = eigenvectors.matmul(&qr.q);
    }
    let mut vals = Vec::new();
    for i in 0..n {
        vals.push(a_mut.data[i][i]);
    }
    let mut pairs: Vec<(usize, f64)> = vals.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut sorted_vals = Vec::new();
    let mut perm = vec![0usize; n];
    for (i, &(idx, val)) in pairs.iter().enumerate() {
        sorted_vals.push(val);
        perm[idx] = i;
    }
    let mut sorted_vecs = Matrix::zeros(n, n);
    for j in 0..n {
        for i in 0..n {
            sorted_vecs.data[i][perm[j]] = eigenvectors.data[i][j];
        }
    }
    EigResult { values: sorted_vals, vectors: sorted_vecs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eigen_2x2() {
        let a = Matrix::new(vec![vec![4.0, 2.0], vec![1.0, 3.0]]);
        let eig = eigen(&a);
        assert_eq!(eig.values.len(), 2);
    }

    #[test]
    fn test_eig_sym() {
        let a = Matrix::new(vec![vec![2.0, 1.0], vec![1.0, 2.0]]);
        let eig = eig_sym(&a);
        assert!((eig.values[0] - 3.0).abs() < 1e-2 || (eig.values[0] - 1.0).abs() < 1e-2);
    }
}