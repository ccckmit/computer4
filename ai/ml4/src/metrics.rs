pub fn accuracy_score<T: PartialEq>(y_true: &[T], y_pred: &[T]) -> f64 {
    if y_true.len() != y_pred.len() {
        panic!("Arrays must have same length");
    }
    let correct = y_true.iter().zip(y_pred.iter()).filter(|(a, b)| a == b).count();
    correct as f64 / y_true.len() as f64
}

pub fn mean_squared_error(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() {
        panic!("Arrays must have same length");
    }
    let sum: f64 = y_true.iter().zip(y_pred.iter())
        .map(|(y, p)| (y - p).powi(2))
        .sum();
    sum / y_true.len() as f64
}

pub fn r2_score(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.len() != y_pred.len() {
        panic!("Arrays must have same length");
    }
    let mean = y_true.iter().sum::<f64>() / y_true.len() as f64;
    let ss_res: f64 = y_true.iter().zip(y_pred.iter())
        .map(|(y, p)| (y - p).powi(2))
        .sum();
    let ss_tot: f64 = y_true.iter()
        .map(|y| (y - mean).powi(2))
        .sum();

    if ss_tot == 0.0 {
        return 0.0;
    }
    1.0 - ss_res / ss_tot
}

pub fn confusion_matrix(y_true: &[i32], y_pred: &[i32]) -> Vec<Vec<i32>> {
    if y_true.len() != y_pred.len() {
        panic!("Arrays must have same length");
    }

    let mut classes: Vec<i32> = y_true.iter().chain(y_pred.iter()).copied().collect();
    classes.sort();
    classes.dedup();

    let n = classes.len();
    let mut cm = vec![vec![0i32; n]; n];

    for (&t, &p) in y_true.iter().zip(y_pred.iter()) {
        if let (Some(ti), Some(pi)) = (classes.iter().position(|&x| x == t),
                                        classes.iter().position(|&x| x == p)) {
            cm[ti][pi] += 1;
        }
    }

    cm
}
