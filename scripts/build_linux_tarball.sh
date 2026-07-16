#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep '^version:' pubspec.yaml | sed 's/version: //' | cut -d+ -f1)
BUNDLE="build/linux/x64/release/bundle"
STAGE="build/yp_launcher-linux-x64"
TARBALL="dist/yp_launcher-$VERSION-linux-x64.tar.gz"

for f in assets/bins/NAMS.exe assets/bins/plugins/yorha_protocol.dll assets/bins/7zz; do
  [[ -f "$f" ]] || { echo "missing $f — see README (Linux build)"; exit 1; }
done

file assets/bins/7zz | grep -q "ELF 64-bit.*x86-64" \
  || { echo "assets/bins/7zz is not a Linux x86-64 ELF"; exit 1; }

flutter clean
flutter pub get
flutter build linux --release

rm -rf "$STAGE" "$TARBALL"
cp -r "$BUNDLE" "$STAGE"
cp packaging/linux/yp_launcher.desktop packaging/linux/yp_launcher.png "$STAGE/"

mkdir -p dist
tar -C build -czf "$TARBALL" "$(basename "$STAGE")"
echo "→ $TARBALL"
