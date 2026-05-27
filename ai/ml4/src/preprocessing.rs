use rand::{SeedableRng, seq::SliceRandom};

pub struct StandardScaler {
    pub mean: Option<Vec<f64>>,
    pub std: Option<Vec<f64>>,
}

impl StandardScaler {
    pub fn new() -> Self {
        StandardScaler { mean: None, std: None }
    }

    pub fn fit(&mut self, X: &[Vec<f64>]) -> &mut Self {
        let n_samples = X.len();
        let n_features = X[0].len();

        self.mean = Some((0..n_features).map(|j| {
            X.iter().map(|row| row[j]).sum::<f64>() / n_samples as f64
        }).collect());

        self.std = Some((0..n_features).map(|j| {
            let mean = self.mean.as_ref().unwrap()[j];
            let variance = X.iter()
                .map(|row| (row[j] - mean).powi(2))
                .sum::<f64>() / n_samples as f64;
            variance.sqrt().max(1e-10)
        }).collect());

        self
    }

    pub fn transform(&self, X: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let mean = self.mean.as_ref().expect("Scaler not fitted");
        let std = self.std.as_ref().expect("Scaler not fitted");

        X.iter().map(|row| {
            row.iter().enumerate().map(|(i, &v)| (v - mean[i]) / std[i]).collect()
        }).collect()
    }

    pub fn fit_transform(&mut self, X: &[Vec<f64>]) -> Vec<Vec<f64>> {
        self.fit(X);
        self.transform(X)
    }
}

impl Default for StandardScaler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn train_test_split(
    X: &[Vec<f64>],
    y: &[f64],
    test_size: f64,
    random_state: Option<u64>,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let mut rng = match random_state {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    let n_samples = X.len();
    let n_test = (n_samples as f64 * test_size) as usize;

    let mut indices: Vec<usize> = (0..n_samples).collect();
    indices.shuffle(&mut rng);

    let test_idx: Vec<usize> = indices.iter().take(n_test).copied().collect();
    let train_idx: Vec<usize> = indices.iter().skip(n_test).copied().collect();

    let X_train: Vec<Vec<f64>> = train_idx.iter().map(|&i| X[i].clone()).collect();
    let X_test: Vec<Vec<f64>> = test_idx.iter().map(|&i| X[i].clone()).collect();
    let y_train: Vec<f64> = train_idx.iter().map(|&i| y[i]).collect();
    let y_test: Vec<f64> = test_idx.iter().map(|&i| y[i]).collect();

    (X_train, X_test, y_train, y_test)
}
