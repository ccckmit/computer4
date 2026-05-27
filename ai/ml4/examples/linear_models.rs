use ml4::{LinearRegression, LogisticRegression, accuracy_score, mean_squared_error, r2_score};

fn main() {
    // ── LinearRegression ──
    println!("=== LinearRegression ===");
    // y = 3x0 + 2x1 - x2 + 1
    let x: Vec<Vec<f64>> = (0..200)
        .map(|_| {
            let x0 = rand::random::<f64>() * 10.0;
            let x1 = rand::random::<f64>() * 10.0;
            let x2 = rand::random::<f64>() * 10.0;
            vec![x0, x1, x2]
        })
        .collect();
    let y: Vec<f64> = x.iter()
        .map(|row| 3.0 * row[0] + 2.0 * row[1] - row[2] + 1.0)
        .collect();

    let mut model = LinearRegression::new(0.01, 2000);
    model.fit(&x, &y);

    let pred = model.predict(&x);
    println!("  MSE: {:.6}", mean_squared_error(&y, &pred));
    println!("  R²:  {:.6}", r2_score(&y, &pred));

    // Predict a single unseen sample
    let single = vec![vec![5.0, 3.0, 2.0]];
    let p = model.predict(&single);
    println!("  predict([5,3,2]) = {:.4}  (expect 5*3+3*2-2+1 = 20)", p[0]);

    // ── LogisticRegression ──
    println!("\n=== LogisticRegression ===");
    let x: Vec<Vec<f64>> = (0..200)
        .map(|_| vec![rand::random::<f64>() * 10.0, rand::random::<f64>() * 10.0])
        .collect();
    let y: Vec<f64> = x.iter()
        .map(|row| if row[0] + row[1] > 10.0 { 1.0 } else { 0.0 })
        .collect();

    let mut logit = LogisticRegression::new(0.01, 2000);
    logit.fit(&x, &y);

    let pred_labels = logit.predict(&x);
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    println!("  Accuracy: {:.4}", accuracy_score(&y_int, &pred_labels));

    let proba = logit.predict_proba(&x);
    println!("  First 5 probabilities: {:.2?}", &proba[..5]);
}
