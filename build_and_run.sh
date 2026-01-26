#!/usr/bin/env bash
set -e

# Step 1: Compile Arduino sketch
arduino-cli compile --fqbn arduino:avr:uno arduino/"$1" --output-dir ./build

# Step 2: Run Rust program
cargo run --example "$1"
