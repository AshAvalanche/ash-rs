#!/bin/bash

# Build Ash CLI for Linux
# Any argument will be passed to cargo build

# The script assumes it is run on a Debian-based Linux distro (e.g. Ubuntu)

# Build Ash CLI for Linux
echo "Building Ash CLI for Linux..."
cargo build "$@"

# Get current version
PACKAGE_VERSION=$(grep '^version =' Cargo.toml | grep -oP '\d+\.\d+\.\d+')

# If any argument passed is '--release', binaries are in 'target/release'
# Otherwise, binaries are in 'target/debug'
if [[ "$*" == *"--release"* ]]; then
  LINUX_TARGET_DIR="target/release"
else
  LINUX_TARGET_DIR="target/debug"
fi

# Create an archive with the Ash CLI binary
## Linux
cd "$LINUX_TARGET_DIR" || exit 1
rm -f "ash-linux-amd64-v$PACKAGE_VERSION.tar.gz"
tar -czf "ash-linux-amd64-v$PACKAGE_VERSION.tar.gz" ash
sha512sum "ash-linux-amd64-v$PACKAGE_VERSION.tar.gz" >"ash-linux-amd64-v$PACKAGE_VERSION.tar.gz.sha512"
cd "$OLDPWD" || exit 1
