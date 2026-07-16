#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep '^version:' pubspec.yaml | sed 's/version: //' | cut -d+ -f1)
BUNDLE="build/linux/x64/release/bundle"
BINARY="yp_launcher"
APPDIR="build/YoRHaProtocol.AppDir"
APPIMAGE="dist/yp_launcher-$VERSION-x86_64.AppImage"
# Static runtime with an extract-and-run fallback, so the AppImage launches
# even where the classic runtime's libfuse2 is missing.
RUNTIME="dist/uruntime-x86_64"
RUNTIME_URL="https://github.com/VHSgunzo/uruntime/releases/download/v0.5.8/uruntime-appimage-squashfs-x86_64"

# appimagetool is itself an AppImage; let it self-extract on FUSE-less hosts.
export APPIMAGE_EXTRACT_AND_RUN=1

command -v appimagetool >/dev/null \
  || { echo "appimagetool not found — install it from https://github.com/AppImage/appimagetool"; exit 1; }

for f in assets/bins/NAMS.exe assets/bins/plugins/yorha_protocol.dll assets/bins/7zz; do
  [[ -f "$f" ]] || { echo "missing $f — see README (Linux build)"; exit 1; }
done

file assets/bins/7zz | grep -q "ELF 64-bit.*x86-64" \
  || { echo "assets/bins/7zz is not a Linux x86-64 ELF"; exit 1; }

flutter clean
flutter pub get
flutter build linux --release

rm -rf "$APPDIR" "$APPIMAGE"
mkdir -p "$APPDIR/usr/bin"
cp -r "$BUNDLE"/. "$APPDIR/usr/bin/"

cp packaging/linux/yp_launcher.desktop "$APPDIR/yp_launcher.desktop"
cp packaging/linux/yp_launcher.png "$APPDIR/yp_launcher.png"
cp packaging/linux/yp_launcher.png "$APPDIR/.DirIcon"

cat > "$APPDIR/AppRun" <<EOF
#!/usr/bin/env bash
here="\$(dirname "\$(readlink -f "\$0")")"
exec "\$here/usr/bin/$BINARY" "\$@"
EOF
chmod +x "$APPDIR/AppRun"

mkdir -p dist
[[ -f "$RUNTIME" ]] || curl -fL -o "$RUNTIME" "$RUNTIME_URL"
chmod +x "$RUNTIME"

ARCH=x86_64 appimagetool --runtime-file "$RUNTIME" "$APPDIR" "$APPIMAGE"
echo "→ $APPIMAGE"
