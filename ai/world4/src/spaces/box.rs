//! world4/spaces/box.rs
//! A (possibly unbounded) box in R^n.

use rand::{Rng, SeedableRng};
use std::f32;

#[derive(Debug, Clone, PartialEq)]
pub struct Box {
    pub low: Vec<f32>,
    pub high: Vec<f32>,
    pub shape: Vec<usize>,
    pub dtype: String,
    rng: rand::rngs::StdRng,
}

impl Box {
    pub fn new(
        low: Vec<f32>,
        high: Vec<f32>,
        shape: Option<Vec<usize>>,
        dtype: &str,
        seed: Option<u64>,
    ) -> Self {
        let shape = match shape {
            Some(s) => s,
            None => {
                if low.len() == high.len() {
                    vec![low.len()]
                } else {
                    vec![low.len().max(high.len())]
                }
            }
        };

        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        Box {
            low,
            high,
            shape,
            dtype: dtype.to_string(),
            rng,
        }
    }

    pub fn sample(&mut self) -> Vec<f32> {
        let mut sample = Vec::with_capacity(self.shape.iter().product::<usize>());

        for i in 0..self.shape.iter().product::<usize>() {
            let mut low = self.low.get(i % self.low.len()).copied().unwrap_or(0.0);
            let mut high = self.high.get(i % self.high.len()).copied().unwrap_or(1.0);

            if low == f32::NEG_INFINITY {
                low = -3.4e38;
            }
            if high == f32::INFINITY {
                high = 3.4e38;
            }

            let val: f32 = self.rng.gen::<f32>() * (high - low) + low;
            sample.push(val);
        }

        sample
    }

    pub fn contains(&self, x: &[f32]) -> bool {
        if x.len() != self.shape.iter().product::<usize>() {
            return false;
        }
        for (i, &val) in x.iter().enumerate() {
            let low = self.low.get(i % self.low.len()).copied().unwrap_or(0.0);
            let high = self.high.get(i % self.high.len()).copied().unwrap_or(1.0);
            if val < low || val > high {
                return false;
            }
        }
        true
    }

    pub fn seed(&mut self, seed: Option<u64>) {
        self.rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };
    }
}

impl Default for Box {
    fn default() -> Self {
        Self::new(vec![0.0], vec![1.0], Some(vec![1]), "f32", None)
    }
}