#!/bin/bash

# ClipSnap Installer

set -e

# Detect if running as root
if [ "$EUID" -ne 0 ]; then 
  echo "Please run as root (or use sudo): sudo ./install.sh"
  exit 1
fi

echo "📦 Installing ClipSnap..."

# Determine script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$SCRIPT_DIR"

# Define install paths
BIN_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"
CONFIG_DIR="/etc/clipsnap"

# Check for Build Dependencies
install_build_dependencies() {
    echo "🔍 Checking for build dependencies..."
    if command -v apt-get &> /dev/null; then
        echo "📦 Installing required libraries (GTK4, Cairo, etc.)..."
        apt-get update -qq
        apt-get install -y build-essential pkg-config libgtk-4-dev libgdk-pixbuf-2.0-dev \
            libcairo2-dev libx11-dev libxrandr-dev libsqlite3-dev
        
        # Install Rust if missing
        if ! command -v cargo &> /dev/null; then
            echo "🦀 Installing Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source $HOME/.cargo/env
        fi
    elif command -v dnf &> /dev/null; then
        echo "📦 Installing dependencies for Fedora/RHEL..."
        dnf install -y gtk4-devel gdk-pixbuf2-devel cairo-devel libX11-devel libXrandr-devel sqlite-devel cargo
    elif command -v pacman &> /dev/null; then
        echo "📦 Installing dependencies for Arch..."
        pacman -Sy --noconfirm gtk4 gdk-pixbuf2 cairo libx11 libxrandr sqlite rust
    else
        echo "⚠️  Could not detect package manager. Please manually install GTK4 and Rust."
    fi
}

# Locate Binary
BINARY_SOURCE=""

if [ -f "./clipsnap" ]; then
    echo "✅ Found pre-built binary in current directory."
    BINARY_SOURCE="./clipsnap"
elif [ -f "target/release/clipsnap" ]; then
    echo "✅ Found binary in target/release/."
    BINARY_SOURCE="target/release/clipsnap"
else
    echo "⚠️  Binary not found. Attempting to build from source..."
    install_build_dependencies
    
    if command -v cargo &> /dev/null; then
        echo "🔨 Building with Cargo..."
        cargo build --release
        BINARY_SOURCE="target/release/clipsnap"
    else
        echo "❌ Error: Cargo is not installed and automatic installation failed."
        exit 1
    fi
fi

# Install Binary
echo "📂 Moving binary to $BIN_DIR..."
install -m 755 "$BINARY_SOURCE" "$BIN_DIR/clipsnap"

# Install Desktop File
if [ -f "resources/clipsnap.desktop" ]; then
    echo "🖥️  Installing desktop shortcut..."
    install -m 644 "resources/clipsnap.desktop" "$DESKTOP_DIR/"
else
    echo "⚠️  Desktop file not found in resources/, skipping."
fi

# Install Config
mkdir -p "$CONFIG_DIR"
if [ -f "resources/default_config.toml" ]; then
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        echo "⚙️  Installing default configuration..."
        install -m 644 "resources/default_config.toml" "$CONFIG_DIR/config.toml"
    else
        echo "   Config exists, skipping."
    fi
else
    echo "⚠️  Config file not found in resources/, skipping."
fi

echo "✅ Installation Complete! You can now run 'clipsnap' from anywhere."
