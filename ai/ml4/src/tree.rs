use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TreeNode {
    Leaf { value: f64 },
    Branch {
        feat_idx: usize,
        threshold: f64,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
}

pub struct DecisionTree {
    pub max_depth: usize,
    pub min_samples_split: usize,
    pub tree: Option<TreeNode>,
    pub is_classification: bool,
    pub n_classes: usize,
}

impl DecisionTree {
    pub fn new(max_depth: usize, min_samples_split: usize) -> Self {
        DecisionTree {
            max_depth,
            min_samples_split,
            tree: None,
            is_classification: false,
            n_classes: 0,
        }
    }

    pub fn fit(&mut self, X: &[Vec<f64>], y: &[f64]) {
        let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
        let mut unique: Vec<i32> = y_int.clone();
        unique.sort();
        unique.dedup();

        self.is_classification = unique.len() < y.len();
        self.n_classes = unique.len();

        self.tree = Some(self.build_tree(X, &y_int, 0));
    }

    fn entropy(&self, y: &[i32]) -> f64 {
        if y.is_empty() {
            return 0.0;
        }

        let mut counts = HashMap::new();
        for &v in y {
            *counts.entry(v).or_insert(0) += 1;
        }

        let n = y.len() as f64;
        let mut entropy = 0.0;
        for count in counts.values() {
            let p = *count as f64 / n;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    fn information_gain(&self, y: &[i32], left_mask: &[bool], right_mask: &[bool]) -> f64 {
        let parent_entropy = self.entropy(y);

        let n = y.len() as f64;
        let left: Vec<i32> = y.iter().enumerate().filter(|(i, _)| left_mask[*i]).map(|(_, &v)| v).collect();
        let right: Vec<i32> = y.iter().enumerate().filter(|(i, _)| right_mask[*i]).map(|(_, &v)| v).collect();

        let n_l = left.len() as f64;
        let n_r = right.len() as f64;

        if n_l == 0.0 || n_r == 0.0 {
            return 0.0;
        }

        let e_l = self.entropy(&left);
        let e_r = self.entropy(&right);

        parent_entropy - (n_l / n) * e_l - (n_r / n) * e_r
    }

    fn best_split(&self, X: &[Vec<f64>], y: &[i32]) -> Option<(usize, f64)> {
        let mut best_gain = -1.0;
        let mut best_split = None;

        let n_samples = X.len();
        let n_features = X[0].len();

        for feat_idx in 0..n_features {
            let mut values: Vec<f64> = X.iter().map(|row| row[feat_idx]).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            values.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

            if values.len() < 2 {
                continue;
            }

            for t in values.iter().skip(1) {
                let mut left_mask = vec![false; n_samples];
                let mut right_mask = vec![false; n_samples];

                for (i, row) in X.iter().enumerate() {
                    if row[feat_idx] <= *t {
                        left_mask[i] = true;
                    } else {
                        right_mask[i] = true;
                    }
                }

                let left_count = left_mask.iter().filter(|&&x| x).count();
                if left_count < self.min_samples_split {
                    continue;
                }

                let gain = self.information_gain(y, &left_mask, &right_mask);
                if gain > best_gain {
                    best_gain = gain;
                    best_split = Some((feat_idx, *t));
                }
            }
        }

        best_split
    }

    fn build_tree(&self, X: &[Vec<f64>], y: &[i32], depth: usize) -> TreeNode {
        if depth >= self.max_depth || y.len() < 2 * self.min_samples_split {
            return self.leaf_value(y);
        }

        if let Some((feat_idx, threshold)) = self.best_split(X, y) {
            let mut left_data = Vec::new();
            let mut right_data = Vec::new();
            let mut left_y = Vec::new();
            let mut right_y = Vec::new();

            for (i, row) in X.iter().enumerate() {
                if row[feat_idx] <= threshold {
                    left_data.push(row.clone());
                    left_y.push(y[i]);
                } else {
                    right_data.push(row.clone());
                    right_y.push(y[i]);
                }
            }

            if left_data.is_empty() || right_data.is_empty() {
                return self.leaf_value(y);
            }

            TreeNode::Branch {
                feat_idx,
                threshold,
                left: Box::new(self.build_tree(&left_data, &left_y, depth + 1)),
                right: Box::new(self.build_tree(&right_data, &right_y, depth + 1)),
            }
        } else {
            self.leaf_value(y)
        }
    }

    fn leaf_value(&self, y: &[i32]) -> TreeNode {
        if y.is_empty() {
            return TreeNode::Leaf { value: 0.0 };
        }

        if self.is_classification {
            let mut counts = HashMap::new();
            for &v in y {
                *counts.entry(v).or_insert(0) += 1;
            }
            let max_class = counts.iter().max_by_key(|(_, &c)| c).map(|(&k, _)| k).unwrap_or(0);
            TreeNode::Leaf { value: max_class as f64 }
        } else {
            let mean = y.iter().map(|&v| v as f64).sum::<f64>() / y.len() as f64;
            TreeNode::Leaf { value: mean }
        }
    }

    pub fn predict(&self, X: &[Vec<f64>]) -> Vec<f64> {
        X.iter().map(|x| self.traverse(x, self.tree.as_ref().unwrap())).collect()
    }

    fn traverse(&self, x: &[f64], node: &TreeNode) -> f64 {
        match node {
            TreeNode::Leaf { value } => *value,
            TreeNode::Branch { feat_idx, threshold, left, right } => {
                if x[*feat_idx] <= *threshold {
                    self.traverse(x, left)
                } else {
                    self.traverse(x, right)
                }
            }
        }
    }
}
