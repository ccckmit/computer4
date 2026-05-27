// Complete ML pipeline: split → scale → train → evaluate
use ml4::{
    LinearRegression, LogisticRegression, DecisionTree,
    RandomForest, StandardScaler, train_test_split,
    accuracy_score, mean_squared_error, r2_score, confusion_matrix,
};

fn main() {
    println!("=== Full ML Pipeline ===\n");

    // Generate synthetic dataset
    let mut x = Vec::new();
    let mut y = Vec::new();
    for _ in 0..500 {
        let x0 = rand::random::<f64>() * 100.0;
        let x1 = rand::random::<f64>() * 100.0;
        x.push(vec![x0, x1]);
        y.push(if 3.0 * x0 + 2.0 * x1 > 250.0 { 1.0 } else { 0.0 });
    }

    // 1. Train/test split
    let (x_tr, x_te, y_tr, y_te) = train_test_split(&x, &y, 0.2, Some(42));
    let y_tr_int: Vec<i32> = y_tr.iter().map(|&v| v as i32).collect();
    let y_te_int: Vec<i32> = y_te.iter().map(|&v| v as i32).collect();
    println!("Train: {}  Test: {}", x_tr.len(), x_te.len());

    // 2. Scale
    let mut scaler = StandardScaler::new();
    let x_tr_s = scaler.fit_transform(&x_tr);
    let x_te_s = scaler.transform(&x_te);

    // 3. Train & evaluate multiple models
    // LogisticRegression
    let mut logit = LogisticRegression::new(0.01, 2000);
    logit.fit(&x_tr_s, &y_tr);
    let pred_log = logit.predict(&x_te_s);
    println!(
        "LogisticRegression  accuracy: {:.4}",
        accuracy_score(&y_te_int, &pred_log)
    );
    let cm = confusion_matrix(&y_te_int, &pred_log);
    println!("  Confusion matrix:");
    for row in &cm {
        println!("    {:?}", row);
    }

    // DecisionTree
    let mut tree = DecisionTree::new(8, 10);
    tree.fit(&x_tr_s, &y_tr);
    let pred_tree = tree.predict(&x_te_s);
    let pred_tree_int: Vec<i32> = pred_tree.iter().map(|&v| v as i32).collect();
    println!(
        "DecisionTree         accuracy: {:.4}",
        accuracy_score(&y_te_int, &pred_tree_int)
    );

    // RandomForest
    let mut forest = RandomForest::new(30, 10, 5);
    forest.fit(&x_tr_s, &y_tr);
    let pred_for = forest.predict(&x_te_s);
    let pred_for_int: Vec<i32> = pred_for.iter().map(|&v| v as i32).collect();
    println!(
        "RandomForest         accuracy: {:.4}",
        accuracy_score(&y_te_int, &pred_for_int)
    );

    // Regression variant using LinearRegression
    let x_reg: Vec<Vec<f64>> = (0..200).map(|_| vec![rand::random::<f64>() * 10.0]).collect();
    let y_reg: Vec<f64> = x_reg.iter().map(|r| 2.0 * r[0] + 1.0).collect();

    let (xr_tr, xr_te, yr_tr, yr_te) = train_test_split(&x_reg, &y_reg, 0.25, Some(0));
    let mut scaler_r = StandardScaler::new();
    let xr_tr_s = scaler_r.fit_transform(&xr_tr);
    let xr_te_s = scaler_r.transform(&xr_te);

    let mut lin = LinearRegression::new(0.1, 2000);
    lin.fit(&xr_tr_s, &yr_tr);
    let pred_lin = lin.predict(&xr_te_s);
    println!(
        "\nLinearRegression (regression)  MSE: {:.6}  R²: {:.6}",
        mean_squared_error(&yr_te, &pred_lin),
        r2_score(&yr_te, &pred_lin)
    );
}
