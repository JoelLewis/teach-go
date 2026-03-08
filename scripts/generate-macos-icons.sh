#!/usr/bin/env bash
set -euo pipefail

# Generates macOS .icns icon set from a source PNG (1024x1024 recommended).
# Usage: ./scripts/generate-macos-icons.sh [source.png]
#
# Requires: sips (macOS built-in) and iconutil (macOS built-in)

SOURCE="${1:-src-tauri/icons/128x128@2x.png}"
ICONSET_DIR="src-tauri/icons/icon.iconset"
OUTPUT="src-tauri/icons/icon.icns"

if [ ! -f "$SOURCE" ]; then
    echo "Error: Source image not found: $SOURCE"
    echo "Provide a 1024x1024 PNG as the first argument."
    exit 1
fi

echo "Generating macOS icon set from $SOURCE..."

mkdir -p "$ICONSET_DIR"

# Generate all required icon sizes
sips -z 16 16     "$SOURCE" --out "$ICONSET_DIR/icon_16x16.png"      > /dev/null
sips -z 32 32     "$SOURCE" --out "$ICONSET_DIR/icon_16x16@2x.png"  > /dev/null
sips -z 32 32     "$SOURCE" --out "$ICONSET_DIR/icon_32x32.png"      > /dev/null
sips -z 64 64     "$SOURCE" --out "$ICONSET_DIR/icon_32x32@2x.png"  > /dev/null
sips -z 128 128   "$SOURCE" --out "$ICONSET_DIR/icon_128x128.png"    > /dev/null
sips -z 256 256   "$SOURCE" --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null
sips -z 256 256   "$SOURCE" --out "$ICONSET_DIR/icon_256x256.png"    > /dev/null
sips -z 512 512   "$SOURCE" --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null
sips -z 512 512   "$SOURCE" --out "$ICONSET_DIR/icon_512x512.png"    > /dev/null
sips -z 1024 1024 "$SOURCE" --out "$ICONSET_DIR/icon_512x512@2x.png" > /dev/null

# Convert iconset to .icns
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT"

# Clean up iconset directory
rm -rf "$ICONSET_DIR"

echo "Generated: $OUTPUT"
echo ""
echo "NOTE: For best quality, provide a 1024x1024 source PNG."
echo "The current source is $(sips -g pixelWidth -g pixelHeight "$SOURCE" 2>/dev/null | grep pixel | awk '{print $2}' | paste -sd'x' -)"
