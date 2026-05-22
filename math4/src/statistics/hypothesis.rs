use crate::statistics::distributions::{pf, pchisq, pt, qchisq, qnorm, qt};
use crate::statistics::stats::variance;

use crate::statistics::stats::mean;

#[derive(Debug, Clone)]
pub struct ZTestResult {
    pub statistic: f64,
    pub p_value: f64,
    pub reject: bool,
    pub alpha: f64,
    pub ci: (f64, f64),
}

pub fn z_test(x: &[f64], mu0: f64, sigma: f64, alpha: f64) -> ZTestResult {
    let n = x.len() as f64;
    let xbar = mean(x);
    let se = sigma / n.sqrt();
    let z = (xbar - mu0) / se;
    let p_value = 2.0 * (1.0 - normal_cdf(z.abs()));

    ZTestResult {
        statistic: z,
        p_value,
        reject: p_value < alpha,
        alpha,
        ci: (xbar - qnorm(1.0 - alpha / 2.0, 0.0, 1.0, true) * se,
             xbar + qnorm(1.0 - alpha / 2.0, 0.0, 1.0, true) * se),
    }
}

#[derive(Debug, Clone)]
pub struct TTestResult {
    pub statistic: f64,
    pub df: f64,
    pub p_value: f64,
    pub reject: bool,
    pub alpha: f64,
    pub mean: f64,
    pub se: f64,
    pub t_crit: f64,
    pub ci: (f64, f64),
}

pub fn t_test(x: &[f64], mu0: f64, alpha: f64) -> TTestResult {
    let n = x.len() as f64;
    let xbar = mean(x);
    let s = variance(x, 1).sqrt();
    let se = s / n.sqrt();
    let t_stat = (xbar - mu0) / se;
    let df_n = n - 1.0;

    let t_crit = qt(1.0 - alpha / 2.0, df_n, true);
    let p_value = 2.0 * (1.0 - pt(t_stat.abs(), df_n, true));

    TTestResult {
        statistic: t_stat,
        df: df_n,
        p_value,
        reject: p_value < alpha,
        alpha,
        mean: xbar,
        se,
        t_crit,
        ci: (xbar - t_crit * se, xbar + t_crit * se),
    }
}

pub fn t_test_two(x1: &[f64], x2: &[f64], alpha: f64) -> TTestResult {
    let n1 = x1.len() as f64;
    let n2 = x2.len() as f64;
    let x1bar = mean(x1);
    let x2bar = mean(x2);
    let s1_sq = variance(x1, 1);
    let s2_sq = variance(x2, 1);

    let s_pooled = (( (n1 - 1.0) * s1_sq + (n2 - 1.0) * s2_sq ) / (n1 + n2 - 2.0)).sqrt();
    let se = s_pooled * (1.0 / n1 + 1.0 / n2).sqrt();
    let t_stat = (x1bar - x2bar) / se;
    let df_n = n1 + n2 - 2.0;

    let p_value = 2.0 * (1.0 - pt(t_stat.abs(), df_n, true));
    let t_crit = qt(1.0 - alpha / 2.0, df_n, true);

    TTestResult {
        statistic: t_stat,
        df: df_n,
        p_value,
        reject: p_value < alpha,
        alpha,
        mean: x1bar - x2bar,
        se,
        t_crit,
        ci: (x1bar - x2bar - t_crit * se, x1bar - x2bar + t_crit * se),
    }
}

pub fn t_test_paired(x1: &[f64], x2: &[f64], alpha: f64) -> TTestResult {
    let diff: Vec<f64> = x1.iter().zip(x2.iter()).map(|(a, b)| a - b).collect();
    t_test(&diff, 0.0, alpha)
}

#[derive(Debug, Clone)]
pub struct ChiSqTestResult {
    pub statistic: f64,
    pub df: f64,
    pub p_value: f64,
    pub reject: bool,
    pub alpha: f64,
    pub chi_crit: f64,
}

pub fn chisq_test(observed: &[f64], expected: &[f64], alpha: f64) -> ChiSqTestResult {
    if observed.len() != expected.len() {
        return ChiSqTestResult {
            statistic: f64::NAN,
            df: f64::NAN,
            p_value: f64::NAN,
            reject: false,
            alpha,
            chi_crit: f64::NAN,
        };
    }
    let mut chi_sq = 0.0;
    for i in 0..observed.len() {
        if expected[i] != 0.0 {
            chi_sq += (observed[i] - expected[i]).powi(2) / expected[i];
        }
    }
    let df_n = (observed.len() - 1) as f64;
    let p_value = 1.0 - pchisq(chi_sq, df_n, true);
    let chi_crit = qchisq(1.0 - alpha, df_n, true);

    ChiSqTestResult {
        statistic: chi_sq,
        df: df_n,
        p_value,
        reject: p_value < alpha,
        alpha,
        chi_crit,
    }
}

#[derive(Debug, Clone)]
pub struct AnovaResult {
    pub statistic: f64,
    pub df_between: f64,
    pub df_within: f64,
    pub df_total: f64,
    pub ss_between: f64,
    pub ss_within: f64,
    pub ss_total: f64,
    pub ms_between: f64,
    pub ms_within: f64,
    pub p_value: f64,
    pub reject: bool,
    pub alpha: f64,
    pub group_means: Vec<f64>,
    pub grand_mean: f64,
}

pub fn anova(groups: &[&[f64]], alpha: f64) -> AnovaResult {
    if groups.len() < 2 {
        return AnovaResult {
            statistic: f64::NAN,
            df_between: f64::NAN,
            df_within: f64::NAN,
            df_total: f64::NAN,
            ss_between: f64::NAN,
            ss_within: f64::NAN,
            ss_total: f64::NAN,
            ms_between: f64::NAN,
            ms_within: f64::NAN,
            p_value: f64::NAN,
            reject: false,
            alpha,
            group_means: vec![],
            grand_mean: f64::NAN,
        };
    }

    let k = groups.len() as f64;
    let n: f64 = groups.iter().map(|g| g.len() as f64).sum();
    let grand_mean = groups.iter().flat_map(|g| g.iter()).sum::<f64>() / n;

    let mut ss_between = 0.0;
    let mut ss_total = 0.0;
    let mut group_means: Vec<f64> = Vec::new();

    for g in groups {
        let g_mean = mean(g);
        group_means.push(g_mean);
        ss_between += g.len() as f64 * (g_mean - grand_mean).powi(2);
    }

    for g in groups {
        for &x in g.iter() {
            ss_total += (x - grand_mean).powi(2);
        }
    }

    let ss_within = ss_total - ss_between;
    let df_b = k - 1.0;
    let df_w = n - k;
    let df_t = n - 1.0;

    let ms_between = ss_between / df_b;
    let ms_within = ss_within / df_w;
    let f_stat = ms_between / ms_within;

    let p_value = 1.0 - pf(f_stat, df_b, df_w, true);

    AnovaResult {
        statistic: f_stat,
        df_between: df_b,
        df_within: df_w,
        df_total: df_t,
        ss_between,
        ss_within,
        ss_total,
        ms_between,
        ms_within,
        p_value,
        reject: p_value < alpha,
        alpha,
        group_means,
        grand_mean,
    }
}

fn normal_cdf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * (-(x_abs * x_abs)).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_test() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = z_test(&data, 3.0, 1.5, 0.05);
        assert!(result.statistic.is_finite());
    }

    #[test]
    fn test_t_test() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = t_test(&data, 3.0, 0.05);
        assert!(result.statistic.is_finite());
        assert!(result.df > 0.0);
    }

    #[test]
    fn test_t_test_paired() {
        let x1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let x2 = vec![1.5, 3.0, 4.5, 6.0, 7.5];
        let result = t_test_paired(&x1, &x2, 0.05);
        assert!(result.statistic.is_finite());
    }

    #[test]
    fn test_chisq_test() {
        let observed = vec![10.0, 20.0, 30.0];
        let expected = vec![15.0, 20.0, 25.0];
        let result = chisq_test(&observed, &expected, 0.05);
        assert!(result.statistic >= 0.0);
    }

    #[test]
    fn test_anova() {
        let g1: &[f64] = &[1.0, 2.0, 3.0];
        let g2: &[f64] = &[4.0, 5.0, 6.0];
        let result = anova(&[g1, g2], 0.05);
        assert!(result.statistic >= 0.0);
        assert!(result.df_between > 0.0);
    }
}