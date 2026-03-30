# ClipSnap 📸

**ClipSnap** is a powerful, lightweight, and modern Area Screenshot & Clipboard History Manager designed specifically for Linux. Built with Rust and GTK4, it offers a seamless experience for capturing screens and managing your clipboard history with speed and elegance.

---

## ✨ Features

- **🎯 Precision Area Screenshot**: Capture exactly what you need with an intuitive overlay.
- **📋 Clipboard history**: Automatically tracks text and image copies.
- **🔍 Searchable Database**: Quickly find past clipboard entries.
- **⌨️ Global Hotkeys**: Access features instantly with customizable keyboard shortcuts.
- **🧹 Auto-Cleanup**: Keep your storage lean with automatic history rotation.
- **🔔 Native Notifications**: Get instant feedback on captures and actions.
- **🌑 Modern UI**: Sleek GTK4 interface with dark mode support.

---

## 🛠️ System Requirements

Before installing ClipSnap, ensure your system has the necessary GTK4 and X11 development libraries.

### Debian / Ubuntu / Mint
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev libadwaita-1-dev libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Fedora
```bash
sudo dnf install -y gcc pkg-config gtk4-devel libadwaita-devel libX11-devel libxcb-devel
```

### Arch Linux
```bash
sudo pacman -S --needed base-devel pkgconf gtk4 libadwaita libx11 libxcb
```

---

## 🚀 Installation

### 1. Prerequisite: Rust
If you don't have Rust installed, get it via [rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install (Automated)
Clone the repository and run the installation script:
```bash
git clone https://github.com/yourusername/clipsnap.git
cd clipsnap
make install
```
This will:
- Build the optimized release binary.
- Install it to `~/.local/bin/clipsnap`.
- Set up and start a **systemd user service** so ClipSnap runs in the background and starts automatically on login.

---

## 📖 Usage

### Managing the Service
ClipSnap runs as a background service. You can manage it using `systemctl`:

- **Check Status**: `systemctl --user status clipsnap`
- **Start**: `systemctl --user start clipsnap`
- **Stop**: `systemctl --user stop clipsnap`
- **Restart**: `systemctl --user restart clipsnap`

*Note: Ensure `~/.local/bin` is in your `PATH`.*

### Default Shortcuts
| Action | Shortcut |
| :--- | :--- |
| **Capture Area** | `Ctrl + Alt + S` |
| **Show History** | `Alt + H` |

---

## ⚙️ Configuration

ClipSnap stores its configuration in `~/.config/clipboard-capture/config.toml`. The file is automatically created on the first run.

### Example `config.toml`
```toml
[shortcuts]
screenshot = "Ctrl+Alt+S"
history = "Alt+H"

[capture]
format = "png"
quality = 95
show_dimensions = true

[history]
max_entries = 200
retention_days = 5
auto_cleanup = true

[storage]
database_path = "~/.config/clipboard-capture/history.db"
image_storage = "database"

[ui]
theme = "auto"
thumbnail_size = 150
notification_duration = 2

[privacy]
exclude_passwords = true
```

---

## 🛡️ License
Distributed under the MIT License. See `LICENSE` for more information.

---

## 🤝 Contributing
Contributions are welcome! Please feel free to submit a Pull Request.
