#!/bin/bash

# ClipSnap Robust Installer Script
# Detects package manager, installs dependencies, builds, and installs the app.

set -e

# Define colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}ClipSnap Installer: Starting installation...${NC}"

# Check for git and cargo
if ! command -v git &> /dev/null; then
    echo -e "${RED}Error: 'git' is not installed. Please install git and try again.${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: 'cargo' (Rust) is not installed. Please install Rust (https://rustup.rs/) and try again.${NC}"
    exit 1
fi

# 0. Check if running from within the repository
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    echo -e "${BLUE}Not in a ClipSnap repository. Cloning to temporary directory...${NC}"
    TMP_DIR=$(mktemp -d)
    git clone https://github.com/prathamrajbhar/clipsnap.git "$TMP_DIR"
    cd "$TMP_DIR"
fi

# 1. Detect Package Manager and Install Dependencies
if [ -f /etc/debian_version ]; then
    echo -e "${BLUE}Detected Debian-based system. Installing dependencies via apt...${NC}"
    sudo apt update
    sudo apt install -y build-essential pkg-config libgtk-4-dev libsqlite3-dev \
                        tesseract-ocr tesseract-ocr-eng libtesseract-dev \
                        libcairo2-dev libx11-dev libxrandr-dev libxfixes-dev
elif [ -f /etc/fedora-release ]; then
    echo -e "${BLUE}Detected Fedora-based system. Installing dependencies via dnf...${NC}"
    sudo dnf install -y gcc pkgconf-pkg-config gtk4-devel sqlite-devel \
                        tesseract tesseract-devel libcairo-devel \
                        libX11-devel libXrandr-devel libXfixes-devel
elif [ -f /etc/arch-release ]; then
    echo -e "${BLUE}Detected Arch-based system. Installing dependencies via pacman...${NC}"
    sudo pacman -Sy --needed --noconfirm base-devel gtk4 sqlite \
                                        tesseract tesseract-data-eng \
                                        cairo libx11 libxrandr libxfixes
else
    echo -e "${RED}Unsupported distribution. Please install dependencies manually (GTK4, Tesseract, SQLite, X11).${NC}"
    exit 1
fi

# 2. Build the project
echo -e "${BLUE}Building ClipSnap in release mode...${NC}"
cargo build --release

# 3. Install binary
echo -e "${BLUE}Installing binary to /usr/local/bin/clipsnap...${NC}"
sudo install -m 755 target/release/clipsnap /usr/local/bin/clipsnap

# 4. Install desktop files
echo -e "${BLUE}Installing desktop files...${NC}"
sudo mkdir -p /usr/share/applications/
sudo install -m 644 resources/clipsnap.desktop /usr/share/applications/clipsnap.desktop

sudo mkdir -p /etc/xdg/autostart/
sudo install -m 644 resources/clipsnap-autostart.desktop /etc/xdg/autostart/clipsnap-autostart.desktop

echo -e "${GREEN}ClipSnap Installation Complete!${NC}"
echo -e "You can now find ClipSnap in your application menu or start it manually with 'clipsnap'."
echo -e "ClipSnap will also start automatically on each login."
