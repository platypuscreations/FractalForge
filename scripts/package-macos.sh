#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Builds a universal (Apple silicon + Intel) FractalForge.app and .dmg in
# target/packager/.
#
# Signing and notarization happen when these are set (see cargo-packager):
#   APPLE_SIGNING_IDENTITY   e.g. "Developer ID Application: Name (TEAMID)"
#   APPLE_ID, APPLE_PASSWORD (app-specific password), APPLE_TEAM_ID
# Without them the app gets an ad-hoc signature: it runs on this Mac, but
# other Macs' Gatekeeper will refuse it until it's notarized.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

for t in aarch64-apple-darwin x86_64-apple-darwin; do
  rustup target add "$t" >/dev/null
  cargo build --release -p fractalforge --target "$t"
done
out="$root/target/universal-apple-darwin/release"
mkdir -p "$out"
lipo -create -output "$out/fractalforge" \
  target/aarch64-apple-darwin/release/fractalforge \
  target/x86_64-apple-darwin/release/fractalforge
lipo -info "$out/fractalforge"

cd apps/gui
cargo packager --release --formats app

app="$root/target/packager/FractalForge.app"
# cargo-packager copies the host-architecture binary; use the universal one.
cp "$out/fractalforge" "$app/Contents/MacOS/fractalforge"
if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
  codesign --force --deep --sign - "$app"
  echo "note: ad-hoc signed (set APPLE_SIGNING_IDENTITY to sign for distribution)"
fi
codesign --verify --deep --strict "$app" && echo "signature OK: $app"

# Disk image: the app plus a link to /Applications. Built with hdiutil
# directly (no Finder scripting), so it works without a logged-in session.
version="$(cargo metadata --no-deps --format-version 1 --manifest-path "$root/Cargo.toml" |
  python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="fractalforge"))')"
stage="$(mktemp -d)"
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
dmg="$root/target/packager/FractalForge-$version-macos-universal.dmg"
hdiutil create -quiet -volname FractalForge -srcfolder "$stage" -ov -format UDZO "$dmg"
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  codesign --sign "$APPLE_SIGNING_IDENTITY" "$dmg"
fi
echo "disk image: $dmg"
