use ml4::{StandardScaler, train_test_split};

fn main() {
    // ── StandardScaler ──
    println!("=== StandardScaler ===");
    let x = vec![
        vec![10.0, 100.0],
        vec![20.0, 200.0],
        vec![30.0, 300.0],
        vec![40.0, 400.0],
        vec![50.0, 500.0],
    ];

    let mut scaler = StandardScaler::new();
    let x_scaled = scaler.fit_transform(&x);

    println!("  Original:");
    for row in &x {
        println!("    {:>8.1} {:>8.1}", row[0], row[1]);
    }
    println!("  Scaled:");
    for row in &x_scaled {
        println!("    {:>8.4} {:>8.4}", row[0], row[1]);
    }

    let mean0: f64 = x_scaled.iter().map(|r| r[0]).sum::<f64>() / x_scaled.len() as f64;
    let mean1: f64 = x_scaled.iter().map(|r| r[1]).sum::<f64>() / x_scaled.len() as f64;
    println!("  Mean after scaling:  {:.6}  {:.6}", mean0, mean1);

    // ── train_test_split ──
    println!("\n=== train_test_split ===");
    let x: Vec<Vec<f64>> = (0..100).map(|i| vec![i as f64]).collect();
    let y: Vec<f64> = (0..100).map(|i| i as f64 * 2.0).collect();

    let (x_tr, x_te, y_tr, y_te) = train_test_split(&x, &y, 0.3, Some(42));

    println!("  Train size: {}", x_tr.len());
    println!("  Test  size: {}", x_te.len());
    println!(
        "  First train sample: x={:.0} y={:.0}",
        x_tr[0][0], y_tr[0]
    );
}
