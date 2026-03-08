#!/usr/bin/env bash
set -euo pipefail

# Builds GoSensei for macOS App Store submission.
#
# Prerequisites:
#   1. Xcode installed with command-line tools
#   2. Apple Developer account enrolled in the Apple Developer Program
#   3. Certificates installed in Keychain:
#      - "3rd Party Mac Developer Application: <Team Name>" (for signing the .app)
#      - "3rd Party Mac Developer Installer: <Team Name>"  (for signing the .pkg)
#   4. KataGo binary downloaded: ./scripts/setup-katago.sh macos
#   5. Icons generated: ./scripts/generate-macos-icons.sh [source-1024x1024.png]
#   6. Node modules installed: npm install
#   7. Rust toolchain installed with macOS targets
#
# Usage:
#   ./scripts/build-macos-appstore.sh
#
# Environment variables:
#   APPLE_SIGNING_IDENTITY   - Signing identity for the .app (default: auto-detect)
#   APPLE_INSTALLER_IDENTITY - Signing identity for the .pkg (default: auto-detect)
#   APPLE_TEAM_ID            - Your Apple Developer Team ID
#   APPLE_PROVISIONING_PROFILE - Path to .provisionprofile (optional)
#
# Output:
#   target/release/bundle/macos/GoSensei.app  — Signed application bundle
#   target/release/bundle/macos/GoSensei.pkg  — Installer package for App Store upload

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# --------------------------------------------------------------------------
# 1. Validate environment
# --------------------------------------------------------------------------
echo "==> Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Install Rust via https://rustup.rs"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "Error: npm not found. Install Node.js via https://nodejs.org"
    exit 1
fi

if ! command -v xcrun &> /dev/null; then
    echo "Error: Xcode command-line tools not found. Run: xcode-select --install"
    exit 1
fi

# Check for icon.icns
if [ ! -f "src-tauri/icons/icon.icns" ]; then
    echo "Warning: icon.icns not found. Generating from existing icon..."
    if command -v sips &> /dev/null && command -v iconutil &> /dev/null; then
        ./scripts/generate-macos-icons.sh
    else
        echo "Error: Cannot generate .icns — sips/iconutil not available (run on macOS)."
        echo "Generate icons first: ./scripts/generate-macos-icons.sh <1024x1024.png>"
        exit 1
    fi
fi

# --------------------------------------------------------------------------
# 2. Determine signing identities
# --------------------------------------------------------------------------
APP_IDENTITY="${APPLE_SIGNING_IDENTITY:-}"
PKG_IDENTITY="${APPLE_INSTALLER_IDENTITY:-}"

if [ -z "$APP_IDENTITY" ]; then
    APP_IDENTITY=$(security find-identity -v -p codesigning | \
        grep "3rd Party Mac Developer Application" | \
        head -1 | \
        sed 's/.*"\(.*\)".*/\1/' || true)
fi

if [ -z "$PKG_IDENTITY" ]; then
    PKG_IDENTITY=$(security find-identity -v -p basic | \
        grep "3rd Party Mac Developer Installer" | \
        head -1 | \
        sed 's/.*"\(.*\)".*/\1/' || true)
fi

if [ -z "$APP_IDENTITY" ]; then
    echo "Warning: No 'App Store' signing identity found."
    echo "  Set APPLE_SIGNING_IDENTITY or install certificates in Keychain."
    echo "  Building unsigned for local testing..."
    SIGN_MODE="unsigned"
else
    echo "  App signing identity: $APP_IDENTITY"
    echo "  Pkg signing identity: ${PKG_IDENTITY:-<not found>}"
    SIGN_MODE="signed"
fi

# --------------------------------------------------------------------------
# 3. Build the frontend
# --------------------------------------------------------------------------
echo ""
echo "==> Building frontend..."
npm run build

# --------------------------------------------------------------------------
# 4. Build the Tauri app (release, no MCP for App Store)
# --------------------------------------------------------------------------
echo ""
echo "==> Building Tauri app (release, App Store target)..."

# Disable MCP plugin for App Store (uses Unix sockets, incompatible with sandbox)
export TAURI_SIGNING_IDENTITY="${APP_IDENTITY:-"-"}"
cargo tauri build --no-default-features -- --release

APP_BUNDLE="target/release/bundle/macos/GoSensei.app"

if [ ! -d "$APP_BUNDLE" ]; then
    echo "Error: App bundle not found at $APP_BUNDLE"
    exit 1
fi

echo "  Built: $APP_BUNDLE"

# --------------------------------------------------------------------------
# 5. Bundle KataGo inside the .app
# --------------------------------------------------------------------------
echo ""
echo "==> Bundling KataGo into app..."

KATAGO_DIR="$APP_BUNDLE/Contents/Resources/katago"
mkdir -p "$KATAGO_DIR"

# Copy KataGo binary if available
if [ -f "src-tauri/binaries/katago" ]; then
    cp "src-tauri/binaries/katago" "$KATAGO_DIR/katago"
    chmod +x "$KATAGO_DIR/katago"
    echo "  Bundled KataGo binary"
fi

# Copy model if available
if ls src-tauri/binaries/*.bin.gz 1>/dev/null 2>&1; then
    cp src-tauri/binaries/*.bin.gz "$KATAGO_DIR/"
    echo "  Bundled KataGo model"
fi

# Copy analysis config if available
if [ -f "src-tauri/binaries/analysis.cfg" ]; then
    cp "src-tauri/binaries/analysis.cfg" "$KATAGO_DIR/"
    echo "  Bundled analysis config"
fi

# --------------------------------------------------------------------------
# 6. Sign the bundle (if identities available)
# --------------------------------------------------------------------------
if [ "$SIGN_MODE" = "signed" ]; then
    echo ""
    echo "==> Signing app bundle..."

    ENTITLEMENTS_APP="src-tauri/entitlements/GoSensei.entitlements"
    ENTITLEMENTS_KATAGO="src-tauri/entitlements/KataGo.entitlements"

    # Sign KataGo helper with inherit entitlements
    if [ -f "$KATAGO_DIR/katago" ]; then
        codesign --force --options runtime \
            --sign "$APP_IDENTITY" \
            --entitlements "$ENTITLEMENTS_KATAGO" \
            "$KATAGO_DIR/katago"
        echo "  Signed KataGo binary"
    fi

    # Sign all frameworks and libraries
    find "$APP_BUNDLE/Contents/Frameworks" -type f -name "*.dylib" 2>/dev/null | while read -r lib; do
        codesign --force --options runtime \
            --sign "$APP_IDENTITY" \
            "$lib"
    done

    # Sign the main app bundle
    codesign --force --options runtime \
        --sign "$APP_IDENTITY" \
        --entitlements "$ENTITLEMENTS_APP" \
        "$APP_BUNDLE"
    echo "  Signed app bundle"

    # Verify signature
    codesign --verify --deep --strict "$APP_BUNDLE"
    echo "  Signature verified"

    # --------------------------------------------------------------------------
    # 7. Create installer .pkg for App Store upload
    # --------------------------------------------------------------------------
    if [ -n "$PKG_IDENTITY" ]; then
        echo ""
        echo "==> Creating installer package..."

        PKG_OUTPUT="target/release/bundle/macos/GoSensei.pkg"

        productbuild \
            --component "$APP_BUNDLE" /Applications \
            --sign "$PKG_IDENTITY" \
            "$PKG_OUTPUT"

        echo "  Created: $PKG_OUTPUT"
        echo ""
        echo "==> Ready for App Store submission!"
        echo "  Upload with: xcrun altool --upload-app -f '$PKG_OUTPUT' -t osx -u <apple-id>"
        echo "  Or use Transporter.app to upload the .pkg"
    else
        echo ""
        echo "Warning: No installer signing identity found."
        echo "  Install '3rd Party Mac Developer Installer' certificate to create .pkg"
    fi
else
    echo ""
    echo "==> Unsigned build complete (for local testing only)."
fi

echo ""
echo "==> Build summary:"
echo "  App bundle: $APP_BUNDLE"
echo "  Signed: $SIGN_MODE"
echo ""
echo "Before submitting to the App Store:"
echo "  1. Ensure you have a 1024x1024 app icon (run generate-macos-icons.sh)"
echo "  2. Create an App Store listing at https://appstoreconnect.apple.com"
echo "  3. Set your APPLE_TEAM_ID environment variable"
echo "  4. Upload the .pkg via Transporter or altool"
