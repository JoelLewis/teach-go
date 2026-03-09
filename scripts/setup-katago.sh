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
    # Self-hosted Metal build (no official macOS release exists)
    KATAGO_URL="https://github.com/jlewis/gosensei/releases/download/katago-builds/katago"
    BINARY_NAME="katago"
    DIRECT_BINARY=true
    ;;
  *)
    echo "Unknown platform: $PLATFORM"
    exit 1
    ;;
esac

echo "Downloading KataGo v${KATAGO_VERSION} for ${PLATFORM}..."
if [ "${DIRECT_BINARY:-false}" = "true" ]; then
  curl -L -o "${BINDIR}/${BINARY_NAME}" "$KATAGO_URL"
  chmod +x "${BINDIR}/${BINARY_NAME}"
else
  TMPDIR=$(mktemp -d)
  curl -L -o "${TMPDIR}/katago.zip" "$KATAGO_URL"
  unzip -o "${TMPDIR}/katago.zip" -d "${TMPDIR}/katago"
  find "${TMPDIR}/katago" -name "$BINARY_NAME" -exec cp {} "${BINDIR}/${BINARY_NAME}" \;
  chmod +x "${BINDIR}/${BINARY_NAME}"
  rm -rf "$TMPDIR"
fi

echo "Downloading KataGo model..."
curl -L -o "${BINDIR}/model.bin.gz" "$MODEL_URL"

echo "KataGo setup complete!"
echo "  Binary: ${BINDIR}/${BINARY_NAME}"
echo "  Model:  ${BINDIR}/model.bin.gz"
