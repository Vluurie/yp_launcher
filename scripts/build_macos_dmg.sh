#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME="YP Launcher"
VERSION=$(grep '^version:' pubspec.yaml | sed 's/version: //' | cut -d+ -f1)
BUILD="build/macos/Build/Products/Release"
APP="$BUILD/$APP_NAME.app"
DMG="build/${APP_NAME// /-}-$VERSION.dmg"

for f in assets/bins/NAMS.exe assets/bins/plugins/yorha_protocol.dll assets/bins/7zz; do
  [[ -f "$f" ]] || { echo "missing $f — see README (macOS build)"; exit 1; }
done

lipo -archs assets/bins/7zz | grep -q arm64 \
  || { echo "assets/bins/7zz is not universal"; exit 1; }
codesign --verify --strict assets/bins/7zz 2>/dev/null \
  || { echo "assets/bins/7zz is not signed — run: codesign --force --sign - assets/bins/7zz"; exit 1; }

flutter clean
flutter pub get
(cd macos && LANG=en_US.UTF-8 pod install)
flutter build macos --release

codesign --force --deep --sign - \
  --entitlements macos/Runner/Release.entitlements "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"

rm -f "$DMG"
hdiutil create -volname "$APP_NAME" -srcfolder "$APP" -ov -format UDZO "$DMG"
codesign --force --sign - "$DMG"

echo "→ $DMG"
