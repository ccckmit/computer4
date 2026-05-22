pub fn entropy(p: &[f64], base: f64) -> f64 {
    let p_sum: f64 = p.iter().sum();
    if p_sum == 0.0 {
        return 0.0;
    }

    let ln_base = base.ln();
    let mut h = 0.0;

    for &pi in p {
        if pi > 0.0 {
            h -= (pi / p_sum) * (pi / p_sum).ln() / ln_base;
        }
    }

    h
}

pub fn cross_entropy(p: &[f64], q: &[f64], base: f64) -> f64 {
    let p_sum: f64 = p.iter().sum();
    let q_sum: f64 = q.iter().sum();
    if p_sum == 0.0 || q_sum == 0.0 {
        return 0.0;
    }

    let ln_base = base.ln();
    let mut h = 0.0;

    for i in 0..p.len().min(q.len()) {
        let pi = p[i] / p_sum;
        let qi = q[i] / q_sum;
        if pi > 0.0 && qi > 0.0 {
            h -= pi * qi.ln() / ln_base;
        }
    }

    h
}

pub fn kl_divergence(p: &[f64], q: &[f64], base: f64) -> f64 {
    let p_sum: f64 = p.iter().sum();
    let q_sum: f64 = q.iter().sum();
    if p_sum == 0.0 || q_sum == 0.0 {
        return 0.0;
    }

    let ln_base = base.ln();
    let mut kl = 0.0;

    for i in 0..p.len().min(q.len()) {
        let pi = p[i] / p_sum;
        let qi = q[i] / q_sum;
        if pi > 0.0 && qi > 0.0 {
            kl += pi * (pi.ln() - qi.ln()) / ln_base;
        }
    }

    kl
}

pub fn mutual_information(x: &[f64], y: &[f64], base: f64) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return f64::NAN;
    }

    let mut x_count: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut y_count: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut joint_count: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();

    for i in 0..x.len() {
        let xi_key = (x[i] * 1e12) as i64;
        let yi_key = (y[i] * 1e12) as i64;
        *x_count.entry(xi_key).or_insert(0) += 1;
        *y_count.entry(yi_key).or_insert(0) += 1;
        *joint_count.entry((xi_key, yi_key)).or_insert(0) += 1;
    }

    let n = x.len() as f64;
    let ln_base = base.ln();
    let mut h_x = 0.0;
    let mut h_y = 0.0;
    let mut h_xy = 0.0;

    for (_, &count) in x_count.iter() {
        let p = count as f64 / n;
        h_x -= p * p.ln() / ln_base;
    }

    for (_, &count) in y_count.iter() {
        let p = count as f64 / n;
        h_y -= p * p.ln() / ln_base;
    }

    for (_, &count) in joint_count.iter() {
        let p = count as f64 / n;
        h_xy -= p * p.ln() / ln_base;
    }

    h_x + h_y - h_xy
}

pub fn conditional_entropy(x: &[f64], y: &[f64], base: f64) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return f64::NAN;
    }

    let mut x_count: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut joint_count: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();

    for i in 0..x.len() {
        let xi_key = (x[i] * 1e12) as i64;
        let yi_key = (y[i] * 1e12) as i64;
        *x_count.entry(xi_key).or_insert(0) += 1;
        *joint_count.entry((xi_key, yi_key)).or_insert(0) += 1;
    }

    let n = x.len() as f64;
    let ln_base = base.ln();
    let mut h_y_given_x = 0.0;

    for i in 0..x.len() {
        let xi_key = (x[i] * 1e12) as i64;
        let yi_key = (y[i] * 1e12) as i64;
        let p_xy = *joint_count.get(&(xi_key, yi_key)).unwrap_or(&0) as f64 / n;
        let p_x = *x_count.get(&xi_key).unwrap_or(&0) as f64 / n;
        if p_xy > 0.0 && p_x > 0.0 {
            let p_y_given_x = p_xy / p_x;
            h_y_given_x -= p_xy * p_y_given_x.ln() / ln_base;
        }
    }

    h_y_given_x
}

pub fn pmi(x: &[f64], y: &[f64], base: f64) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return f64::NAN;
    }

    let mut x_count: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut y_count: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut joint_count: std::collections::HashMap<(i64, i64), usize> = std::collections::HashMap::new();

    for i in 0..x.len() {
        let xi_key = (x[i] * 1e12) as i64;
        let yi_key = (y[i] * 1e12) as i64;
        *x_count.entry(xi_key).or_insert(0) += 1;
        *y_count.entry(yi_key).or_insert(0) += 1;
        *joint_count.entry((xi_key, yi_key)).or_insert(0) += 1;
    }

    let n = x.len() as f64;
    let ln_base = base.ln();
    let mut total_pmi = 0.0;
    let mut count = 0.0;

    for (key, &joint_cnt) in joint_count.iter() {
        let (xi_key, yi_key) = *key;
        let p_xy = joint_cnt as f64 / n;
        let p_x = *x_count.get(&xi_key).unwrap_or(&0) as f64 / n;
        let p_y = *y_count.get(&yi_key).unwrap_or(&0) as f64 / n;

        if p_xy > 0.0 && p_x > 0.0 && p_y > 0.0 {
            let pmi_val = (p_xy / (p_x * p_y)).ln() / ln_base;
            total_pmi += pmi_val * joint_cnt as f64;
            count += joint_cnt as f64;
        }
    }

    if count > 0.0 {
        total_pmi / count
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy() {
        let p = vec![0.5, 0.5];
        let h = entropy(&p, 2.0);
        assert!((h - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_entropy_uniform() {
        let p = vec![0.25, 0.25, 0.25, 0.25];
        let h = entropy(&p, 2.0);
        assert!((h - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_cross_entropy() {
        let p = vec![0.5, 0.5];
        let q = vec![0.5, 0.5];
        let h = cross_entropy(&p, &q, 2.0);
        assert!((h - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_kl_divergence() {
        let p = vec![0.5, 0.5];
        let q = vec![0.5, 0.5];
        let kl = kl_divergence(&p, &q, 2.0);
        assert!(kl.abs() < 0.01);
    }

    #[test]
    fn test_mutual_information() {
        let x = vec![1.0, 1.0, 2.0, 2.0];
        let y = vec![1.0, 1.0, 2.0, 2.0];
        let mi = mutual_information(&x, &y, 2.0);
        assert!(mi >= 0.0);
    }

    #[test]
    fn test_conditional_entropy() {
        let x = vec![1.0, 1.0, 2.0, 2.0];
        let y = vec![1.0, 1.0, 2.0, 2.0];
        let h = conditional_entropy(&x, &y, 2.0);
        assert!(h >= 0.0);
    }

    #[test]
    fn test_pmi() {
        let x = vec![1.0, 1.0, 2.0, 2.0];
        let y = vec![1.0, 1.0, 2.0, 2.0];
        let p = pmi(&x, &y, 2.0);
        assert!(p >= 0.0 || p.is_nan());
    }
}