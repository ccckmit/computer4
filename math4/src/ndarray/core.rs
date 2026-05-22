use ndarray::{ArrayD, Array1, Array2, Axis, ShapeBuilder};

pub type Array = ArrayD<f64>;
pub type Matrix = Array2<f64>;
pub type Vector = Array1<f64>;

pub fn zeros(shape: &[usize]) -> Array {
    Array::zeros(shape)
}

pub fn ones(shape: &[usize]) -> Array {
    Array::from_shape_vec(shape.to_vec(), vec![1.0; shape.iter().product()]).unwrap()
}

pub fn full(shape: &[usize], fill: f64) -> Array {
    Array::from_shape_vec(shape.to_vec(), vec![fill; shape.iter().product()]).unwrap()
}

pub fn eye(n: usize) -> Matrix {
    Array2::eye(n)
}

pub fn identity(n: usize) -> Matrix {
    Array2::eye(n)
}

pub fn diag(v: &[f64], k: isize) -> Matrix {
    let n = (v.len() as isize + k.abs()) as usize;
    let mut a = Array2::zeros((n, n));
    for (i, &val) in v.iter().enumerate() {
        let row = if k >= 0 { i } else { i - k as usize };
        let col = if k >= 0 { i + k as usize } else { i };
        if row < n && col < n {
            a[[row, col]] = val;
        }
    }
    a
}

pub fn arange(start: f64, stop: f64, step: f64) -> Vector {
    let n = if (stop - start) / step >= 0.0 {
        ((stop - start) / step).ceil() as usize
    } else {
        0
    };
    let mut data = Vec::with_capacity(n);
    let mut cur = start;
    for _ in 0..n {
        data.push(cur);
        cur += step;
    }
    Array1::from_vec(data)
}

pub fn linspace(start: f64, stop: f64, num: usize) -> Vector {
    if num == 0 {
        return Array1::zeros(0);
    }
    if num == 1 {
        return Array1::from_vec(vec![start]);
    }
    let step = (stop - start) / (num - 1) as f64;
    Array1::from_vec((0..num).map(|i| start + i as f64 * step).collect())
}

pub fn array(data: &[f64], shape: &[usize]) -> Array {
    Array::from_shape_vec(shape.to_vec(), data.to_vec()).unwrap()
}

pub fn from_flat(data: &[f64], shape: &[usize]) -> Array {
    Array::from_shape_vec(shape.to_vec(), data.to_vec()).unwrap()
}

pub fn concatenate(arrays: &[Array], axis: Axis) -> Array {
    if arrays.is_empty() {
        panic!("concatenate: no arrays");
    }
    let views: Vec<_> = arrays.iter().map(|a| a.view()).collect();
    ndarray::concatenate(axis, &views).unwrap()
}

pub fn vstack(arrays: &[Array]) -> Array {
    concatenate(arrays, Axis(0))
}

pub fn hstack(arrays: &[Array]) -> Array {
    concatenate(arrays, Axis(1))
}

pub fn shape(a: &Array) -> Vec<usize> {
    a.shape().to_vec()
}

pub fn ndim(a: &Array) -> usize {
    a.ndim()
}

pub fn size(a: &Array) -> usize {
    a.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeros() {
        let a = zeros(&[2, 3]);
        assert_eq!(a.shape(), &[2, 3]);
        assert!(a.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_ones() {
        let a = ones(&[2, 3]);
        assert_eq!(a.shape(), &[2, 3]);
        assert!(a.iter().all(|&v| v == 1.0));
    }

    #[test]
    fn test_eye() {
        let a = eye(3);
        assert_eq!(a.shape(), &[3, 3]);
        for i in 0..3 {
            assert_eq!(a[[i, i]], 1.0);
        }
    }

    #[test]
    fn test_diag() {
        let a = diag(&[1.0, 2.0, 3.0], 0);
        assert_eq!(a.shape(), &[3, 3]);
        assert_eq!(a[[0, 0]], 1.0);
        assert_eq!(a[[1, 1]], 2.0);
        assert_eq!(a[[2, 2]], 3.0);
    }

    #[test]
    fn test_arange() {
        let a = arange(0.0, 5.0, 1.0);
        assert_eq!(a.len(), 5);
        assert_eq!(a[0], 0.0);
        assert_eq!(a[4], 4.0);
    }

    #[test]
    fn test_linspace() {
        let a = linspace(0.0, 1.0, 5);
        assert_eq!(a.len(), 5);
        assert_eq!(a[0], 0.0);
        assert_eq!(a[4], 1.0);
    }

    #[test]
    fn test_array() {
        let a = array(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        assert_eq!(a[[0, 0]], 1.0);
        assert_eq!(a[[1, 1]], 4.0);
    }

    #[test]
    fn test_vstack_hstack() {
        let a = array(&[1.0, 2.0], &[1, 2]);
        let b = array(&[3.0, 4.0], &[1, 2]);
        let v = vstack(&[a.clone(), b.clone()]);
        assert_eq!(v.shape(), &[2, 2]);
        let h = hstack(&[a, b]);
        assert_eq!(h.shape(), &[1, 4]);
    }
}