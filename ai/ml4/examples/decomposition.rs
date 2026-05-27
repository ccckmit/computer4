use ml4::PCA;

fn main() {
    println!("=== PCA ===");
    // Generate 5D data that's actually 2D with noise
    let mut x = Vec::new();
    for _ in 0..200 {
        let t = rand::random::<f64>() * std::f64::consts::TAU;
        let r = rand::random::<f64>();
        x.push(vec![
            r * t.cos(),
            r * t.sin(),
            rand::random::<f64>() * 0.01,
            rand::random::<f64>() * 0.01,
            rand::random::<f64>() * 0.01,
        ]);
    }

    // Fit PCA reducing to 2 components
    let mut pca = PCA::new(Some(2));
    let x_2d = pca.fit_transform(&x);

    println!("  Original shape:      {} x {}", x.len(), x[0].len());
    println!("  Reduced shape:       {} x {}", x_2d.len(), x_2d[0].len());

    // Explained variance ratio
    let ev = pca.explained_variance.as_ref().unwrap();
    let total: f64 = ev.iter().sum();
    println!("  Explained variance:  {:.4} / {:.4}", ev[0], ev[1]);
    println!("  Explained ratio:     {:.2}%",
        ev[0] / total * 100.0);

    // Inverse transform: reconstruct back to 5D
    let x_reconstructed = pca.inverse_transform(&x_2d);
    println!("  Reconstructed shape: {} x {}", x_reconstructed.len(), x_reconstructed[0].len());

    // Reconstruction error
    let mut err = 0.0;
    for i in 0..x.len() {
        for j in 0..x[0].len() {
            let d = x[i][j] - x_reconstructed[i][j];
            err += d * d;
        }
    }
    err /= (x.len() * x[0].len()) as f64;
    println!("  Reconstruction MSE:  {:.8}", err);
}
