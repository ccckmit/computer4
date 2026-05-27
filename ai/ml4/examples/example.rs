use ml4::{
    LinearRegression, LogisticRegression, DecisionTree,
    RandomForest, KMeans, PCA,
    StandardScaler, accuracy_score, r2_score
};

fn main() {
    println!("=== Linear Regression ===");
    let x: Vec<Vec<f64>> = (0..100)
        .map(|i| {
            let base = i as f64 / 100.0;
            vec![base, base * 2.0, base * 3.0]
        })
        .collect();
    let y: Vec<f64> = x.iter()
        .map(|row| 3.0 * row[0] + 2.0 * row[1] - row[2] + 1.0)
        .collect();

    let mut model = LinearRegression::new(0.001, 3000);
    model.fit(&x, &y);
    let pred = model.predict(&x);
    let r2 = r2_score(&y, &pred);
    println!("R2 Score: {:.4}", r2);

    println!("\n=== Logistic Regression ===");
    let x: Vec<Vec<f64>> = (0..100)
        .map(|_| vec![rand::random(), rand::random()])
        .collect();
    let y: Vec<f64> = x.iter()
        .map(|row| if row[0] + row[1] > 0.0 { 1.0 } else { 0.0 })
        .collect();

    let mut model = LogisticRegression::new(0.1, 500);
    model.fit(&x, &y);
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    let pred: Vec<i32> = model.predict(&x).iter().map(|&v| v as i32).collect();
    let acc = accuracy_score(&y_int, &pred);
    println!("Accuracy: {:.4}", acc);

    println!("\n=== Decision Tree ===");
    let x = vec![
        vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0], vec![6.0]
    ];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let mut tree = DecisionTree::new(3, 2);
    tree.fit(&x, &y);
    let pred: Vec<i32> = tree.predict(&x).iter().map(|&v| v as i32).collect();
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    let acc = accuracy_score(&y_int, &pred);
    println!("Accuracy: {:.4}", acc);

    println!("\n=== Random Forest ===");
    let x: Vec<Vec<f64>> = (0..100)
        .map(|_| vec![rand::random(), rand::random(), rand::random(), rand::random()])
        .collect();
    let y: Vec<f64> = x.iter()
        .map(|row| if row[0] + row[1] > 0.0 { 1.0 } else { 0.0 })
        .collect();

    let mut forest = RandomForest::new(5, 5, 2);
    forest.fit(&x, &y);
    let pred: Vec<i32> = forest.predict(&x).iter().map(|&v| v as i32).collect();
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    let acc = accuracy_score(&y_int, &pred);
    println!("Accuracy: {:.4}", acc);

    println!("\n=== KMeans Clustering ===");
    let mut x1 = Vec::new();
    let mut x2 = Vec::new();
    for _ in 0..30 {
        x1.push(vec![rand::random::<f64>() * 2.0 + 2.0, rand::random::<f64>() * 2.0 + 2.0]);
        x2.push(vec![rand::random::<f64>() * 2.0 - 2.0, rand::random::<f64>() * 2.0 - 2.0]);
    }
    let mut x = x1;
    x.extend(x2);

    let mut kmeans = KMeans::new(2, 300, 5);
    kmeans.fit(&x);
    let labels = kmeans.predict(&x);
    let count0 = labels.iter().filter(|&&l| l == 0).count();
    let count1 = labels.iter().filter(|&&l| l == 1).count();
    println!("Cluster sizes: {}, {}", count0, count1);

    println!("\n=== PCA ===");
    let x: Vec<Vec<f64>> = (0..100)
        .map(|_| vec![rand::random(), rand::random(), rand::random(), rand::random(), rand::random()])
        .collect();
    let mut pca = PCA::new(Some(2));
    let x_transformed = pca.fit_transform(&x);
    println!("Transformed shape: ({}, {})", x_transformed.len(), x_transformed[0].len());

    println!("\n=== StandardScaler ===");
    let x = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
    let mut scaler = StandardScaler::new();
    let x_scaled = scaler.fit_transform(&x);
    let mean: f64 = x_scaled.iter().map(|r| r[0]).sum::<f64>() / x_scaled.len() as f64;
    println!("Mean after scaling: {:.6}", mean);

    println!("\nAll examples completed successfully!");
}
