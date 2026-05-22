use rand::distributions::Distribution;
use rand::Rng;
use statrs::distribution::{ContinuousCDF, DiscreteCDF};

const SQRT_TWO_PI: f64 = 2.5066282746310005024157652848110452530069867406099_f64;

fn ln_gamma(mut z: f64) -> f64 {
    let g = 7.0;
    let c = [
        0.99999999999980993_f64,
        676.5203681218851_f64,
        -1259.1392167224028_f64,
        771.32342877765313_f64,
        -176.61502916214059_f64,
        12.507343278686905_f64,
        -0.13857109526572012_f64,
        9.9843695780195716e-6_f64,
        1.5056327351493116e-7_f64,
    ];
    if z < 0.5 {
        return (std::f64::consts::PI / (std::f64::consts::PI * z).sin()).ln() - ln_gamma(1.0 - z);
    }
    z -= 1.0;
    let mut x = c[0];
    for i in 1..(g as usize + 2) {
        x += c[i] / (z + i as f64);
    }
    let t = z + g + 0.5;
    0.5 * (1.8378770664093453_f64 + (z + 0.5) * t.ln() - t) + x.ln()
}

fn ln_factorial_float(n: f64) -> f64 {
    ln_gamma(n + 1.0)
}

pub fn dnorm(x: f64, mean: f64, sd: f64) -> f64 {
    if sd <= 0.0 {
        return f64::NAN;
    }
    let z = (x - mean) / sd;
    (-0.5 * z * z).exp() / (sd * SQRT_TWO_PI)
}

pub fn pnorm(q: f64, mean: f64, sd: f64, lower_tail: bool) -> f64 {
    let p = statrs::distribution::Normal::new(mean, sd)
        .map(|d| d.cdf(q))
        .unwrap_or(f64::NAN);
    if lower_tail {
        p
    } else {
        1.0 - p
    }
}

pub fn qnorm(p: f64, mean: f64, sd: f64, lower_tail: bool) -> f64 {
    let p_adj = if lower_tail { p } else { 1.0 - p };
    if p_adj <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p_adj >= 1.0 {
        return f64::INFINITY;
    }
    statrs::distribution::Normal::new(mean, sd)
        .ok()
        .map(|d| d.inverse_cdf(p_adj) as f64)
        .unwrap_or(f64::NAN)
}

pub fn rnorm(n: usize, mean: f64, sd: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let dist = statrs::distribution::Normal::new(mean, sd).unwrap();
    (0..n).map(|_| dist.sample(&mut rng)).collect()
}

pub fn dt(x: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return f64::NAN;
    }
    let a = (df + 1.0) / 2.0;
    let ln_beta = ln_gamma(a) + ln_gamma(0.5) - ln_gamma((df + 1.0) / 2.0);
    ((df + 1.0) / 2.0).ln()
        - ln_beta
        - ((df + 1.0) / 2.0) * (1.0 + x * x / df).ln()
}

pub fn pt(q: f64, df: f64, lower_tail: bool) -> f64 {
    let p = statrs::distribution::StudentsT::new(0.0, 1.0, df)
        .map(|d| d.cdf(q))
        .unwrap_or(f64::NAN);
    if lower_tail {
        p
    } else {
        1.0 - p
    }
}

pub fn qt(p: f64, df: f64, lower_tail: bool) -> f64 {
    let p_adj = if lower_tail { p } else { 1.0 - p };
    if p_adj <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p_adj >= 1.0 {
        return f64::INFINITY;
    }
    statrs::distribution::StudentsT::new(0.0, 1.0, df)
        .ok()
        .map(|d| d.inverse_cdf(p_adj) as f64)
        .unwrap_or(f64::NAN)
}

pub fn rt(n: usize, df: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let dist = statrs::distribution::StudentsT::new(0.0, 1.0, df).unwrap();
    (0..n).map(|_| dist.sample(&mut rng)).collect()
}

pub fn dchisq(x: f64, df: f64) -> f64 {
    if x <= 0.0 || df <= 0.0 {
        return f64::NAN;
    }
    let k = df / 2.0;
    let ln_norm = k * (df / 2.0).ln() + ln_gamma(k) - (k - 1.0) * (2.0 * x / df).ln();
    let ln_gamma_kx = ln_gamma(k + x / 2.0);
    (ln_norm - ln_gamma_kx - x / 2.0).exp()
}

pub fn pchisq(q: f64, df: f64, lower_tail: bool) -> f64 {
    let p = statrs::distribution::ChiSquared::new(df)
        .map(|d| d.cdf(q))
        .unwrap_or(f64::NAN);
    if lower_tail {
        p
    } else {
        1.0 - p
    }
}

pub fn qchisq(p: f64, df: f64, lower_tail: bool) -> f64 {
    let p_adj = if lower_tail { p } else { 1.0 - p };
    if p_adj <= 0.0 {
        return 0.0;
    }
    if p_adj >= 1.0 {
        return f64::INFINITY;
    }
    statrs::distribution::ChiSquared::new(df)
        .ok()
        .map(|d| d.inverse_cdf(p_adj) as f64)
        .unwrap_or(f64::NAN)
}

pub fn rchisq(n: usize, df: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let dist = statrs::distribution::ChiSquared::new(df).unwrap();
    (0..n).map(|_| dist.sample(&mut rng)).collect()
}

pub fn df(x: f64, df1: f64, df2: f64) -> f64 {
    if df1 <= 0.0 || df2 <= 0.0 || x < 0.0 {
        return f64::NAN;
    }
    let a = df1 / 2.0;
    let b = df2 / 2.0;
    let c = (df1 * x) / (df1 * x + df2);
    let ln_term = a.ln() * df1 / 2.0 + b.ln() * df2 / 2.0 - ln_gamma(a + b) + ln_gamma(a) + ln_gamma(b)
        - ln_gamma(df1 / 2.0) - ln_gamma(df2 / 2.0);
    let ln_result = ln_term
        - ((df1 + df2) / 2.0) * (1.0 + (df1 * x / df2).ln())
        + ((df1 / 2.0 - 1.0) * c.ln())
        + ((df2 / 2.0 - 1.0) * (1.0 - c).ln());
    ln_result.exp()
}

pub fn pf(q: f64, df1: f64, df2: f64, lower_tail: bool) -> f64 {
    let p = statrs::distribution::FisherSnedecor::new(df1, df2)
        .map(|d| d.cdf(q))
        .unwrap_or(f64::NAN);
    if lower_tail {
        p
    } else {
        1.0 - p
    }
}

pub fn qf(p: f64, df1: f64, df2: f64, lower_tail: bool) -> f64 {
    let p_adj = if lower_tail { p } else { 1.0 - p };
    if p_adj <= 0.0 {
        return 0.0;
    }
    if p_adj >= 1.0 {
        return f64::INFINITY;
    }
    statrs::distribution::FisherSnedecor::new(df1, df2)
        .ok()
        .map(|d| d.inverse_cdf(p_adj) as f64)
        .unwrap_or(f64::NAN)
}

pub fn rf(n: usize, df1: f64, df2: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let dist = statrs::distribution::FisherSnedecor::new(df1, df2).unwrap();
    (0..n).map(|_| dist.sample(&mut rng)).collect()
}

pub fn log_choose(n: f64, k: f64) -> f64 {
    if k < 0.0 || k > n {
        return f64::NEG_INFINITY;
    }
    if k == 0.0 || k == n {
        return 0.0;
    }
    ln_factorial_float(n) - ln_factorial_float(k) - ln_factorial_float(n - k)
}

pub fn dbinom(k: f64, n: f64, p: f64) -> f64 {
    if p < 0.0 || p > 1.0 || n < 0.0 || k < 0.0 || k > n {
        return f64::NAN;
    }
    if n != n.floor() {
        return f64::NAN;
    }
    let n_int = n as i64;
    let k_int = k as i64;
    let log_p = if p == 0.0 {
        if k_int == 0 {
            0.0
        } else {
            f64::NEG_INFINITY
        }
    } else if p == 1.0 {
        if k_int == n_int {
            0.0
        } else {
            f64::NEG_INFINITY
        }
    } else {
        log_choose(n, k) + (k * p.ln()) + ((n - k) * (1.0 - p).ln())
    };
    log_p.exp()
}

pub fn pbinom(k: f64, n: f64, prob: f64, lower_tail: bool) -> f64 {
    let n_int = n as i64;
    let k_int = k as i64;
    let p = statrs::distribution::Binomial::new(prob, n_int as u64)
        .map(|d| d.cdf(k_int as u64))
        .unwrap_or(f64::NAN);
    if lower_tail {
        p
    } else {
        1.0 - p
    }
}

pub fn qbinom(p: f64, n: f64, prob: f64, lower_tail: bool) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return n;
    }
    let p_adj = if lower_tail { p } else { 1.0 - p };
    statrs::distribution::Binomial::new(prob, n as u64)
        .ok()
        .map(|d| d.inverse_cdf(p_adj) as f64)
        .unwrap_or(f64::NAN)
}

pub fn rbinom(n: usize, size: i64, prob: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    statrs::distribution::Binomial::new(prob, size as u64)
        .map(|dist| (0..n).map(|_| dist.sample(&mut rng) as f64).collect())
        .unwrap_or_else(|_| vec![])
}

pub fn dpois(k: f64, lambda: f64) -> f64 {
    if lambda < 0.0 || k < 0.0 || k != k.floor() {
        return f64::NAN;
    }
    if lambda == 0.0 {
        return if k == 0.0 { 1.0 } else { 0.0 };
    }
    let log_p = -lambda + (k * lambda.ln()) - ln_factorial_float(k);
    log_p.exp()
}

pub fn ppois(k: f64, lambda: f64, lower_tail: bool) -> f64 {
    let k_int = k as i64;
    let p = statrs::distribution::Poisson::new(lambda)
        .map(|d| d.cdf(k_int as u64))
        .unwrap_or(f64::NAN);
    if lower_tail {
        p
    } else {
        1.0 - p
    }
}

pub fn qpois(p: f64, lambda: f64, lower_tail: bool) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let p_adj = if lower_tail { p } else { 1.0 - p };
    statrs::distribution::Poisson::new(lambda)
        .ok()
        .map(|d| d.inverse_cdf(p_adj) as f64)
        .unwrap_or(f64::NAN)
}

pub fn rpois(n: usize, lambda: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    statrs::distribution::Poisson::new(lambda)
        .map(|dist| (0..n).map(|_| dist.sample(&mut rng) as f64).collect())
        .unwrap_or_else(|_| vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnorm() {
        let result = dnorm(0.0, 0.0, 1.0);
        assert!((result - 0.3989422804014317).abs() < 1e-6);
    }

    #[test]
    fn test_pnorm() {
        let p = pnorm(0.0, 0.0, 1.0, true);
        assert!((p - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_qnorm() {
        let q = qnorm(0.5, 0.0, 1.0, true);
        assert!(q.abs() < 1e-10);
    }

    #[test]
    fn test_rnorm() {
        let samples = rnorm(10, 0.0, 1.0);
        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_pt() {
        let p = pt(0.0, 10.0, true);
        assert!((p - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_qt() {
        let q = qt(0.5, 10.0, true);
        assert!(q.abs() < 0.1);
    }

    #[test]
    fn test_pchisq() {
        let p = pchisq(5.0, 3.0, true);
        assert!(p > 0.5 && p < 1.0);
    }

    #[test]
    fn test_pf() {
        let p = pf(1.0, 5.0, 10.0, true);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_dbinom() {
        let p = dbinom(5.0, 10.0, 0.5);
        assert!(p > 0.0);
    }

    #[test]
    fn test_pbinom() {
        let p = pbinom(5.0, 10.0, 0.5, true);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_dpois() {
        let p = dpois(3.0, 2.0);
        assert!(p > 0.0);
    }

    #[test]
    fn test_ppois() {
        let p = ppois(3.0, 2.0, true);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_rpois() {
        let samples = rpois(10, 2.0);
        assert_eq!(samples.len(), 10);
    }
}