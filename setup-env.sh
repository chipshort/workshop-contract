#!/usr/bin/env sh

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Reload env variables
source $HOME/.cargo/env

# Install wasm
rustup target add wasm32-unknown-unknown

# Install cargo-generate
cargo install cargo-generate

# Might be needed on MacOS?
# xcode-select --install
