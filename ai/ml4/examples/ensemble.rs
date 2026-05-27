use ml4::{RandomForest, accuracy_score, mean_squared_error};

fn main() {
    // ── Classification ──
    println!("=== RandomForest (classification) ===");
    let mut x = Vec::new();
    let mut y = Vec::new();
    for _ in 0..500 {
        let a = rand::random::<f64>() * 10.0;
        let b = rand::random::<f64>() * 10.0;
        x.push(vec![a, b]);
        y.push(if a > b { 1.0 } else { 0.0 });
    }

    let mut forest = RandomForest::new(20, 8, 4);
    forest.fit(&x, &y);
    let pred = forest.predict(&x);
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    let pred_int: Vec<i32> = pred.iter().map(|&v| v as i32).collect();
    println!("  Accuracy: {:.4}", accuracy_score(&y_int, &pred_int));

    // ── Regression ──
    println!("\n=== RandomForest (regression) ===");
    let x: Vec<Vec<f64>> = (0..200)
        .map(|_| vec![rand::random::<f64>() * 10.0, rand::random::<f64>() * 10.0])
        .collect();
    let y: Vec<f64> = x.iter().map(|r| r[0] * 1.5 + r[1] * 0.5 + 2.0).collect();

    let mut reg = RandomForest::new(20, 8, 4);
    reg.fit(&x, &y);
    // Override classification heuristic: force regression
    reg.is_classification = false;
    let pred = reg.predict(&x);
    println!("  MSE: {:.6}", mean_squared_error(&y, &pred));
}
