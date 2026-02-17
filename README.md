# 📸 ClipSnap

![License](https://img.shields.io/badge/License-MIT-purple.svg?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-Linux-blue.svg?style=flat-square)
![Rust](https://img.shields.io/badge/Built%20With-Rust-orange.svg?style=flat-square)

**A professional, lightweight screenshot & clipboard history manager for Linux.**

ClipSnap seamlessly integrates into your workflow, allowing you to capture specific screen areas instantly and manage your clipboard history with a clean, modern interface.

---

## ✨ Features

- **🎯 Precision Capture**: Select any screen area to capture.
- **📋 Clipboard History**: Access, search, and restore past clipboard items.
- **🚀 Instant Access**: Global hotkeys for minimal friction.
- **🎨 Modern UI**: Clean interface built with GTK4.
- **💾 Auto-Save**: History is persistent across reboots (SQLite).

---

## 🚀 Quick Install (Clone & Run)

The easiest way to get started. This script detects your distro, installs dependencies, builds the app, and sets up shortcuts.

```bash
# 1. Clone the repository
git clone https://github.com/prathamrajbhar/ClipSnap.git
cd ClipSnap

# 2. Run the installer
chmod +x install.sh
sudo ./install.sh
```

---

## 🛠️ Manual Installation

If you prefer full control, you can build and install manually using the commands below.

**Prerequisites:** `cargo`, `rustc`, `libgtk-4-dev`, `libgdk-pixbuf-2.0-dev`, `libcairo2-dev`, `libx11-dev`

```bash
# 1. Install Dependencies (Ubuntu/Debian)
sudo apt update && sudo apt install -y build-essential pkg-config libgtk-4-dev \
    libgdk-pixbuf-2.0-dev libcairo2-dev libx11-dev libxrandr-dev libsqlite3-dev

# 2. Build the Project
cargo build --release

# 3. Install Binary & Assets
sudo cp target/release/clipsnap /usr/local/bin/
sudo cp resources/clipsnap.desktop /usr/share/applications/
sudo mkdir -p /etc/clipsnap
sudo cp resources/default_config.toml /etc/clipsnap/config.toml

echo "✅ Installation Complete!"
```

---

## ⌨️ Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Alt+S` | 📸 Take Area Screenshot |
| `Alt+H` | 📋 Open Clipboard History |

---

## 🗑️ Uninstall

To remove ClipSnap completely:

```bash
sudo rm /usr/local/bin/clipsnap
sudo rm /usr/share/applications/clipsnap.desktop
sudo rm -rf /etc/clipsnap
```

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).

Made with ❤️ by [Pratham Rajbhar](https://github.com/prathamrajbhar)
