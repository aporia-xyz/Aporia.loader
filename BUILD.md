# Build Instructions

## Quick Build (Current Platform Only)

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

The binary will be in `target/release/aporia-loader` (or `.exe` on Windows).

---

## Cross-Platform Build

### Prerequisites

#### Option 1: Using `cross` (Recommended for Windows users)

1. Install Docker Desktop: https://www.docker.com/products/docker-desktop
2. Install cross:
   ```bash
   cargo install cross
   ```

#### Option 2: Native toolchains (Advanced)

Install target toolchains:
```bash
# Add targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

Note: Cross-compiling to macOS requires macOS SDK and is complex on non-macOS systems.

---

## Build Commands

### Windows

#### Native build:
```bash
cargo build --release
```

#### Using cross (for Linux/macOS):
```bash
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin
```

#### Using build script:
```bash
# Windows
build.bat

# Git Bash / WSL
bash build.sh
```

---

### Linux

```bash
# Native Linux build
cargo build --release

# Cross-compile to Windows
cargo build --release --target x86_64-pc-windows-gnu

# Cross-compile to macOS (requires osxcross)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Or use build script
chmod +x build.sh
./build.sh
```

---

### macOS

```bash
# Native macOS build (Intel)
cargo build --release

# Native macOS build (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Cross-compile to Windows
cargo build --release --target x86_64-pc-windows-gnu

# Cross-compile to Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Or use build script
chmod +x build.sh
./build.sh
```

---

## Manual Build for Each Platform

### Windows x64
```bash
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/aporia-loader.exe
```

### Linux x64
```bash
cargo build --release --target x86_64-unknown-linux-gnu
# Output: target/x86_64-unknown-linux-gnu/release/aporia-loader
```

### macOS x64 (Intel)
```bash
cargo build --release --target x86_64-apple-darwin
# Output: target/x86_64-apple-darwin/release/aporia-loader
```

### macOS ARM64 (Apple Silicon)
```bash
cargo build --release --target aarch64-apple-darwin
# Output: target/aarch64-apple-darwin/release/aporia-loader
```

---

## GitHub Actions CI/CD

For automated builds on every commit, add `.github/workflows/build.yml`:

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Build
      run: cargo build --release
    - name: Upload artifact
      uses: actions/upload-artifact@v3
      with:
        name: aporia-loader-${{ matrix.os }}
        path: target/release/aporia-loader*
```

---

## Troubleshooting

### OpenSSL errors on Linux
```bash
sudo apt-get install pkg-config libssl-dev
```

### Missing linker on Windows
```bash
# Install MinGW-w64
# Or use MSVC toolchain:
rustup default stable-msvc
```

### macOS cross-compilation issues
- Cross-compiling to macOS from non-macOS is complex
- Recommended: Use GitHub Actions or build on actual macOS hardware
- Alternative: Use osxcross (advanced)

---

## Release Checklist

1. Update version in `Cargo.toml`
2. Run tests: `cargo test`
3. Build for all platforms
4. Test each binary
5. Create GitHub release with binaries
6. Tag release: `git tag v0.2.0 && git push --tags`
