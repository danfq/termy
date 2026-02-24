#!/usr/bin/env bash
# Generate .icns and .ico files from 1024px PNG for app bundles/installers
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ASSETS_DIR="$PROJECT_ROOT/assets"
ICONSET_DIR="$ASSETS_DIR/termy.iconset"
ICNS_FILE="$ASSETS_DIR/termy.icns"
ICO_FILE="$ASSETS_DIR/termy.ico"
SOURCE_PNG="$ASSETS_DIR/termy_icon@1024px.png"
NORMALIZED_PNG="$ASSETS_DIR/termy_icon@1024px.normalized.png"
MODE="all"

if [ "${1-}" = "--ico-only" ]; then
    MODE="ico-only"
elif [ -n "${1-}" ]; then
    echo "Usage: $0 [--ico-only]"
    exit 2
fi

status=0

if [ ! -f "$SOURCE_PNG" ]; then
    echo "Error: termy_icon@1024px.png not found in assets/"
    exit 1
fi

echo "Generating icon set from 1024px PNG..."

# Normalize source image to improve compatibility with icon tools.
if command -v magick >/dev/null 2>&1; then
    magick "$SOURCE_PNG" -resize 1024x1024\! -colorspace sRGB -depth 8 "$NORMALIZED_PNG"
elif command -v convert >/dev/null 2>&1; then
    convert "$SOURCE_PNG" -resize 1024x1024\! -colorspace sRGB -depth 8 "$NORMALIZED_PNG"
else
    cp "$SOURCE_PNG" "$NORMALIZED_PNG"
fi

echo "Creating .ico file..."
if command -v magick >/dev/null 2>&1; then
    if ! magick "$NORMALIZED_PNG" \
        -background none \
        -define icon:auto-resize=256,128,64,48,32,16 \
        "$ICO_FILE"; then
        echo "Error: Failed to generate .ico"
        status=1
    fi
elif command -v convert >/dev/null 2>&1; then
    if ! convert "$NORMALIZED_PNG" \
        -background none \
        -define icon:auto-resize=256,128,64,48,32,16 \
        "$ICO_FILE"; then
        echo "Error: Failed to generate .ico"
        status=1
    fi
else
    echo "Error: ImageMagick not found (requires 'magick' or 'convert' to generate .ico)"
    status=1
fi

if [ "$MODE" = "all" ]; then
    echo "Creating .icns file..."
    rm -rf "$ICONSET_DIR"
    mkdir -p "$ICONSET_DIR"

    sips -z 16 16     "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_16x16.png"
    sips -z 32 32     "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png"
    sips -z 32 32     "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_32x32.png"
    sips -z 64 64     "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png"
    sips -z 128 128   "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_128x128.png"
    sips -z 256 256   "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png"
    sips -z 256 256   "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_256x256.png"
    sips -z 512 512   "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png"
    sips -z 512 512   "$NORMALIZED_PNG" --out "$ICONSET_DIR/icon_512x512.png"
    cp "$NORMALIZED_PNG" "$ICONSET_DIR/icon_512x512@2x.png"

    if ! iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"; then
        echo "Error: Failed to generate .icns from iconset"
        status=1
    fi

    rm -rf "$ICONSET_DIR"
fi
rm -f "$NORMALIZED_PNG"

if [ -f "$ICO_FILE" ]; then
    echo "Created: $ICO_FILE"
fi
if [ "$MODE" = "all" ] && [ -f "$ICNS_FILE" ]; then
    echo "Created: $ICNS_FILE"
fi

exit "$status"
