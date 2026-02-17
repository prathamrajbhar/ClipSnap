# ClipSnap

A lightweight screenshot & clipboard history manager for Linux.

**Screenshot** any area → instantly copied to clipboard.  
**Clipboard history** → browse, search, and re-copy previous items.

## Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Alt+S` | Start area screenshot |
| `Alt+H` | Open clipboard history |

## 🚀 Quick Install (Clone & Run)

You can install ClipSnap directly from the terminal. This script will automatically install dependencies, build the project, and set up the desktop shortcut.

```bash
# 1. Clone the repository
git clone https://github.com/prathamrajbhar/ClipSnap.git
cd ClipSnap

# 2. Run the installer
chmod +x install.sh
sudo ./install.sh
```

The installer supports **Debian/Ubuntu**, **Fedora**, and **Arch Linux** based systems.

### Manual Installation (Advanced)
If you prefer to build manually:
1. Install dependencies: `libgtk-4-dev`, `libgdk-pixbuf-2.0-dev`, `libcairo2-dev`.
2. Run `cargo build --release`.
3. Copy `target/release/clipsnap` to `/usr/local/bin/`.
4. Copy `resources/clipsnap.desktop` to `/usr/share/applications/`.

## ✅ Features

✅ **Automatic startup** - Starts on login  
✅ **System integration** - Proper package installation  
✅ **Clean uninstall** - `sudo apt remove --purge clipsnap`  

---

## 🗑️ Uninstall

```bash
sudo apt remove --purge clipsnap
```

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
