#!/bin/bash
set -e
cd /Users/Shared/ccc/project/computer4/embed/rvboard4

echo "Building simulator..."
cd simulator
LIBRARY_PATH=/opt/homebrew/lib cargo build --release
echo "Built: simulator/target/release/rvboard4-sim"