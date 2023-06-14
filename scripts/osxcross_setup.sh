#!/bin/bash

# Install osxcross (run the script where you want to install osxcross)

# Environment variables
## Using the same version as Avalanche CLI by default
## See https://github.com/ava-labs/avalanche-cli/blob/5b8e17abceabaffeca38a3c8fcc43fbc2ab9eb7c/.github/workflows/release.yml#L76
MACOSX_SDK_VERSION=${MACOSX_SDK_VERSION:-12.3}
MACOSX_SDK_CHECKSUM="${MACOSX_SDK_CHECKSUM:-3abd261ceb483c44295a6623fdffe5d44fc4ac2c872526576ec5ab5ad0f6e26c}"

git clone https://github.com/tpoechtrager/osxcross
cd osxcross || exit 1
wget -nc "https://github.com/joseluisq/macosx-sdks/releases/download/${MACOSX_SDK_VERSION}/MacOSX${MACOSX_SDK_VERSION}.sdk.tar.xz"
echo "${MACOSX_SDK_CHECKSUM}  MacOSX${MACOSX_SDK_VERSION}.sdk.tar.xz" | sha256sum -c -
mv "MacOSX${MACOSX_SDK_VERSION}.sdk.tar.xz" tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
