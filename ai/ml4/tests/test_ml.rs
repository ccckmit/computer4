use ml4::{
    LinearRegression, LogisticRegression, DecisionTree,
    RandomForest, KMeans, PCA, StandardScaler,
    accuracy_score, mean_squared_error, r2_score, confusion_matrix,
    train_test_split,
};

#[test]
fn test_linear_regression_fit_predict() {
    let X = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0]];
    let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

    let mut model = LinearRegression::new(0.01, 1000);
    model.fit(&X, &y);

    let pred = model.predict(&X);
    assert_eq!(pred.len(), y.len());

    for (p, &y) in pred.iter().zip(&y) {
        assert!((p - y).abs() < 0.5, "Expected close to {}, got {}", y, p);
    }
}

#[test]
fn test_linear_regression_multi_feature() {
    let mut X = Vec::new();
    let mut y = Vec::new();

    for _ in 0..100 {
        let x0 = rand::random::<f64>() * 2.0 - 1.0;
        let x1 = rand::random::<f64>() * 2.0 - 1.0;
        let x2 = rand::random::<f64>() * 2.0 - 1.0;
        X.push(vec![x0, x1, x2]);
        y.push(3.0 * x0 + 2.0 * x1 - x2 + 1.0);
    }

    let mut model = LinearRegression::new(0.1, 1000);
    model.fit(&X, &y);

    let pred = model.predict(&X);
    let r2 = r2_score(&y, &pred);
    assert!(r2 > 0.9, "R2 score should be > 0.9, got {}", r2);
}

#[test]
fn test_linear_regression_single_predict() {
    let x = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0]];
    let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

    let mut model = LinearRegression::new(0.01, 1000);
    model.fit(&x, &y);

    let pred = model.predict(&vec![vec![6.0]]);
    assert!((pred[0] - 12.0).abs() < 1.0);
}

#[test]
fn test_logistic_regression_binary() {
    let X = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0], vec![6.0]];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    let mut model = LogisticRegression::new(0.1, 1000);
    model.fit(&X, &y);

    let pred = model.predict(&X);
    let acc = accuracy_score(&y.iter().map(|&x| x as i32).collect::<Vec<_>>(), &pred);
    assert!(acc > 0.7, "Accuracy should be > 0.7, got {}", acc);
}

#[test]
fn test_logistic_regression_predict_proba() {
    let X = vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0]];
    let y = vec![0.0, 0.0, 1.0, 1.0];

    let mut model = LogisticRegression::new(0.1, 500);
    model.fit(&X, &y);

    let proba = model.predict_proba(&X);
    assert_eq!(proba.len(), 4);
    for p in &proba {
        assert!(*p >= 0.0 && *p <= 1.0);
    }
}

#[test]
fn test_decision_tree_classification() {
    let X = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0], vec![6.0]];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    let mut tree = DecisionTree::new(3, 2);
    tree.fit(&X, &y);

    let pred = tree.predict(&X);
    let y_int: Vec<i32> = y.iter().map(|&x| x as i32).collect();
    let pred_int: Vec<i32> = pred.iter().map(|&x| x as i32).collect();
    let acc = accuracy_score(&y_int, &pred_int);
    assert!(acc >= 0.8, "Accuracy should be >= 0.8, got {}", acc);
}

#[test]
fn test_decision_tree_regression() {
    let X = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0], vec![5.0]];
    let y = vec![1.0, 1.0, 3.0, 3.0, 5.0];

    let mut tree = DecisionTree::new(3, 2);
    tree.fit(&X, &y);

    let pred = tree.predict(&X);
    let error: f64 = pred.iter().zip(&y).map(|(p, &y)| (p - y).abs()).sum::<f64>() / y.len() as f64;
    assert!(error < 1.0, "Mean abs error should be < 1, got {}", error);
}

#[test]
fn test_random_forest_classification() {
    let mut X = Vec::new();
    let mut y = Vec::new();

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for _ in 0..100 {
        let x0 = rng.gen::<f64>();
        let x1 = rng.gen::<f64>();
        X.push(vec![x0, x1]);
        y.push(if x0 + x1 > 1.0 { 1.0 } else { 0.0 });
    }

    let mut forest = RandomForest::new(10, 10, 2);
    forest.fit(&X, &y);

    let pred = forest.predict(&X);
    let y_int: Vec<i32> = y.iter().map(|&v| v as i32).collect();
    let acc = accuracy_score(&y_int, &pred.iter().map(|&v| v as i32).collect::<Vec<_>>());
    assert!(acc > 0.5, "Accuracy should be > 0.5, got {}", acc);
}

use rand::{Rng, SeedableRng};

#[test]
fn test_kmeans_fit_predict() {
    let mut X1 = Vec::new();
    let mut X2 = Vec::new();

    for _ in 0..30 {
        X1.push(vec![rand::random::<f64>() * 2.0 + 2.0, rand::random::<f64>() * 2.0 + 2.0]);
        X2.push(vec![rand::random::<f64>() * 2.0 - 2.0, rand::random::<f64>() * 2.0 - 2.0]);
    }

    let mut X = X1;
    X.extend(X2);

    let mut kmeans = KMeans::new(2, 300, 5);
    kmeans.fit(&X);

    let labels = kmeans.predict(&X);
    assert_eq!(labels.len(), X.len());

    let mut unique_sorted = labels.clone();
    unique_sorted.sort();
    unique_sorted.dedup();
    assert_eq!(unique_sorted.len(), 2);
}

#[test]
fn test_pca_fit_transform() {
    let mut X = Vec::new();
    for _ in 0..100 {
        X.push(vec![rand::random(), rand::random(), rand::random(), rand::random(), rand::random()]);
    }

    let mut pca = PCA::new(Some(2));
    let X_transformed = pca.fit_transform(&X);

    assert_eq!(X_transformed.len(), 100);
    assert_eq!(X_transformed[0].len(), 2);
    assert!(pca.components.is_some());
}

#[test]
fn test_pca_inverse_transform() {
    let mut X = Vec::new();
    for _ in 0..50 {
        X.push(vec![rand::random(), rand::random(), rand::random(), rand::random()]);
    }

    let mut pca = PCA::new(Some(2));
    let X_transformed = pca.fit_transform(&X);
    let X_reconstructed = pca.inverse_transform(&X_transformed);

    assert_eq!(X_reconstructed.len(), X.len());
    assert_eq!(X_reconstructed[0].len(), 4);
}

#[test]
fn test_standard_scaler() {
    let X = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];

    let mut scaler = StandardScaler::new();
    let X_scaled = scaler.fit_transform(&X);

    for col in 0..2 {
        let mean: f64 = X_scaled.iter().map(|row| row[col]).sum::<f64>() / X_scaled.len() as f64;
        assert!((mean - 0.0).abs() < 1e-10, "Mean should be ~0, got {}", mean);
    }
}

#[test]
fn test_train_test_split_sizes() {
    let mut X = Vec::new();
    let mut y = Vec::new();

    for _ in 0..100 {
        X.push(vec![rand::random(), rand::random(), rand::random()]);
        y.push(rand::random());
    }

    let (X_train, X_test, y_train, y_test) = train_test_split(&X, &y, 0.2, None);

    assert_eq!(X_train.len(), 80);
    assert_eq!(X_test.len(), 20);
    assert_eq!(y_train.len(), 80);
    assert_eq!(y_test.len(), 20);
}

#[test]
fn test_accuracy_score() {
    let y_true = vec![0, 1, 0, 1];
    let y_pred = vec![0, 1, 0, 0];
    assert_eq!(accuracy_score(&y_true, &y_pred), 0.75);

    let y_true2 = vec![0, 1, 2, 0, 1, 2];
    let y_pred2 = vec![0, 1, 2, 0, 0, 2];
    assert_eq!(accuracy_score(&y_true2, &y_pred2), 5.0 / 6.0);
}

#[test]
fn test_mean_squared_error() {
    let y_true = vec![1.0, 2.0, 3.0, 4.0];
    let y_pred = vec![1.1, 2.1, 2.9, 4.1];
    let mse = mean_squared_error(&y_true, &y_pred);
    assert!(mse < 0.1, "MSE should be < 0.1, got {}", mse);
}

#[test]
fn test_r2_score() {
    let y_true = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_pred = vec![1.1, 2.1, 2.9, 4.1, 4.9];
    let r2 = r2_score(&y_true, &y_pred);
    assert!(r2 > 0.95, "R2 should be > 0.95, got {}", r2);
}

#[test]
fn test_confusion_matrix() {
    let y_true = vec![0, 1, 0, 1, 0, 1];
    let y_pred = vec![0, 1, 1, 1, 0, 0];
    let cm = confusion_matrix(&y_true, &y_pred);

    assert_eq!(cm.len(), 2);
    assert_eq!(cm[0].len(), 2);
    assert_eq!(cm[0][0], 2);
    assert_eq!(cm[1][1], 2);
}
