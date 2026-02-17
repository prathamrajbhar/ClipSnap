#!/bin/bash
# ClipSnap .deb Package Builder

set -e

echo "📦 Building ClipSnap .deb package..."

# Ensure we're in the project root
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Error: Run from project root directory"
    exit 1
fi

# Check for required tools
if ! command -v dpkg-buildpackage >/dev/null 2>&1; then
    echo "❌ Missing build tools. Install with:"
    echo "   sudo apt update && sudo apt install build-essential debhelper"
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Rust not found. Install with:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the package
echo "🔧 Building package..."
dpkg-buildpackage -us -uc -b

# Find and copy the package
DEB_FILE=$(find .. -maxdepth 1 -name "clipsnap_*.deb" -type f | head -1)
if [[ -n "$DEB_FILE" ]]; then
    mkdir -p dist
    cp "$DEB_FILE" dist/
    echo ""
    echo "✅ Package created: dist/$(basename $DEB_FILE)"
    echo ""
    echo "Install with:"
    echo "   sudo apt install ./dist/$(basename $DEB_FILE)"
else
    echo "❌ Package build failed"
    exit 1
fi