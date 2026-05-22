pkill -f "cargo run --example server" 2>/dev/null; sleep 1
cargo run --example server&
sleep 5
cargo run --example client
