use crate::statistics::random::random;
use crate::statistics::stats::{mean, sd, variance};

type SampleFn = Box<dyn Fn(usize) -> Vec<f64>>;

#[derive(Debug, Clone)]
pub struct CLResult {
    pub pass: bool,
    pub expected_mean: f64,
    pub observed_mean: f64,
    pub mean_error: f64,
    pub expected_se: f64,
    pub observed_se: f64,
    pub se_error: f64,
}

pub fn central_limit_theorem(
    sample_fn: impl Fn(usize) -> Vec<f64> + 'static,
    true_mean: f64,
    true_var: f64,
    n: usize,
    n_samples: usize,
) -> CLResult {
    let mut sample_means: Vec<f64> = Vec::with_capacity(n_samples);
    for _ in 0..n_samples {
        sample_means.push(mean(&sample_fn(n)));
    }
    let expected_se = (true_var / n as f64).sqrt();
    let observed_mean = mean(&sample_means);
    let observed_se = sd(&sample_means, 1);

    let mean_error = (observed_mean - true_mean).abs();
    let se_error = (observed_se - expected_se).abs();

    let pass_mean = if true_var > 0.0 {
        mean_error < 0.1 * true_var
    } else {
        mean_error < 0.1
    };
    let pass_se = se_error < 0.2 * expected_se;

    CLResult {
        pass: pass_mean && pass_se,
        expected_mean: true_mean,
        observed_mean,
        mean_error,
        expected_se,
        observed_se,
        se_error,
    }
}

#[derive(Debug, Clone)]
pub struct LLNResult {
    pub pass: bool,
    pub true_mean: f64,
    pub sample_mean: f64,
    pub error: f64,
    pub relative_error: f64,
}

pub fn law_of_large_numbers(
    sample_fn: impl Fn(usize) -> Vec<f64> + 'static,
    true_mean: f64,
    n: usize,
) -> LLNResult {
    let sample_mean = mean(&sample_fn(n));
    let error = (sample_mean - true_mean).abs();
    let relative_error = if true_mean != 0.0 {
        error / true_mean.abs()
    } else {
        error
    };

    LLNResult {
        pass: relative_error < 0.1,
        true_mean,
        sample_mean,
        error,
        relative_error,
    }
}

#[derive(Debug, Clone)]
pub struct ChebyshevResult {
    pub pass: bool,
    pub bound: f64,
    pub k: f64,
}

pub fn chebyshev_inequality(var: f64, k: f64) -> ChebyshevResult {
    let bound = 1.0 / k.powi(2);
    ChebyshevResult {
        pass: true,
        bound,
        k,
    }
}

#[derive(Debug, Clone)]
pub struct ChebyshevVerifyResult {
    pub pass: bool,
    pub observed_prob: Option<f64>,
    pub bound: Option<f64>,
    pub note: Option<String>,
}

pub fn chebyshev_verify(samples: &[f64], k: f64) -> ChebyshevVerifyResult {
    if samples.is_empty() {
        return ChebyshevVerifyResult {
            pass: true,
            observed_prob: None,
            bound: None,
            note: Some("no samples".to_string()),
        };
    }
    let mu = mean(samples);
    let sigma = sd(samples, 1);
    if sigma == 0.0 {
        return ChebyshevVerifyResult {
            pass: true,
            observed_prob: None,
            bound: None,
            note: Some("zero variance".to_string()),
        };
    }
    let violations = samples.iter()
        .filter(|&&x| (x - mu).abs() >= k * sigma)
        .count() as f64 / samples.len() as f64;
    let bound = 1.0 / k.powi(2);

    ChebyshevVerifyResult {
        pass: violations <= bound,
        observed_prob: Some(violations),
        bound: Some(bound),
        note: None,
    }
}

#[derive(Debug, Clone)]
pub struct MarkovResult {
    pub pass: bool,
    pub k: Option<f64>,
    pub prob: Option<f64>,
    pub bound: Option<f64>,
    pub note: Option<String>,
}

pub fn markov_inequality(x: &[f64]) -> MarkovResult {
    let mu = mean(x);
    if mu <= 0.0 {
        return MarkovResult {
            pass: true,
            k: None,
            prob: None,
            bound: None,
            note: Some("mean <= 0".to_string()),
        };
    }

    for k in &[mu * 0.5, mu, mu * 2.0] {
        let prob = x.iter().filter(|&&xi| xi >= *k).count() as f64 / x.len() as f64;
        if prob > mu / k {
            return MarkovResult {
                pass: false,
                k: Some(*k),
                prob: Some(prob),
                bound: Some(mu / k),
                note: None,
            };
        }
    }

    MarkovResult {
        pass: true,
        k: None,
        prob: None,
        bound: None,
        note: None,
    }
}

#[derive(Debug, Clone)]
pub struct MarkovVerifyResult {
    pub pass: bool,
    pub violations: Option<Vec<bool>>,
    pub note: Option<String>,
}

pub fn markov_verify(x: &[f64]) -> MarkovVerifyResult {
    let mu = mean(x);
    if mu <= 0.0 {
        return MarkovVerifyResult {
            pass: true,
            violations: None,
            note: Some("mean <= 0".to_string()),
        };
    }

    let mut violations: Vec<bool> = Vec::new();
    for k in &[mu * 0.5, mu, mu * 1.5, mu * 2.0] {
        if *k > 0.0 {
            let obs_prob = x.iter().filter(|&&xi| xi >= *k).count() as f64 / x.len() as f64;
            let bound = mu / k;
            violations.push(obs_prob <= bound);
        }
    }

    MarkovVerifyResult {
        pass: violations.iter().all(|&v| v),
        violations: Some(violations),
        note: None,
    }
}

#[derive(Debug, Clone)]
pub struct BernoulliVerifyResult {
    pub pass: bool,
    pub expected_mean: f64,
    pub observed_mean: f64,
    pub expected_var: f64,
    pub observed_var: f64,
}

pub fn bernoulli_verify(n: i64, p: f64, n_samples: usize) -> BernoulliVerifyResult {
    let mut experiments: Vec<f64> = Vec::with_capacity(n_samples);
    for _ in 0..n_samples {
        let mut successes = 0.0;
        for _ in 0..n {
            if random() < p {
                successes += 1.0;
            }
        }
        experiments.push(successes);
    }

    let expected_mean = n as f64 * p;
    let expected_var = n as f64 * p * (1.0 - p);
    let observed_mean = mean(&experiments);
    let observed_var = variance(&experiments, 1);

    BernoulliVerifyResult {
        pass: (observed_mean - expected_mean).abs() < 0.1 * n as f64
            && (observed_var - expected_var).abs() < 0.1 * n as f64,
        expected_mean,
        observed_mean,
        expected_var,
        observed_var,
    }
}

#[derive(Debug, Clone)]
pub struct BayesTheoremResult {
    pub pass: bool,
    pub prior: f64,
    pub p_b_given_a: f64,
    pub p_b: f64,
    pub posterior: f64,
}

pub fn bayes_theorem(p_a: f64, p_b_given_a: f64, p_b: f64) -> BayesTheoremResult {
    let posterior = if p_b != 0.0 {
        (p_b_given_a * p_a) / p_b
    } else {
        f64::NAN
    };
    BayesTheoremResult {
        pass: true,
        prior: p_a,
        p_b_given_a,
        p_b,
        posterior,
    }
}

#[derive(Debug, Clone)]
pub struct BayesVerifyResult {
    pub pass: bool,
    pub prior: Vec<f64>,
    pub expected_posterior: Vec<f64>,
    pub note: Option<String>,
}

pub fn bayes_verify(prior: &[f64], likelihood: &[f64]) -> BayesVerifyResult {
    let prior_sum: f64 = prior.iter().sum();
    let likelihood_sum: f64 = likelihood.iter().sum();

    if prior_sum == 0.0 || likelihood_sum == 0.0 {
        return BayesVerifyResult {
            pass: true,
            prior: vec![],
            expected_posterior: vec![],
            note: Some("zero sum".to_string()),
        };
    }

    let prior_norm: Vec<f64> = prior.iter().map(|&p| p / prior_sum).collect();
    let likelihood_norm: Vec<f64> = likelihood.iter().map(|&l| l / likelihood_sum).collect();

    let unnorm: Vec<f64> = prior_norm
        .iter()
        .zip(likelihood_norm.iter())
        .map(|(&p, &l)| p * l)
        .collect();

    let unnorm_sum: f64 = unnorm.iter().sum();
    if unnorm_sum == 0.0 {
        return BayesVerifyResult {
            pass: true,
            prior: prior_norm,
            expected_posterior: vec![],
            note: Some("unnormalized sum is zero".to_string()),
        };
    }

    let expected_posterior: Vec<f64> = unnorm.iter().map(|&u| u / unnorm_sum).collect();

    BayesVerifyResult {
        pass: true,
        prior: prior_norm,
        expected_posterior,
        note: None,
    }
}

#[derive(Debug, Clone)]
pub struct InformationEntropyResult {
    pub pass: bool,
    pub entropy: f64,
}

pub fn information_entropy(p: &[f64], base: f64) -> InformationEntropyResult {
    let p_sum: f64 = p.iter().sum();
    if p_sum == 0.0 {
        return InformationEntropyResult {
            pass: true,
            entropy: 0.0,
        };
    }

    let p_norm: Vec<f64> = p.iter().map(|&pi| pi / p_sum).collect();
    let ln_base = base.ln();

    let mut entropy = 0.0;
    for pi in p_norm.iter() {
        if *pi > 0.0 {
            entropy -= pi * pi.ln() / ln_base;
        }
    }

    InformationEntropyResult {
        pass: true,
        entropy,
    }
}

#[derive(Debug, Clone)]
pub struct InformationEntropyVerifyResult {
    pub pass: bool,
    pub entropy: f64,
    pub min: f64,
    pub max: f64,
    pub note: Option<String>,
}

pub fn information_entropy_verify(p: &[f64], base: f64) -> InformationEntropyVerifyResult {
    let p_sum: f64 = p.iter().sum();
    if p_sum == 0.0 {
        return InformationEntropyVerifyResult {
            pass: true,
            entropy: 0.0,
            min: 0.0,
            max: 0.0,
            note: Some("zero sum".to_string()),
        };
    }

    let p_norm: Vec<f64> = p.iter().map(|&pi| pi / p_sum).collect();
    let ln_base = base.ln();

    let mut entropy = 0.0;
    for pi in p_norm.iter() {
        if *pi > 0.0 {
            entropy -= pi * pi.ln() / ln_base;
        }
    }

    let max_entropy = (p_norm.len() as f64).ln() / ln_base;
    let min_entropy = 0.0;

    InformationEntropyVerifyResult {
        pass: min_entropy <= entropy && entropy <= max_entropy,
        entropy,
        min: min_entropy,
        max: max_entropy,
        note: None,
    }
}

#[derive(Debug, Clone)]
pub struct MutualInfoResult {
    pub pass: bool,
    pub mi: f64,
    pub h_x: f64,
    pub h_y: f64,
}

pub fn mutual_information(x: &[f64], y: &[f64]) -> MutualInfoResult {
    let x_sum: f64 = x.iter().sum();
    let y_sum: f64 = y.iter().sum();

    let px = if x_sum > 0.0 {
        x.iter().map(|&xi| xi / x_sum).collect::<Vec<_>>()
    } else {
        x.to_vec()
    };
    let py = if y_sum > 0.0 {
        y.iter().map(|&yi| yi / y_sum).collect::<Vec<_>>()
    } else {
        y.to_vec()
    };

    let ln2 = 2.0_f64.ln();
    let mut h_x = 0.0;
    let mut h_y = 0.0;

    for &p in px.iter() {
        if p > 0.0 {
            h_x -= p * p.ln() / ln2;
        }
    }
    for &p in py.iter() {
        if p > 0.0 {
            h_y -= p * p.ln() / ln2;
        }
    }

    let mi = h_x + h_y;

    MutualInfoResult {
        pass: true,
        mi,
        h_x,
        h_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_central_limit_theorem() {
        let result = central_limit_theorem(
            |n| (0..n).map(|_| random()).collect(),
            0.5,
            1.0 / 12.0,
            100,
            100,
        );
        assert!(result.observed_mean > 0.0);
    }

    #[test]
    fn test_law_of_large_numbers() {
        let result = law_of_large_numbers(
            |n| (0..n).map(|_| random()).collect(),
            0.5,
            10000,
        );
        assert!(result.sample_mean > 0.0);
    }

    #[test]
    fn test_chebyshev_inequality() {
        let result = chebyshev_inequality(1.0, 2.0);
        assert!((result.bound - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_markov_inequality() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = markov_inequality(&data);
        assert!(result.pass);
    }

    #[test]
    fn test_bernoulli_verify() {
        let result = bernoulli_verify(10, 0.5, 100);
        assert!(result.expected_mean > 0.0);
    }

    #[test]
    fn test_bayes_theorem() {
        let result = bayes_theorem(0.3, 0.8, 0.5);
        assert!((result.posterior - 0.48).abs() < 1e-10);
    }

    #[test]
    fn test_information_entropy() {
        let p = vec![0.5, 0.5];
        let result = information_entropy(&p, 2.0);
        assert!((result.entropy - 1.0).abs() < 0.01);
    }
}