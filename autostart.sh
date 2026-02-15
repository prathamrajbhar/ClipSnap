#!/bin/bash

# ClipSnap Installation & Autostart Setup Script
# Optimized for production readiness

set -e

echo "🚀 Starting ClipSnap installation & autostart setup..."

# 1. Kill any existing instances to avoid hotkey conflicts
echo "🛑 Stopping any running instances of ClipSnap..."
pkill clipsnap || true

# 2. Build optimized release binary
echo "🛠️ Building optimized release binary (this may take a minute)..."
cargo build --release

# 3. Ensure installation directory exists
mkdir -p "$HOME/.local/bin"

# 4. Install binary
echo "📦 Installing ClipSnap to $HOME/.local/bin/..."
cp target/release/clipsnap "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/clipsnap"

# 5. Ensure autostart directory exists
mkdir -p "$HOME/.config/autostart"

# 6. Create .desktop file for autostart
echo "🔄 Setting up autostart entry..."
cat > "$HOME/.config/autostart/clipsnap.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=ClipSnap
Comment=Optimized Keyboard & Clipboard Manager
Exec=$HOME/.local/bin/clipsnap
Icon=accessories-clipboard
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
StartupNotify=false
EOF

# 7. Launch ClipSnap now
echo "🚀 Launching ClipSnap in the background..."
"$HOME/.local/bin/clipsnap" > /dev/null 2>&1 &

echo ""
echo "✅ Installation & Setup complete!"
echo "✨ ClipSnap is now running in the background."
echo "✨ Binary: $HOME/.local/bin/clipsnap"
echo "✨ Autostart: $HOME/.config/autostart/clipsnap.desktop"
echo ""
echo "Shortcut reminders:"
echo "📸 Screen Capture: Ctrl+Alt+S"
echo "📜 History Dialog: Alt+H"
echo ""
echo "Note: If it doesn't seem to respond, ensure no other app is using these hotkeys."
