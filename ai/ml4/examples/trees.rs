use ml4::{DecisionTree, accuracy_score, mean_squared_error};

fn main() {
    // ── Classification ──
    println!("=== DecisionTree (classification) ===");
    // Circles: points inside r=1 vs outside
    let mut x = Vec::new();
    let mut y = Vec::new();
    for _ in 0..200 {
        let a = rand::random::<f64>() * 2.0 - 1.0;
        let b = rand::random::<f64>() * 2.0 - 1.0;
        x.push(vec![a, b]);
        y.push(if a * a + b * b < 1.0 { 1.0 } else { 0.0 });
    }

    let mut tree = DecisionTree::new(5, 5);
    tree.fit(&x, &y);
    let pred = tree.predict(&x);
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    let pred_int: Vec<i32> = pred.iter().map(|&v| v as i32).collect();
    println!("  Accuracy: {:.4}", accuracy_score(&y_int, &pred_int));

    // ── Regression ──
    println!("\n=== DecisionTree (regression) ===");
    let x: Vec<Vec<f64>> = (0..200).map(|i| vec![i as f64 / 10.0]).collect();
    let y: Vec<f64> = x.iter().map(|row| (row[0] * 0.5).sin() + row[0] * 0.1).collect();

    let mut reg = DecisionTree::new(6, 4);
    reg.fit(&x, &y);
    let pred = reg.predict(&x);
    println!("  MSE: {:.6}", mean_squared_error(&y, &pred));
}
