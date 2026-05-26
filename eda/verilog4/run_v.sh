set -x
export RUST_BACKTRACE=1
cargo run -- verilog/$1.v verilog/$1.rhdl
cargo run -- verilog/$1.rhdl
