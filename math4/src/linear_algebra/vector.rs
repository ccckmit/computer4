pub fn norm_vector(v: &[f64]) -> f64 {
    v.iter().map(|&x| x * x).sum::<f64>().sqrt()
}

pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub fn cross_product(a: &[f64], b: &[f64]) -> Vec<f64> {
    if a.len() != 3 || b.len() != 3 {
        panic!("cross_product requires 3D vectors");
    }
    vec![
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn normalize(v: &[f64]) -> Vec<f64> {
    let n = norm_vector(v);
    if n < 1e-12 {
        return v.to_vec();
    }
    v.iter().map(|&x| x / n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((dot_product(&a, &b)).abs() < 1e-10);
    }

    #[test]
    fn test_cross() {
        let i = vec![1.0, 0.0, 0.0];
        let j = vec![0.0, 1.0, 0.0];
        let k = cross_product(&i, &j);
        assert!((k[0] - 0.0).abs() < 1e-10);
        assert!((k[1] - 0.0).abs() < 1e-10);
        assert!((k[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_norm() {
        let v = vec![3.0, 4.0];
        assert!((norm_vector(&v) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize() {
        let v = vec![3.0, 4.0];
        let n = normalize(&v);
        assert!((norm_vector(&n) - 1.0).abs() < 1e-10);
    }
}