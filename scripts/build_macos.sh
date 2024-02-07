#!/bin/bash

# Build Ash CLI for MacOS
# Any argument will be passed to cargo build
# Inspired from https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html

# The script assumes it is run on a Debian-based Linux distro (e.g. Ubuntu)
# and that the following packages are installed: clang, gcc, g++, zlib1g-dev, libmpc-dev, libmpfr-dev, libgmp-dev
# osxcross (see ./scripts/osxcross_setup.sh)

# Environment variables
# OSXCROSS_PATH: path to osxcross installation

# Check that the dependencies are installed
# Dependencies: clang, gcc, g++, zlib1g-dev, libmpc-dev, libmpfr-dev, libgmp-dev
for pkg in clang gcc g++ zlib1g-dev libmpc-dev libmpfr-dev libgmp-dev; do
  if ! dpkg -s $pkg >/dev/null 2>&1; then
    echo "Error: $pkg is not installed. Please install it and try again."
    exit 1
  fi
done

# Add *-apple-darwin targets to rustup
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Check that OSXCROSS_PATH is set
if [ -z "$OSXCROSS_PATH" ]; then
  echo "Error: OSXCROSS_PATH is not set. Please set it and try again."
  exit 1
fi

# Build Ash CLI for Mac x86_64
echo "Building Ash CLI for MacOS x86_64..."
PATH="$OSXCROSS_PATH/target/bin:$PATH" \
  CC=o64-clang \
  CXX=o64-clang++ \
  LIBZ_SYS_STATIC=1 \
  cargo build --target x86_64-apple-darwin "$@"

# Build Ash CLI for Mac aarch64
echo "Building Ash CLI for MacOS aarch64..."
PATH="$OSXCROSS_PATH/target/bin:$PATH" \
  CC=o64-clang \
  CXX=o64-clang++ \
  TARGET_CC="$OSXCROSS_PATH/target/bin/aarch64-apple-darwin21.4-clang" \
  TARGET_AR="$OSXCROSS_PATH/target/bin/aarch64-apple-darwin21.4-ar" \
  LIBZ_SYS_STATIC=1 \
  cargo build --target aarch64-apple-darwin "$@"

# Get current version
PACKAGE_VERSION=$(grep '^version =' Cargo.toml | grep -oP '\d+\.\d+\.\d+-?(alpha|beta|rc)?(.\d+)?')

# If any argument passed is '--release', binaries are in 'target/release'
# Otherwise, binaries are in 'target/debug'
if [[ "$*" == *"--release"* ]]; then
  MAC_AMD_TARGET_DIR="target/x86_64-apple-darwin/release"
  MAC_ARM_TARGET_DIR="target/aarch64-apple-darwin/release"
else
  MAC_AMD_TARGET_DIR="target/x86_64-apple-darwin/debug"
  MAC_ARM_TARGET_DIR="target/aarch64-apple-darwin/debug"
fi

# Create an archive with the Ash CLI binary
## Mac x86_64
cd "$MAC_AMD_TARGET_DIR" || exit 1
rm -f "ash-macos-amd64-v$PACKAGE_VERSION.tar.gz"
tar -czf "ash-macos-amd64-v$PACKAGE_VERSION.tar.gz" ash
sha512sum "ash-macos-amd64-v$PACKAGE_VERSION.tar.gz" >"ash-macos-amd64-v$PACKAGE_VERSION.tar.gz.sha512"
cd "$OLDPWD" || exit 1

## Mac aarch64
cd "$MAC_ARM_TARGET_DIR" || exit 1
rm -f "ash-macos-arm64-v$PACKAGE_VERSION.tar.gz"
tar -czf "ash-macos-arm64-v$PACKAGE_VERSION.tar.gz" ash
sha512sum "ash-macos-arm64-v$PACKAGE_VERSION.tar.gz" >"ash-macos-arm64-v$PACKAGE_VERSION.tar.gz.sha512"
