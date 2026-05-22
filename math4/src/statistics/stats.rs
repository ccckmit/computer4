use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Summary {
    pub n: usize,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub mean: f64,
    pub q3: f64,
    pub max: f64,
    pub sd: f64,
    pub var: f64,
}

pub fn mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        return f64::NAN;
    }
    x.iter().sum::<f64>() / x.len() as f64
}

pub fn median(x: &[f64]) -> f64 {
    quantile(x, 0.5)
}

pub fn variance(x: &[f64], ddof: usize) -> f64 {
    if x.len() <= ddof {
        return f64::NAN;
    }
    let mu = mean(x);
    let sum_sq = x.iter().map(|v| (v - mu).powi(2)).sum::<f64>();
    sum_sq / (x.len() - ddof) as f64
}

pub fn sd(x: &[f64], ddof: usize) -> f64 {
    variance(x, ddof).sqrt()
}

pub fn covariance(x: &[f64], y: &[f64], ddof: usize) -> f64 {
    if x.len() != y.len() || x.len() <= ddof {
        return f64::NAN;
    }
    let mu_x = mean(x);
    let mu_y = mean(y);
    let n = x.len() as f64;
    let mut cov = 0.0;
    for i in 0..x.len() {
        cov += (x[i] - mu_x) * (y[i] - mu_y);
    }
    cov / (n - ddof as f64)
}

pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let cov = covariance(x, y, 1);
    let sd_x = sd(x, 1);
    let sd_y = sd(y, 1);
    if sd_x == 0.0 || sd_y == 0.0 {
        return f64::NAN;
    }
    cov / (sd_x * sd_y)
}

pub fn quantile(x: &[f64], p: f64) -> f64 {
    if x.is_empty() {
        return f64::NAN;
    }
    let mut sorted = x.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = sorted.len();
    let idx = p * (len - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = idx - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

pub fn iqr(x: &[f64]) -> f64 {
    quantile(x, 0.75) - quantile(x, 0.25)
}

pub fn z_score(x: f64, mu: f64, sigma: f64) -> f64 {
    if sigma == 0.0 {
        return f64::NAN;
    }
    (x - mu) / sigma
}

pub fn standardize(x: &[f64]) -> Vec<f64> {
    if x.is_empty() {
        return vec![];
    }
    let mu = mean(x);
    let sigma = sd(x, 1);
    if sigma == 0.0 {
        return vec![0.0; x.len()];
    }
    x.iter().map(|v| (v - mu) / sigma).collect()
}

pub fn summary(x: &[f64]) -> Summary {
    Summary {
        n: x.len(),
        min: min_val(x),
        q1: quantile(x, 0.25),
        median: median(x),
        mean: mean(x),
        q3: quantile(x, 0.75),
        max: max_val(x),
        sd: sd(x, 1),
        var: variance(x, 1),
    }
}

pub fn min_val(x: &[f64]) -> f64 {
    x.iter().cloned().fold(f64::INFINITY, f64::min)
}

pub fn max_val(x: &[f64]) -> f64 {
    x.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
}

pub fn range_stat(x: &[f64]) -> f64 {
    max_val(x) - min_val(x)
}

pub fn sum_stat(x: &[f64]) -> f64 {
    x.iter().sum()
}

pub fn mode(x: &[f64]) -> Vec<f64> {
    if x.is_empty() {
        return vec![];
    }
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for &v in x {
        let key = (v * 1e12) as i64;
        *counts.entry(key).or_insert(0) += 1;
    }
    let max_count = counts.values().max().cloned().unwrap_or(0);
    counts
        .into_iter()
        .filter(|(_, c)| *c == max_count)
        .map(|(k, _)| k as f64 / 1e12)
        .collect()
}

pub fn skewness(x: &[f64]) -> f64 {
    if x.len() < 3 {
        return f64::NAN;
    }
    let mu = mean(x);
    let s = sd(x, 1);
    if s == 0.0 {
        return f64::NAN;
    }
    let n = x.len() as f64;
    let sum = x.iter().map(|v| ((v - mu) / s).powi(3)).sum::<f64>();
    (n / ((n - 1.0) * (n - 2.0))) * sum
}

pub fn kurtosis(x: &[f64]) -> f64 {
    if x.len() < 4 {
        return f64::NAN;
    }
    let mu = mean(x);
    let s = sd(x, 1);
    if s == 0.0 {
        return f64::NAN;
    }
    let n = x.len() as f64;
    let sum = x.iter().map(|v| ((v - mu) / s).powi(4)).sum::<f64>();
    ((n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * sum
        - (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&data) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((median(&data) - 3.0).abs() < 1e-10);
        let data2 = vec![1.0, 2.0, 3.0, 4.0];
        assert!((median(&data2) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_variance() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let var = variance(&data, 1);
        assert!((var - 4.571428571428571).abs() < 1e-10);
    }

    #[test]
    fn test_quantile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((quantile(&data, 0.25) - 2.0).abs() < 1e-10);
        assert!((quantile(&data, 0.5) - 3.0).abs() < 1e-10);
        assert!((quantile(&data, 0.75) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = correlation(&x, &y);
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_standardize() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let z = standardize(&data);
        assert!((mean(&z)).abs() < 1e-10);
        assert!((sd(&z, 1) - 1.0).abs() < 1e-10);
    }
}