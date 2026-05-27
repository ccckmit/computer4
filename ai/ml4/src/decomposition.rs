pub struct PCA {
    pub n_components: Option<usize>,
    pub components: Option<Vec<Vec<f64>>>,
    pub mean: Option<Vec<f64>>,
    pub explained_variance: Option<Vec<f64>>,
}

impl PCA {
    pub fn new(n_components: Option<usize>) -> Self {
        PCA {
            n_components,
            components: None,
            mean: None,
            explained_variance: None,
        }
    }

    pub fn fit(&mut self, X: &[Vec<f64>]) -> &mut Self {
        let n_samples = X.len();
        let n_features = X[0].len();

        self.mean = Some((0..n_features).map(|j| {
            X.iter().map(|row| row[j]).sum::<f64>() / n_samples as f64
        }).collect());

        let mean = self.mean.as_ref().unwrap();

        let mut components = Vec::new();
        let mut variances = Vec::new();

        let mut working_matrix: Vec<Vec<f64>> = X.iter().map(|row| {
            row.iter().zip(mean.iter()).map(|(a, b)| a - b).collect()
        }).collect();

        let n_comp = self.n_components.unwrap_or(n_features);

        for _ in 0..n_comp {
            let mut v: Vec<f64> = (0..n_features).map(|_| rand::random::<f64>()).collect();
            let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            v = v.iter().map(|x| x / norm).collect();

            for _ in 0..100 {
                let mut new_v = vec![0.0; n_features];
                for row in &working_matrix {
                    let dot: f64 = row.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                    for (j, &val) in row.iter().enumerate() {
                        new_v[j] += dot * val;
                    }
                }
                let norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
                if norm > 1e-10 {
                    v = new_v.iter().map(|x| x / norm).collect();
                }
            }

            let var: f64 = working_matrix.iter().map(|row| {
                let dot: f64 = row.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                dot * dot
            }).sum();

            components.push(v.clone());
            variances.push(var);

            for row in working_matrix.iter_mut() {
                let dot: f64 = row.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                for (j, &ev) in v.iter().enumerate() {
                    row[j] -= dot * ev;
                }
            }
        }

        self.components = Some(components);
        self.explained_variance = Some(variances);

        self
    }

    pub fn transform(&self, X: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let mean = self.mean.as_ref().expect("Model not fitted");
        let components = self.components.as_ref().expect("Model not fitted");

        X.iter().map(|row| {
            let centered: Vec<f64> = row.iter().zip(mean.iter()).map(|(a, b)| a - b).collect();
            components.iter().map(|comp| {
                centered.iter().zip(comp.iter()).map(|(a, b)| a * b).sum()
            }).collect()
        }).collect()
    }

    pub fn fit_transform(&mut self, X: &[Vec<f64>]) -> Vec<Vec<f64>> {
        self.fit(X);
        self.transform(X)
    }

    pub fn inverse_transform(&self, X_transformed: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let mean = self.mean.as_ref().expect("Model not fitted");
        let components = self.components.as_ref().expect("Model not fitted");

        X_transformed.iter().map(|row| {
            (0..mean.len()).map(|j| {
                let mut sum = mean[j];
                for (i, &v) in row.iter().enumerate() {
                    sum += v * components[i][j];
                }
                sum
            }).collect()
        }).collect()
    }
}
