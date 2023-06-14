#!/bin/bash

# Build Ash CLI for Linux and MacOS
# Any argument will be passed to cargo build

# Build Ash CLI for Linux
"$(dirname "${BASH_SOURCE[0]}")/build_linux.sh" "$@"

# Build Ash CLI for Mac
"$(dirname "${BASH_SOURCE[0]}")/build_macos.sh" "$@"
