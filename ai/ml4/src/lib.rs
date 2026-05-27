#![allow(non_snake_case, dead_code)]

pub mod linear_models;
pub mod tree;
pub mod ensemble;
pub mod clustering;
pub mod decomposition;
pub mod metrics;
pub mod preprocessing;

pub use linear_models::{LinearRegression, LogisticRegression};
pub use tree::DecisionTree;
pub use ensemble::RandomForest;
pub use clustering::KMeans;
pub use decomposition::PCA;
pub use preprocessing::{StandardScaler, train_test_split};
pub use metrics::{accuracy_score, mean_squared_error, r2_score, confusion_matrix};
