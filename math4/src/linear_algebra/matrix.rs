use std::fmt;

#[derive(Clone)]
pub struct Matrix {
    pub(crate) data: Vec<Vec<f64>>,
}

impl std::ops::Index<[usize; 2]> for Matrix {
    type Output = f64;
    fn index(&self, idx: [usize; 2]) -> &f64 {
        &self.data[idx[0]][idx[1]]
    }
}

impl std::ops::IndexMut<[usize; 2]> for Matrix {
    fn index_mut(&mut self, idx: [usize; 2]) -> &mut f64 {
        &mut self.data[idx[0]][idx[1]]
    }
}

#[derive(Clone)]
pub struct Vector {
    data: Vec<f64>,
}

impl Matrix {
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        Matrix { data }
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix {
            data: vec![vec![0.0; cols]; rows],
        }
    }

    pub fn ones(rows: usize, cols: usize) -> Self {
        Matrix {
            data: vec![vec![1.0; cols]; rows],
        }
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Matrix::zeros(n, n);
        for i in 0..n {
            m.data[i][i] = 1.0;
        }
        m
    }

    pub fn eye(n: usize) -> Self {
        Matrix::identity(n)
    }

    pub fn random(rows: usize, cols: usize) -> Self {
        use crate::statistics::random::random;
        Matrix {
            data: (0..rows)
                .map(|_| (0..cols).map(|_| random()).collect())
                .collect(),
        }
    }

    pub fn diag(v: &[f64]) -> Self {
        let n = v.len();
        let mut m = Matrix::zeros(n, n);
        for (i, &val) in v.iter().enumerate() {
            m.data[i][i] = val;
        }
        m
    }

    pub fn from_diag(v: &[f64]) -> Self {
        let n = v.len();
        let mut m = Matrix::zeros(n, n);
        for (i, &val) in v.iter().enumerate() {
            m.data[i][i] = val;
        }
        m
    }

    pub fn rows(&self) -> usize {
        self.data.len()
    }

    pub fn cols(&self) -> usize {
        self.data.get(0).map(|r| r.len()).unwrap_or(0)
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.rows(), self.cols())
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i][j]
    }

    pub fn transpose(&self) -> Self {
        let (r, c) = (self.rows(), self.cols());
        let mut t = Matrix::zeros(c, r);
        for i in 0..r {
            for j in 0..c {
                t.data[j][i] = self.data[i][j];
            }
        }
        t
    }

    pub fn add(&self, other: &Matrix) -> Matrix {
        let (r, c) = (self.rows(), self.cols());
        let mut result = Matrix::zeros(r, c);
        for i in 0..r {
            for j in 0..c {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }

    pub fn sub(&self, other: &Matrix) -> Matrix {
        let (r, c) = (self.rows(), self.cols());
        let mut result = Matrix::zeros(r, c);
        for i in 0..r {
            for j in 0..c {
                result.data[i][j] = self.data[i][j] - other.data[i][j];
            }
        }
        result
    }

    pub fn scale(&self, k: f64) -> Matrix {
        let mut result = self.clone();
        for row in &mut result.data {
            for val in row {
                *val *= k;
            }
        }
        result
    }

    pub fn matmul(&self, other: &Matrix) -> Matrix {
        let (r1, c1) = (self.rows(), self.cols());
        let (r2, c2) = (other.rows(), other.cols());
        if c1 != r2 {
            panic!("matmul: {}x{} * {}x{} dimension mismatch", r1, c1, r2, c2);
        }
        let mut result = Matrix::zeros(r1, c2);
        for i in 0..r1 {
            for j in 0..c2 {
                let mut sum = 0.0;
                for k in 0..c1 {
                    sum += self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        result
    }

    pub fn dot(&self, other: &Matrix) -> Matrix {
        self.matmul(other)
    }

    pub fn det(&self) -> f64 {
        let (r, c) = (self.rows(), self.cols());
        if r != c {
            panic!("det: not a square matrix");
        }
        match r {
            0 => 1.0,
            1 => self.data[0][0],
            2 => self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0],
            _ => {
                let mut lu = self.clone();
                let mut det_sign = 1.0;
                let n = r;
                for j in 0..n {
                    let mut max_row = j;
                    for i in (j + 1)..n {
                        if lu.data[i][j].abs() > lu.data[max_row][j].abs() {
                            max_row = i;
                        }
                    }
                    if (lu.data[max_row][j]).abs() < 1e-12 {
                        return 0.0;
                    }
                    if max_row != j {
                        lu.data.swap(max_row, j);
                        det_sign *= -1.0;
                    }
                    for i in (j + 1)..n {
                        let factor = lu.data[i][j] / lu.data[j][j];
                        for k in j..n {
                            lu.data[i][k] -= factor * lu.data[j][k];
                        }
                    }
                }
                let mut d = det_sign;
                for i in 0..n {
                    d *= lu.data[i][i];
                }
                d
            }
        }
    }

    pub fn inv(&self) -> Result<Matrix, &'static str> {
        let n = self.rows();
        if n != self.cols() {
            return Err("inv: not square");
        }
        if n == 2 {
            let d = self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0];
            if d.abs() < 1e-12 {
                return Err("inv: singular matrix");
            }
            let inv_d = 1.0 / d;
            return Ok(Matrix::new(vec![
                vec![self.data[1][1] * inv_d, -self.data[0][1] * inv_d],
                vec![-self.data[1][0] * inv_d, self.data[0][0] * inv_d],
            ]));
        }
        let mut aug = self.clone();
        let mut identity = Matrix::identity(n);
        for j in 0..n {
            let mut max_row = j;
            for i in (j + 1)..n {
                if aug.data[i][j].abs() > aug.data[max_row][j].abs() {
                    max_row = i;
                }
            }
            if aug.data[max_row][j].abs() < 1e-12 {
                return Err("inv: singular matrix");
            }
            if max_row != j {
                aug.data.swap(max_row, j);
                identity.data.swap(max_row, j);
            }
            let piv = aug.data[j][j];
            for k in 0..n {
                aug.data[j][k] /= piv;
                identity.data[j][k] /= piv;
            }
            for i in 0..n {
                if i != j {
                    let factor = aug.data[i][j];
                    for k in 0..n {
                        aug.data[i][k] -= factor * aug.data[j][k];
                        identity.data[i][k] -= factor * identity.data[j][k];
                    }
                }
            }
        }
        Ok(identity)
    }

    pub fn trace(&self) -> f64 {
        let n = self.rows().min(self.cols());
        (0..n).map(|i| self.data[i][i]).sum()
    }

    pub fn norm(&self, ord: f64) -> f64 {
        if ord == 2.0 || ord == 2.0_f64 {
            let mut sum_sq = 0.0;
            for row in &self.data {
                for &v in row {
                    sum_sq += v * v;
                }
            }
            sum_sq.sqrt()
        } else {
            let mut s = 0.0;
            for row in &self.data {
                for &v in row {
                    s += v.abs().powf(ord);
                }
            }
            s.powf(1.0 / ord)
        }
    }

    pub fn rank(&self) -> usize {
        let (m, n) = (self.rows(), self.cols());
        let mut b = self.clone();
        let mut r = 0;
        for j in 0..n {
            let mut max_row = r;
            for i in r..m {
                if b.data[i][j].abs() > b.data[max_row][j].abs() {
                    max_row = i;
                }
            }
            if b.data[max_row][j].abs() < 1e-10 {
                continue;
            }
            b.data.swap(r, max_row);
            let piv = b.data[r][j];
            for k in 0..n {
                b.data[r][k] /= piv;
            }
            for i in 0..m {
                if i != r {
                    let factor = b.data[i][j];
                    for k in 0..n {
                        b.data[i][k] -= factor * b.data[r][k];
                    }
                }
            }
            r += 1;
        }
        r
    }

    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        self.data.clone()
    }

    pub fn row(&self, i: usize) -> Vec<f64> {
        self.data[i].clone()
    }

    pub fn col(&self, j: usize) -> Vec<f64> {
        self.data.iter().map(|row| row[j]).collect()
    }
}

impl Vector {
    pub fn new(data: Vec<f64>) -> Self {
        Vector { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, i: usize) -> f64 {
        self.data[i]
    }

    pub fn to_vec(&self) -> Vec<f64> {
        self.data.clone()
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, row) in self.data.iter().enumerate() {
            for (j, val) in row.iter().enumerate() {
                if j > 0 { write!(f, "\t")?; }
                write!(f, "{:.4}", val)?;
            }
            if i + 1 < self.rows() { writeln!(f)?; }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let i = Matrix::identity(3);
        assert_eq!(i[[0, 0]], 1.0);
        assert_eq!(i[[1, 2]], 0.0);
    }

    #[test]
    fn test_matmul() {
        let a = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let b = Matrix::new(vec![vec![5.0, 6.0], vec![7.0, 8.0]]);
        let c = a.matmul(&b);
        assert_eq!(c[[0, 0]], 19.0);
        assert_eq!(c[[1, 1]], 50.0);
    }

    #[test]
    fn test_det_2x2() {
        let a = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert!((a.det() - (-2.0)).abs() < 1e-10);
    }

    #[test]
    fn test_inv_2x2() {
        let a = Matrix::new(vec![vec![4.0, 7.0], vec![2.0, 6.0]]);
        let inv = a.inv().unwrap();
        let id = a.matmul(&inv);
        assert!((id[[0, 0]] - 1.0).abs() < 1e-8);
        assert!((id[[1, 1]] - 1.0).abs() < 1e-8);
    }

    #[test]
    fn test_rank() {
        let a = Matrix::new(vec![vec![1.0, 2.0], vec![2.0, 4.0]]);
        assert_eq!(a.rank(), 1);
        let b = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(b.rank(), 2);
    }

    #[test]
    fn test_trace() {
        let a = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(a.trace(), 5.0);
    }
}