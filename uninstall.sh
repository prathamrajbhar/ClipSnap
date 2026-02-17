#!/bin/bash

# ClipSnap Uninstaller

set -e

# Detect if running as root
if [ "$EUID" -ne 0 ]; then 
  echo "Please run as root (or use sudo): sudo ./uninstall.sh"
  exit 1
fi

echo "🗑️  Uninstalling ClipSnap..."

# 1. Remove Binary
if [ -f "/usr/local/bin/clipsnap" ]; then
    echo "Checking /usr/local/bin/clipsnap... found."
    rm -v "/usr/local/bin/clipsnap"
else
    echo "Binary /usr/local/bin/clipsnap not found."
fi

# 2. Remove Desktop Shortcut
if [ -f "/usr/share/applications/clipsnap.desktop" ]; then
    echo "Checking /usr/share/applications/clipsnap.desktop... found."
    rm -v "/usr/share/applications/clipsnap.desktop"
else
    echo "Desktop file not found."
fi

# 3. Remove System Config
if [ -d "/etc/clipsnap" ]; then
    echo "Checking /etc/clipsnap... found."
    rm -rfv "/etc/clipsnap"
else
    echo "System config directory not found."
fi

echo ""
echo "✅ System-wide uninstallation complete."
echo ""
echo "⚠️  User Data Removal"
echo "To completely remove all data (including clipboard history and user preferences),"
echo "run the following command as your normal user (NOT sudo):"
echo ""
echo "   rm -rf ~/.config/clipboard-capture"
echo ""
