//! world4/spaces/discrete.rs
//! A discrete space of n integers {0, 1, ..., n-1}.

use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, PartialEq)]
pub struct Discrete {
    pub n: usize,
    pub start: usize,
    rng: rand::rngs::StdRng,
}

impl Discrete {
    pub fn new(n: usize, start: usize, seed: Option<u64>) -> Self {
        assert!(n >= 1, "n must be >= 1");
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };
        Discrete {
            n,
            start,
            rng,
        }
    }

    pub fn sample(&mut self) -> usize {
        self.rng.gen_range(self.start..self.start + self.n)
    }

    pub fn contains(&self, x: &usize) -> bool {
        *x >= self.start && *x < self.start + self.n
    }

    pub fn seed(&mut self, seed: Option<u64>) {
        self.rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };
    }

    pub fn shape(&self) -> () {
        ()
    }

    pub fn dtype(&self) -> &'static str {
        "u64"
    }
}

impl Default for Discrete {
    fn default() -> Self {
        Self::new(1, 0, None)
    }
}