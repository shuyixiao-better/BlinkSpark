#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ASSET_DIR="$ROOT_DIR/assets/branding/generated"
SOURCE_PNG="$ASSET_DIR/blinkspark-icon-1024.png"
ICONSET_DIR="$ASSET_DIR/blinkspark.iconset"
OUTPUT_ICNS="$ASSET_DIR/blinkspark.icns"

if ! command -v sips >/dev/null 2>&1; then
  echo "[ERROR] Missing tool: sips (install Xcode Command Line Tools: xcode-select --install)" >&2
  exit 1
fi

if ! command -v iconutil >/dev/null 2>&1; then
  echo "[ERROR] Missing tool: iconutil (install Xcode Command Line Tools: xcode-select --install)" >&2
  exit 1
fi

if [ ! -f "$SOURCE_PNG" ]; then
  echo "[ERROR] Missing source icon: $SOURCE_PNG" >&2
  exit 1
fi

rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

make_icon() {
  local size="$1"
  local file="$2"
  sips -z "$size" "$size" "$SOURCE_PNG" --out "$ICONSET_DIR/$file" >/dev/null
}

make_icon 16 icon_16x16.png
make_icon 32 icon_16x16@2x.png
make_icon 32 icon_32x32.png
make_icon 64 icon_32x32@2x.png
make_icon 128 icon_128x128.png
make_icon 256 icon_128x128@2x.png
make_icon 256 icon_256x256.png
make_icon 512 icon_256x256@2x.png
make_icon 512 icon_512x512.png
make_icon 1024 icon_512x512@2x.png

iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"
rm -rf "$ICONSET_DIR"

echo "[OK] Generated macOS icon: $OUTPUT_ICNS"
