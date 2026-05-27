set -x
cargo test
cargo run --example linear_models
cargo run --example trees
cargo run --example ensemble
cargo run --example clustering
cargo run --example decomposition
cargo run --example preprocessing
cargo run --example pipeline
