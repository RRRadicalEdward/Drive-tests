set -ex

cargo check

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

cargo build
cargo build --release

cargo test 
cargo test --release
