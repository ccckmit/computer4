#!/bin/bash
set -x

cargo build

cargo run --example adder
cargo run --example counter
cargo run --example mux

cargo test