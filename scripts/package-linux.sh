#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds the Linux packages in target/packager/: a .deb of the app (for
# Ubuntu 22.04 and later) and a .tar.gz of the command-line renderer.
# Run on Linux with cargo-packager installed.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release --locked -p fractalforge -p fractalforge-cli
(cd apps/gui && cargo packager --release --formats deb)

version="$(cargo metadata --no-deps --format-version 1 |
  python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="fractalforge"))')"
arch="$(uname -m)"
stage="$(mktemp -d)/fractalforge-cli-$version-linux-$arch"
mkdir -p "$stage"
cp target/release/fractalforge-cli README.md LICENSE "$stage/"
tar -C "$(dirname "$stage")" -czf "target/packager/fractalforge-cli-$version-linux-$arch.tar.gz" "$(basename "$stage")"
ls -l target/packager/*.deb target/packager/*.tar.gz
