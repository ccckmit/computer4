#!/bin/bash
set -x
cargo build
RUST_BACKTRACE=1 cargo run "$@"
