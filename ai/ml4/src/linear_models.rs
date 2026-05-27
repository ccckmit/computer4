use std::f64::consts::E;

pub struct LinearRegression {
    pub lr: f64,
    pub n_iterations: usize,
    pub weights: Option<Vec<f64>>,
    pub bias: Option<f64>,
}

impl LinearRegression {
    pub fn new(lr: f64, n_iterations: usize) -> Self {
        LinearRegression { lr, n_iterations, weights: None, bias: None }
    }

    pub fn fit(&mut self, X: &[Vec<f64>], y: &[f64]) {
        let n_samples = X.len();
        let n_features = X[0].len();

        self.weights = Some(vec![0.0; n_features]);
        self.bias = Some(0.0);

        for _ in 0..self.n_iterations {
            let mut y_pred = vec![0.0; n_samples];

            for i in 0..n_samples {
                y_pred[i] = self.bias.unwrap();
                for j in 0..n_features {
                    y_pred[i] += X[i][j] * self.weights.as_ref().unwrap()[j];
                }
            }

            let mut dw = vec![0.0; n_features];
            let mut db = 0.0;

            for i in 0..n_samples {
                let error = y_pred[i] - y[i];
                for j in 0..n_features {
                    dw[j] += (1.0 / n_samples as f64) * X[i][j] * error;
                }
                db += (1.0 / n_samples as f64) * error;
            }

            let weights = self.weights.as_mut().unwrap();
            for j in 0..n_features {
                weights[j] -= self.lr * dw[j];
            }
            *self.bias.as_mut().unwrap() -= self.lr * db;
        }
    }

    pub fn predict(&self, X: &[Vec<f64>]) -> Vec<f64> {
        let weights = self.weights.as_ref().expect("Model not fitted");
        let bias = self.bias.expect("Model not fitted");

        X.iter().map(|row| {
            row.iter().zip(weights.iter()).map(|(x, w)| x * w).sum::<f64>() + bias
        }).collect()
    }
}

pub struct LogisticRegression {
    pub lr: f64,
    pub n_iterations: usize,
    pub weights: Option<Vec<f64>>,
    pub bias: Option<f64>,
}

impl LogisticRegression {
    pub fn new(lr: f64, n_iterations: usize) -> Self {
        LogisticRegression { lr, n_iterations, weights: None, bias: None }
    }

    fn sigmoid(&self, z: f64) -> f64 {
        let z = z.max(-500.0).min(500.0);
        1.0 / (1.0 + E.powf(-z))
    }

    pub fn fit(&mut self, X: &[Vec<f64>], y: &[f64]) {
        let n_samples = X.len();
        let n_features = X[0].len();

        self.weights = Some(vec![0.0; n_features]);
        self.bias = Some(0.0);

        for _ in 0..self.n_iterations {
            let mut y_pred = vec![0.0; n_samples];

            for i in 0..n_samples {
                let mut linear = self.bias.unwrap();
                for j in 0..n_features {
                    linear += X[i][j] * self.weights.as_ref().unwrap()[j];
                }
                y_pred[i] = self.sigmoid(linear);
            }

            let mut dw = vec![0.0; n_features];
            let mut db = 0.0;

            for i in 0..n_samples {
                let error = y_pred[i] - y[i];
                for j in 0..n_features {
                    dw[j] += (1.0 / n_samples as f64) * X[i][j] * error;
                }
                db += (1.0 / n_samples as f64) * error;
            }

            let weights = self.weights.as_mut().unwrap();
            for j in 0..n_features {
                weights[j] -= self.lr * dw[j];
            }
            *self.bias.as_mut().unwrap() -= self.lr * db;
        }
    }

    pub fn predict(&self, X: &[Vec<f64>]) -> Vec<i32> {
        self.predict_proba(X).iter().map(|&p| if p >= 0.5 { 1 } else { 0 }).collect()
    }

    pub fn predict_proba(&self, X: &[Vec<f64>]) -> Vec<f64> {
        let weights = self.weights.as_ref().expect("Model not fitted");
        let bias = self.bias.expect("Model not fitted");

        X.iter().map(|row| {
            let linear: f64 = row.iter().zip(weights.iter()).map(|(x, w)| x * w).sum::<f64>() + bias;
            self.sigmoid(linear)
        }).collect()
    }
}
