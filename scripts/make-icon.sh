#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Regenerates the app icon (apps/gui/assets/icon.png and icon.icns) from a
# FractalForge render. macOS only (uses sips and iconutil).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
assets="$root/apps/gui/assets"
cargo build -q --release -p fractalforge-cli --manifest-path "$root/Cargo.toml"
"$root/target/release/fractalforge-cli" render \
  --center-mag -0.7436447860/0.1318252536/180 --maxiter 1200 \
  --size 1024x1024 --palette ember -o "$assets/icon.png" >/dev/null

set_dir="$(mktemp -d)/FractalForge.iconset"
mkdir -p "$set_dir"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" "$assets/icon.png" --out "$set_dir/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2))
  sips -z "$d" "$d" "$assets/icon.png" --out "$set_dir/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$set_dir" -o "$assets/icon.icns"
echo "wrote $assets/icon.png and icon.icns"
