#!/bin/bash
# Build script for cross-platform compilation

set -e

echo "Building Aporia Loader for all platforms..."

# Windows x64
echo "Building for Windows x64..."
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/aporia-loader.exe builds/aporia-loader-windows-x64.exe

# Linux x64
echo "Building for Linux x64..."
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/aporia-loader builds/aporia-loader-linux-x64

# macOS x64 (Intel)
echo "Building for macOS x64..."
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/aporia-loader builds/aporia-loader-macos-x64

# macOS ARM64 (Apple Silicon)
echo "Building for macOS ARM64..."
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/aporia-loader builds/aporia-loader-macos-arm64

echo "Build complete! Binaries are in builds/ directory"
