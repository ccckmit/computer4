use ndarray::{Axis, Zip, ArrayView2, Array1};
use std::cmp::Ordering;
use super::core::Array;

pub fn add(a: Array, b: Array) -> Array {
    &a + &b
}

pub fn add_scalar(a: Array, b: f64) -> Array {
    a.mapv(|x| x + b)
}

pub fn sub(a: Array, b: Array) -> Array {
    &a - &b
}

pub fn sub_scalar(a: Array, b: f64) -> Array {
    a.mapv(|x| x - b)
}

pub fn mul(a: Array, b: Array) -> Array {
    &a * &b
}

pub fn mul_scalar(a: Array, b: f64) -> Array {
    a.mapv(|x| x * b)
}

pub fn div(a: Array, b: Array) -> Array {
    &a / &b
}

pub fn div_scalar(a: Array, b: f64) -> Array {
    a.mapv(|x| x / b)
}

pub fn matmul(a: Array, b: Array) -> Array {
    if a.ndim() != 2 || b.ndim() != 2 {
        panic!("matmul requires 2D arrays");
    }
    let a2: ArrayView2<f64> = a.view().into_dimensionality().unwrap();
    let b2: ArrayView2<f64> = b.view().into_dimensionality().unwrap();
    a2.dot(&b2).into_dyn()
}

pub fn dot(a: Array, b: Array) -> f64 {
    if a.ndim() == 1 && b.ndim() == 1 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    } else {
        let result = matmul(a, b);
        result.iter().cloned().sum()
    }
}

pub fn abs(a: Array) -> Array {
    a.mapv(|x| x.abs())
}

pub fn sqrt(a: Array) -> Array {
    a.mapv(|x| x.sqrt())
}

pub fn exp(a: Array) -> Array {
    a.mapv(|x| x.exp())
}

pub fn log(a: Array) -> Array {
    a.mapv(|x| x.ln())
}

pub fn sin(a: Array) -> Array {
    a.mapv(|x| x.sin())
}

pub fn cos(a: Array) -> Array {
    a.mapv(|x| x.cos())
}

pub fn tan(a: Array) -> Array {
    a.mapv(|x| x.tan())
}

fn scalar_arr(v: f64) -> Array {
    Array1::from_vec(vec![v]).into_dyn()
}

pub fn sum(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    match axis {
        None => Ok(scalar_arr(a.iter().sum())),
        Some(ax) => Ok(a.sum_axis(Axis(ax))),
    }
}

pub fn mean(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    match axis {
        None => Ok(scalar_arr(a.iter().sum::<f64>() / a.len() as f64)),
        Some(ax) => {
            let s = a.sum_axis(Axis(ax));
            let dim = a.shape()[ax] as f64;
            Ok(s.mapv(|x| x / dim))
        }
    }
}

pub fn std(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    match axis {
        None => {
            let m = a.iter().sum::<f64>() / a.len() as f64;
            let v = a.iter().map(|x| (x - m).powi(2)).sum::<f64>() / a.len() as f64;
            Ok(scalar_arr(v.sqrt()))
        }
        Some(ax) => {
            let m_arr = mean(a, Some(ax))?;
            let m_val = m_arr[0];
            let dim = a.shape()[ax] as f64;
            let mut result = a.mapv(|x| (x - m_val).powi(2));
            result = result.sum_axis(Axis(ax));
            result.mapv_inplace(|x| (x / dim).sqrt());
            Ok(result)
        }
    }
}

pub fn var_(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    let s = std(a, axis)?;
    Ok(s.mapv(|x| x * x))
}

pub fn min(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    match axis {
        None => Ok(scalar_arr(a.iter().cloned().fold(f64::INFINITY, f64::min))),
        Some(ax) => Ok(a.map_axis(Axis(ax), |view| view.iter().cloned().fold(f64::INFINITY, f64::min))),
    }
}

pub fn max(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    match axis {
        None => Ok(scalar_arr(a.iter().cloned().fold(f64::NEG_INFINITY, f64::max))),
        Some(ax) => Ok(a.map_axis(Axis(ax), |view| view.iter().cloned().fold(f64::NEG_INFINITY, f64::max))),
    }
}

pub fn argmin(a: &Array) -> usize {
    a.iter()
        .enumerate()
        .min_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn argmax(a: &Array) -> usize {
    a.iter()
        .enumerate()
        .max_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

pub fn prod(a: &Array, axis: Option<usize>) -> Result<Array, f64> {
    match axis {
        None => Ok(scalar_arr(a.iter().product())),
        Some(ax) => Ok(a.map_axis(Axis(ax), |view| view.iter().product())),
    }
}

fn cmp<F>(a: &Array, b: f64, op: F) -> Array
where
    F: Fn(f64, f64) -> bool,
{
    let mut result = a.mapv(|_| 0.0);
    Zip::from(&mut result).and(a).for_each(|r, &a_val| {
        *r = if op(a_val, b) { 1.0 } else { 0.0 };
    });
    result
}

pub fn gt(a: &Array, b: f64) -> Array { cmp(a, b, |x, y| x > y) }
pub fn gte(a: &Array, b: f64) -> Array { cmp(a, b, |x, y| x >= y) }
pub fn lt(a: &Array, b: f64) -> Array { cmp(a, b, |x, y| x < y) }
pub fn lte(a: &Array, b: f64) -> Array { cmp(a, b, |x, y| x <= y) }
pub fn eq(a: &Array, b: f64) -> Array { cmp(a, b, |x, y| (x - y).abs() < 1e-10) }
pub fn ne(a: &Array, b: f64) -> Array { cmp(a, b, |x, y| (x - y).abs() >= 1e-10) }

pub fn where_(cond: &Array, x: &Array, y: &Array) -> Array {
    let mut result = x.mapv(|_| 0.0);
    Zip::from(&mut result).and(cond).and(x).and(y).for_each(|r, &c, &xv, &yv| {
        *r = if c != 0.0 { xv } else { yv };
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a = crate::ndarray::array(&[1.0, 2.0, 3.0], &[3]);
        let b = crate::ndarray::array(&[4.0, 5.0, 6.0], &[3]);
        let c = add(a, b);
        assert_eq!(c[0], 5.0);
        assert_eq!(c[2], 9.0);
    }

    #[test]
    fn test_matmul() {
        let a = crate::ndarray::array(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        let b = crate::ndarray::array(&[5.0, 6.0, 7.0, 8.0], &[2, 2]);
        let c = matmul(a, b);
        assert_eq!(c[[0, 0]], 19.0);
        assert_eq!(c[[1, 1]], 50.0);
    }

    #[test]
    fn test_sum() {
        let a = crate::ndarray::array(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        assert_eq!(sum(&a, None).unwrap()[0], 10.0);
        let col = sum(&a, Some(0)).unwrap();
        assert_eq!(col[[0]], 4.0);
        assert_eq!(col[[1]], 6.0);
    }

    #[test]
    fn test_mean() {
        let a = crate::ndarray::array(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        assert!((mean(&a, None).unwrap()[0] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_abs() {
        let a = crate::ndarray::array(&[-1.0, 2.0, -3.0], &[3]);
        let b = abs(a);
        assert_eq!(b[0], 1.0);
        assert_eq!(b[2], 3.0);
    }

    #[test]
    fn test_comparisons() {
        let a = crate::ndarray::array(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        let gt2 = gt(&a, 2.0);
        assert_eq!(gt2[[0, 0]], 0.0);
        assert_eq!(gt2[[1, 0]], 1.0);
    }

    #[test]
    fn test_where() {
        let cond = crate::ndarray::array(&[1.0, 0.0, 1.0, 0.0], &[2, 2]);
        let x = crate::ndarray::array(&[1.0, 2.0, 3.0, 4.0], &[2, 2]);
        let y = crate::ndarray::array(&[10.0, 20.0, 30.0, 40.0], &[2, 2]);
        let r = where_(&cond, &x, &y);
        assert_eq!(r[[0, 0]], 1.0);
        assert_eq!(r[[0, 1]], 20.0);
    }
}