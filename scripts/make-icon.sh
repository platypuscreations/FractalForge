#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Regenerates the app icons (apps/gui/assets/icon.png, icon-256.png, icon.icns
# and icon.ico) from a
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
sips -z 48 48 "$assets/icon.png" --out "$set_dir/icon_48.png" >/dev/null
iconutil -c icns "$set_dir" -o "$assets/icon.icns"
sips -z 256 256 "$assets/icon.png" --out "$assets/icon-256.png" >/dev/null

# Windows .ico: PNG-compressed entries (supported since Windows Vista).
python3 -I - "$set_dir" "$assets/icon.ico" <<'PY'
import struct, sys
d, out = sys.argv[1], sys.argv[2]
sizes = [16, 32, 48, 256]
pngs = []
for s in sizes:
    name = f"{d}/icon_{s}x{s}.png" if s != 48 else f"{d}/icon_48.png"
    pngs.append(open(name, "rb").read())
head = struct.pack("<HHH", 0, 1, len(sizes))
offset = 6 + 16 * len(sizes)
entries = b""
for s, data in zip(sizes, pngs):
    entries += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(data), offset)
    offset += len(data)
open(out, "wb").write(head + entries + b"".join(pngs))
PY
echo "wrote $assets/icon.png, icon-256.png, icon.icns and icon.ico"
