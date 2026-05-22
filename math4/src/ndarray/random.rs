use crate::statistics::random::{random, rand_int};
use ndarray::Array2;
use super::core::Array;

pub fn rand(shape: &[usize]) -> Array {
    let size: usize = shape.iter().product();
    let data: Vec<f64> = (0..size).map(|_| random()).collect();
    Array::from_shape_vec(shape.to_vec(), data).unwrap()
}

pub fn randn(shape: &[usize]) -> Array {
    let size: usize = shape.iter().product();
    let data: Vec<f64> = (0..size).map(|_| {
        let u1 = random();
        let u2 = random();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        z
    }).collect();
    Array::from_shape_vec(shape.to_vec(), data).unwrap()
}

pub fn randint(low: usize, high: usize, shape: &[usize]) -> Array {
    let size: usize = shape.iter().product();
    let data: Vec<f64> = (0..size).map(|_| rand_int(low as i64, high as i64) as f64).collect();
    Array::from_shape_vec(shape.to_vec(), data).unwrap()
}

pub fn choice(a: &Array, n: usize, replace: bool) -> Array {
    let size = a.len();
    if n == 0 {
        return Array::from_shape_vec(a.shape().to_vec(), vec![]).unwrap();
    }
    if !replace {
        let mut indices: Vec<usize> = (0..size).collect();
        for i in (1..indices.len()).rev() {
            let j = rand_int(0, i as i64) as usize;
            indices.swap(i, j);
        }
        let chosen: Vec<f64> = indices[..n].iter().map(|&i| a[i]).collect();
        Array::from_shape_vec(vec![n], chosen).unwrap()
    } else {
        let chosen: Vec<f64> = (0..n).map(|_| {
            let idx = rand_int(0, size as i64 - 1) as usize;
            a[idx]
        }).collect();
        Array::from_shape_vec(vec![n], chosen).unwrap()
    }
}

pub fn shuffle(a: &mut Array) {
    let size = a.len();
    if size <= 1 { return; }
    let shape = a.shape().to_vec();
    let mut data: Vec<f64> = a.iter().cloned().collect();
    let mut indices: Vec<usize> = (0..size).collect();
    for i in (1..size).rev() {
        let j = rand_int(0, i as i64) as usize;
        indices.swap(i, j);
    }
    let mut tmp = data.clone();
    for (new_idx, &old_idx) in indices.iter().enumerate() {
        tmp[new_idx] = data[old_idx];
    }
    *a = Array::from_shape_vec(shape, tmp).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand() {
        let a = rand(&[3, 4]);
        assert_eq!(a.shape(), &[3, 4]);
        assert!(a.iter().all(|&v| v >= 0.0 && v < 1.0));
    }

    #[test]
    fn test_randn() {
        let a = randn(&[100]);
        let mean: f64 = a.iter().sum::<f64>() / 100.0;
        assert!(mean.abs() < 0.5);
    }

    #[test]
    fn test_randint() {
        let a = randint(1, 6, &[10]);
        assert!(a.iter().all(|&v| v >= 1.0 && v <= 6.0));
    }

    #[test]
    fn test_choice_no_replace() {
        let a = Array::from_shape_vec(vec![5], vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let c = choice(&a, 3, false);
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn test_choice_replace() {
        let a = Array::from_shape_vec(vec![3], vec![1.0, 2.0, 3.0]).unwrap();
        let c = choice(&a, 5, true);
        assert_eq!(c.len(), 5);
    }

    #[test]
    fn test_shuffle() {
        let mut a = Array::from_shape_vec(vec![5], vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        shuffle(&mut a);
        let sum: f64 = a.iter().sum();
        assert!((sum - 15.0).abs() < 1e-10);
    }
}