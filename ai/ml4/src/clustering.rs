use rand::{SeedableRng, seq::SliceRandom};

pub struct KMeans {
    pub n_clusters: usize,
    pub max_iter: usize,
    pub n_init: usize,
    pub centroids: Option<Vec<Vec<f64>>>,
    pub labels: Option<Vec<usize>>,
}

impl KMeans {
    pub fn new(n_clusters: usize, max_iter: usize, n_init: usize) -> Self {
        KMeans {
            n_clusters,
            max_iter,
            n_init,
            centroids: None,
            labels: None,
        }
    }

    pub fn fit(&mut self, X: &[Vec<f64>]) -> &mut Self {
        let mut best_inertia = f64::MAX;
        let mut best_centroids = None;
        let mut best_labels = None;

        let mut rng = rand::rngs::StdRng::from_entropy();

        for _ in 0..self.n_init {
            let n_samples = X.len();
            let mut indices: Vec<usize> = (0..n_samples).collect();
            indices.shuffle(&mut rng);
            let init_indices = &indices[..self.n_clusters];

            let mut centroids: Vec<Vec<f64>> = init_indices.iter()
                .map(|&i| X[i].clone())
                .collect();

            for _ in 0..self.max_iter {
                let labels = self.assign_labels(X, &centroids);
                let new_centroids = self.compute_centroids(X, &labels);

                let mut converged = true;
                for (c, nc) in centroids.iter().zip(new_centroids.iter()) {
                    for (a, b) in c.iter().zip(nc.iter()) {
                        if (a - b).abs() > 1e-9 {
                            converged = false;
                            break;
                        }
                    }
                }

                if converged {
                    break;
                }
                centroids = new_centroids;
            }

            let labels = self.assign_labels(X, &centroids);
            let inertia = self.compute_inertia(X, &labels, &centroids);

            if inertia < best_inertia {
                best_inertia = inertia;
                best_centroids = Some(centroids);
                best_labels = Some(labels);
            }
        }

        self.centroids = best_centroids;
        self.labels = best_labels;
        self
    }

    fn assign_labels(&self, X: &[Vec<f64>], centroids: &[Vec<f64>]) -> Vec<usize> {
        X.iter().map(|x| {
            centroids.iter()
                .map(|c| x.iter().zip(c).map(|(a, b)| (a - b).powi(2)).sum::<f64>())
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap()
        }).collect()
    }

    fn compute_centroids(&self, X: &[Vec<f64>], labels: &[usize]) -> Vec<Vec<f64>> {
        let n_features = X[0].len();
        let mut sums = vec![vec![0.0; n_features]; self.n_clusters];
        let mut counts = vec![0usize; self.n_clusters];

        for (x, &l) in X.iter().zip(labels.iter()) {
            for (i, &v) in x.iter().enumerate() {
                sums[l][i] += v;
            }
            counts[l] += 1;
        }

        sums.iter().enumerate().map(|(i, s)| {
            if counts[i] == 0 {
                s.clone()
            } else {
                s.iter().map(|&v| v / counts[i] as f64).collect()
            }
        }).collect()
    }

    fn compute_inertia(&self, X: &[Vec<f64>], labels: &[usize], centroids: &[Vec<f64>]) -> f64 {
        X.iter().zip(labels.iter())
            .map(|(x, &l)| {
                x.iter().zip(centroids[l].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
            })
            .sum()
    }

    pub fn predict(&self, X: &[Vec<f64>]) -> Vec<usize> {
        self.assign_labels(X, self.centroids.as_ref().unwrap())
    }
}
