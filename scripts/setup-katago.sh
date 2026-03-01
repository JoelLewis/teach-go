#!/usr/bin/env bash
set -euo pipefail

# Downloads KataGo binary and model for local development.
# Usage: ./scripts/setup-katago.sh [platform]
# Platforms: linux, macos, windows

KATAGO_VERSION="1.15.3"
KATAGO_RELEASE_BASE="https://github.com/lightvector/KataGo/releases/download/v${KATAGO_VERSION}"
MODEL_URL="https://media.katagotraining.org/uploaded/networks/models/kata1/kata1-b18c384nbt-s9996604416-d4316597426.bin.gz"

PLATFORM="${1:-$(uname -s | tr '[:upper:]' '[:lower:]')}"
BINDIR="src-tauri/binaries"

mkdir -p "$BINDIR"

case "$PLATFORM" in
  linux)
    KATAGO_URL="${KATAGO_RELEASE_BASE}/katago-v${KATAGO_VERSION}-eigen-linux-x64.zip"
    BINARY_NAME="katago"
    ;;
  darwin|macos)
    KATAGO_URL="${KATAGO_RELEASE_BASE}/katago-v${KATAGO_VERSION}-eigen-osx-x64.zip"
    BINARY_NAME="katago"
    ;;
  windows|mingw*|msys*)
    KATAGO_URL="${KATAGO_RELEASE_BASE}/katago-v${KATAGO_VERSION}-eigen-windows-x64.zip"
    BINARY_NAME="katago.exe"
    ;;
  *)
    echo "Unknown platform: $PLATFORM"
    exit 1
    ;;
esac

echo "Downloading KataGo v${KATAGO_VERSION} for ${PLATFORM}..."
TMPDIR=$(mktemp -d)
curl -L -o "${TMPDIR}/katago.zip" "$KATAGO_URL"
unzip -o "${TMPDIR}/katago.zip" -d "${TMPDIR}/katago"
find "${TMPDIR}/katago" -name "$BINARY_NAME" -exec cp {} "${BINDIR}/${BINARY_NAME}" \;
chmod +x "${BINDIR}/${BINARY_NAME}"

echo "Downloading KataGo model..."
curl -L -o "${BINDIR}/model.bin.gz" "$MODEL_URL"

rm -rf "$TMPDIR"

echo "KataGo setup complete!"
echo "  Binary: ${BINDIR}/${BINARY_NAME}"
echo "  Model:  ${BINDIR}/model.bin.gz"
