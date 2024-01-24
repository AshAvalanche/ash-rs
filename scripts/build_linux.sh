#!/bin/bash

# Build Ash CLI for Linux
# Any argument will be passed to cargo build

# The script assumes it is run on a Debian-based Linux distro (e.g. Ubuntu) on an x86_64 (amd64) machine
# and that the following packages are installed: gcc-aarch64-linux-gnu

# Add aaarch64-unknown-linux-gnu target to rustup
rustup target add aarch64-unknown-linux-gnu

# Build Ash CLI for Linux x86_64
echo "Building Ash CLI for Linux x86_64..."
cargo build "$@"

# Build Ash CLI for Linux aarch64
echo "Building Ash CLI for Linux aarch64..."
cargo build --target aarch64-unknown-linux-gnu "$@"

# Get current version
PACKAGE_VERSION=$(grep '^version =' Cargo.toml | grep -oP '\d+\.\d+\.\d+-?(alpha|beta|rc)?(.\d+)?')

# If any argument passed is '--release', binaries are in 'target/release'
# Otherwise, binaries are in 'target/debug'
if [[ "$*" == *"--release"* ]]; then
  LINUX_AMD_TARGET_DIR="target/release"
  LINUX_ARM_TARGET_DIR="target/aarch64-unknown-linux-gnu/release"
else
  LINUX_AMD_TARGET_DIR="target/debug"
  LINUX_ARM_TARGET_DIR="target/aarch64-unknown-linux-gnu/debug"
fi

# Create an archive with the Ash CLI binary
## Linux x86_64
cd "$LINUX_AMD_TARGET_DIR" || exit 1
rm -f "ash-linux-amd64-v$PACKAGE_VERSION.tar.gz"
tar -czf "ash-linux-amd64-v$PACKAGE_VERSION.tar.gz" ash
sha512sum "ash-linux-amd64-v$PACKAGE_VERSION.tar.gz" >"ash-linux-amd64-v$PACKAGE_VERSION.tar.gz.sha512"
cd "$OLDPWD" || exit 1

## Linux aarch64
cd "$LINUX_ARM_TARGET_DIR" || exit 1
rm -f "ash-linux-arm64-v$PACKAGE_VERSION.tar.gz"
tar -czf "ash-linux-arm64-v$PACKAGE_VERSION.tar.gz" ash
sha512sum "ash-linux-arm64-v$PACKAGE_VERSION.tar.gz" >"ash-linux-arm64-v$PACKAGE_VERSION.tar.gz.sha512"
