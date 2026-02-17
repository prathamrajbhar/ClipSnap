# ClipSnap

A lightweight screenshot & clipboard history manager for Linux.

**Screenshot** any area → instantly copied to clipboard.  
**Clipboard history** → browse, search, and re-copy previous items.

## Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Alt+S` | Start area screenshot |
| `Alt+H` | Open clipboard history |

## 📦 Installation

### 1. Install dependencies:
```bash
sudo apt update && sudo apt install -y build-essential debhelper cargo rustc pkg-config \
  libgtk-4-dev libgdk-pixbuf-2.0-dev libcairo2-dev \
  libx11-dev libxrandr-dev libsqlite3-dev
```

### 2. Build .deb package:
```bash
./build-deb.sh
```

### 3. Install:
```bash
sudo apt install ./dist/clipsnap_1.0.0-1_amd64.deb
```

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
