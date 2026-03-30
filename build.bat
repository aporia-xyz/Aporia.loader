@echo off
REM Build script for cross-platform compilation on Windows

echo Building Aporia Loader for all platforms...

REM Create builds directory
if not exist builds mkdir builds

REM Windows x64 (native)
echo Building for Windows x64...
cargo build --release
copy target\release\aporia-loader.exe builds\aporia-loader-windows-x64.exe

REM For cross-compilation to Linux/macOS from Windows, you need:
REM 1. Install cross: cargo install cross
REM 2. Install Docker Desktop
REM 3. Use cross instead of cargo

REM Linux x64
echo Building for Linux x64...
cross build --release --target x86_64-unknown-linux-gnu
copy target\x86_64-unknown-linux-gnu\release\aporia-loader builds\aporia-loader-linux-x64

REM macOS x64 (Intel)
echo Building for macOS x64...
cross build --release --target x86_64-apple-darwin
copy target\x86_64-apple-darwin\release\aporia-loader builds\aporia-loader-macos-x64

REM macOS ARM64 (Apple Silicon)
echo Building for macOS ARM64...
cross build --release --target aarch64-apple-darwin
copy target\aarch64-apple-darwin\release\aporia-loader builds\aporia-loader-macos-arm64

echo Build complete! Binaries are in builds\ directory
pause
