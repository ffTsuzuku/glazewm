#!/usr/bin/env bash
set -e

VERSION="${1:-0.0.0}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Building GlazeWM in release mode..."
source "$HOME/.cargo/env" 2>/dev/null || true
cargo build --release --bin glazewm --bin glazewm-cli

OUT_DIR="$ROOT_DIR/out"
TEMP_DIR="$ROOT_DIR/temp"
rm -rf "$TEMP_DIR" "$OUT_DIR"
mkdir -p "$TEMP_DIR" "$OUT_DIR"

echo "==> Creating macOS ICNS icon..."
ICONSET_DIR="$TEMP_DIR/icon.iconset"
mkdir -p "$ICONSET_DIR"
sips -z 16 16     resources/assets/icon.png --out "$ICONSET_DIR/icon_16x16.png" >/dev/null 2>&1
sips -z 32 32     resources/assets/icon.png --out "$ICONSET_DIR/icon_16x16@2x.png" >/dev/null 2>&1
sips -z 32 32     resources/assets/icon.png --out "$ICONSET_DIR/icon_32x32.png" >/dev/null 2>&1
sips -z 64 64     resources/assets/icon.png --out "$ICONSET_DIR/icon_32x32@2x.png" >/dev/null 2>&1
sips -z 128 128   resources/assets/icon.png --out "$ICONSET_DIR/icon_128x128.png" >/dev/null 2>&1
sips -z 256 256   resources/assets/icon.png --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null 2>&1
sips -z 256 256   resources/assets/icon.png --out "$ICONSET_DIR/icon_256x256.png" >/dev/null 2>&1
sips -z 512 512   resources/assets/icon.png --out "$ICONSET_DIR/icon_256x256@2x.png" >/dev/null 2>&1
sips -z 512 512   resources/assets/icon.png --out "$ICONSET_DIR/icon_512x512.png" >/dev/null 2>&1
sips -z 1024 1024 resources/assets/icon.png --out "$ICONSET_DIR/icon_512x512@2x.png" >/dev/null 2>&1
iconutil -c icns "$ICONSET_DIR" -o "$TEMP_DIR/icon.icns"

echo "==> Constructing GlazeWM.app bundle..."
APP_DIR="$TEMP_DIR/GlazeWM.app"
CONTENTS_DIR="$APP_DIR/Contents"
mkdir -p "$CONTENTS_DIR/MacOS" "$CONTENTS_DIR/Resources"

cp "target/release/glazewm" "$CONTENTS_DIR/MacOS/"
cp "target/release/glazewm-cli" "$CONTENTS_DIR/MacOS/"
cp "$TEMP_DIR/icon.icns" "$CONTENTS_DIR/Resources/icon.icns"
chmod +x "$CONTENTS_DIR/MacOS"/*

sed "s/\${VERSION}/$VERSION/g" resources/Info.plist > "$CONTENTS_DIR/Info.plist"
echo -n "APPL????" > "$CONTENTS_DIR/PkgInfo"

echo "==> Ad-hoc signing GlazeWM.app..."
codesign --force --deep --sign - "$APP_DIR"

echo "==> Building DMG installer..."
DMG_DIR="$TEMP_DIR/dmg-contents"
mkdir -p "$DMG_DIR"
cp -R "$APP_DIR" "$DMG_DIR/"
ln -s /Applications "$DMG_DIR/Applications"

DMG_NAME="GlazeWM-$VERSION.dmg"
hdiutil create -volname "GlazeWM" \
  -srcfolder "$DMG_DIR" \
  -ov -format UDZO \
  "$OUT_DIR/$DMG_NAME" >/dev/null

cp -R "$APP_DIR" "$OUT_DIR/"
rm -rf "$TEMP_DIR"

echo "==> Packaging complete!"
echo "    App bundle: $OUT_DIR/GlazeWM.app"
echo "    Installer:  $OUT_DIR/$DMG_NAME"
