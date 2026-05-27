use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use crate::tree::DecisionTree;

pub struct RandomForest {
    pub n_estimators: usize,
    pub max_depth: usize,
    pub min_samples_split: usize,
    pub trees: Vec<DecisionTree>,
    pub is_classification: bool,
    pub n_classes: usize,
}

impl RandomForest {
    pub fn new(n_estimators: usize, max_depth: usize, min_samples_split: usize) -> Self {
        RandomForest {
            n_estimators,
            max_depth,
            min_samples_split,
            trees: Vec::new(),
            is_classification: true,
            n_classes: 0,
        }
    }

    pub fn fit(&mut self, X: &[Vec<f64>], y: &[f64]) {
        let mut unique: Vec<f64> = y.to_vec();
        unique.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

        self.is_classification = unique.len() < y.len() / 2;
        self.n_classes = unique.len();
        self.trees = Vec::new();

        let n_samples = X.len();
        let mut rng = rand::rngs::StdRng::from_entropy();

        for _ in 0..self.n_estimators {
            let mut X_boot = Vec::with_capacity(n_samples);
            let mut y_boot = Vec::with_capacity(n_samples);

            for _ in 0..n_samples {
                let idx = rng.gen_range(0..n_samples);
                X_boot.push(X[idx].clone());
                y_boot.push(y[idx]);
            }

            let mut tree = DecisionTree::new(self.max_depth, self.min_samples_split);
            tree.fit(&X_boot, &y_boot);
            self.trees.push(tree);
        }
    }

    pub fn predict(&self, X: &[Vec<f64>]) -> Vec<f64> {
        let n_trees = self.trees.len();
        let n_samples = X.len();

        let all_preds: Vec<Vec<f64>> = self.trees.iter()
            .map(|tree| tree.predict(X))
            .collect();

        if self.is_classification {
            let mut result = Vec::with_capacity(n_samples);
            for j in 0..n_samples {
                let mut counts = HashMap::new();
                for preds in &all_preds {
                    *counts.entry(preds[j] as i32).or_insert(0) += 1;
                }
                let majority = counts.iter().max_by_key(|(_, &c)| c).map(|(&k, _)| k as f64).unwrap_or(0.0);
                result.push(majority);
            }
            result
        } else {
            let mut result = Vec::with_capacity(n_samples);
            for j in 0..n_samples {
                let sum: f64 = all_preds.iter().map(|preds| preds[j]).sum();
                result.push(sum / n_trees as f64);
            }
            result
        }
    }
}
