use ml4::KMeans;

fn main() {
    println!("=== KMeans ===");
    // Three well-separated clusters
    let mut x = Vec::new();

    for _ in 0..40 {
        x.push(vec![
            rand::random::<f64>() * 2.0 + 0.0,
            rand::random::<f64>() * 2.0 + 0.0,
        ]);
    }
    for _ in 0..40 {
        x.push(vec![
            rand::random::<f64>() * 2.0 + 5.0,
            rand::random::<f64>() * 2.0 + 5.0,
        ]);
    }
    for _ in 0..40 {
        x.push(vec![
            rand::random::<f64>() * 2.0 + 10.0,
            rand::random::<f64>() * 2.0 + 0.0,
        ]);
    }

    let mut kmeans = KMeans::new(3, 300, 10);
    kmeans.fit(&x);

    let labels = kmeans.predict(&x);
    for k in 0..3 {
        let count = labels.iter().filter(|&&l| l == k).count();
        println!("  Cluster {} size: {}", k, count);
    }

    // Show centroids
    let centroids = kmeans.centroids.as_ref().unwrap();
    for (i, c) in centroids.iter().enumerate() {
        println!("  Centroids[{}]: ({:.4}, {:.4})", i, c[0], c[1]);
    }
}
