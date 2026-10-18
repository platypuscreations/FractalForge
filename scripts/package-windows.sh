#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds the Windows packages in target/packager/: an .msi installer of the
# app and a portable .zip with the app and the command-line renderer.
# Run in Git Bash on Windows with cargo-packager installed (it downloads the
# WiX Toolset for the .msi itself).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build --release --locked -p fractalforge -p fractalforge-cli
(cd apps/gui && cargo packager --release --formats wix)

version="$(cargo metadata --no-deps --format-version 1 |
  python -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="fractalforge"))')"
name="FractalForge-$version-windows-x86_64"
stage="target/packager/$name"
rm -rf -- "$stage"
mkdir -p "$stage"
cp target/release/fractalforge.exe target/release/fractalforge-cli.exe README.md LICENSE "$stage/"
(cd target/packager && 7z a -tzip "$name.zip" "$name" >/dev/null)
ls -l target/packager/*.msi target/packager/*.zip
