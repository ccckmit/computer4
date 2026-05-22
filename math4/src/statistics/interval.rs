use crate::statistics::distributions::{qchisq, qnorm, qt};
use crate::statistics::random::random;

use crate::statistics::stats::{mean, sd, variance};

#[derive(Debug, Clone)]
pub struct ConfIntervalResult {
    pub lower: f64,
    pub upper: f64,
    pub mean: f64,
    pub se: f64,
    pub alpha: f64,
    pub confidence: f64,
    pub df: f64,
}

pub fn conf_interval(x: &[f64], alpha: f64) -> ConfIntervalResult {
    let n = x.len();
    if n == 0 {
        return ConfIntervalResult {
            lower: f64::NAN,
            upper: f64::NAN,
            mean: f64::NAN,
            se: f64::NAN,
            alpha,
            confidence: 1.0 - alpha,
            df: f64::NAN,
        };
    }
    let xbar = mean(x);
    let s = sd(x, 1);
    let se = s / (n as f64).sqrt();
    let t_crit = qt(1.0 - alpha / 2.0, (n - 1) as f64, true);

    ConfIntervalResult {
        lower: xbar - t_crit * se,
        upper: xbar + t_crit * se,
        mean: xbar,
        se,
        alpha,
        confidence: 1.0 - alpha,
        df: (n - 1) as f64,
    }
}

#[derive(Debug, Clone)]
pub struct ConfIntervalProportionResult {
    pub lower: f64,
    pub upper: f64,
    pub proportion: f64,
    pub se: f64,
    pub alpha: f64,
    pub confidence: f64,
}

pub fn conf_interval_proportion(x: f64, n: f64, alpha: f64) -> ConfIntervalProportionResult {
    let p_hat = x / n;
    let z_crit = qnorm(1.0 - alpha / 2.0, 0.0, 1.0, true);
    let se = ((p_hat * (1.0 - p_hat)) / n).sqrt();

    ConfIntervalProportionResult {
        lower: p_hat - z_crit * se,
        upper: p_hat + z_crit * se,
        proportion: p_hat,
        se,
        alpha,
        confidence: 1.0 - alpha,
    }
}

type StatisticFn = Box<dyn Fn(&[f64]) -> f64>;

#[derive(Debug, Clone)]
pub struct BootstrapCIResult {
    pub lower: f64,
    pub upper: f64,
    pub statistic: f64,
    pub alpha: f64,
    pub confidence: f64,
    pub n_bootstrap: usize,
}

pub fn bootstrap_ci<F>(x: &[f64], statistic_fn: F, n_bootstrap: usize, alpha: f64) -> BootstrapCIResult
where
    F: Fn(&[f64]) -> f64 + 'static,
{
    let x_len = x.len();
    if x_len == 0 {
        return BootstrapCIResult {
            lower: f64::NAN,
            upper: f64::NAN,
            statistic: f64::NAN,
            alpha,
            confidence: 1.0 - alpha,
            n_bootstrap,
        };
    }
    let mut stats: Vec<f64> = Vec::with_capacity(n_bootstrap);
    for _ in 0..n_bootstrap {
        let mut resampled: Vec<f64> = Vec::with_capacity(x_len);
        for _ in 0..x_len {
            let idx = (random() * x_len as f64) as usize;
            resampled.push(x[idx]);
        }
        stats.push(statistic_fn(&resampled));
    }
    stats.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lower_idx = ((alpha / 2.0) * n_bootstrap as f64).floor() as usize;
    let upper_idx = ((1.0 - alpha / 2.0) * n_bootstrap as f64).floor() as usize;
    let lower_idx = lower_idx.min(n_bootstrap - 1);
    let upper_idx = upper_idx.min(n_bootstrap - 1);

    BootstrapCIResult {
        lower: stats[lower_idx],
        upper: stats[upper_idx],
        statistic: statistic_fn(x),
        alpha,
        confidence: 1.0 - alpha,
        n_bootstrap,
    }
}

#[derive(Debug, Clone)]
pub struct ConfIntervalVarianceResult {
    pub lower: f64,
    pub upper: f64,
    pub variance: f64,
    pub alpha: f64,
    pub confidence: f64,
    pub df: f64,
}

pub fn conf_interval_variance(x: &[f64], alpha: f64) -> ConfIntervalVarianceResult {
    let n = x.len();
    if n == 0 {
        return ConfIntervalVarianceResult {
            lower: f64::NAN,
            upper: f64::NAN,
            variance: f64::NAN,
            alpha,
            confidence: 1.0 - alpha,
            df: f64::NAN,
        };
    }
    let s_sq = variance(x, 1);
    let df_n = (n - 1) as f64;

    let chi_lower = qchisq(1.0 - alpha / 2.0, df_n, true);
    let chi_upper = qchisq(alpha / 2.0, df_n, true);

    let lower = (df_n * s_sq) / chi_lower;
    let upper = (df_n * s_sq) / chi_upper;

    ConfIntervalVarianceResult {
        lower: lower.min(upper),
        upper: lower.max(upper),
        variance: s_sq,
        alpha,
        confidence: 1.0 - alpha,
        df: df_n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conf_interval() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = conf_interval(&data, 0.05);
        assert!(result.lower < result.mean);
        assert!(result.upper > result.mean);
    }

    #[test]
    fn test_conf_interval_proportion() {
        let result = conf_interval_proportion(30.0, 100.0, 0.05);
        assert!(result.lower < result.proportion);
        assert!(result.upper > result.proportion);
    }

    #[test]
    fn test_bootstrap_ci() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = bootstrap_ci(&data, |x| mean(x), 100, 0.05);
        assert!(result.lower < result.statistic);
        assert!(result.upper > result.statistic);
    }

    #[test]
    fn test_conf_interval_variance() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = conf_interval_variance(&data, 0.05);
        assert!(result.lower < result.variance);
        assert!(result.upper > result.variance);
    }
}