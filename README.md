# ClipSnap

A lightweight screenshot & clipboard history manager for Linux (X11).

**Screenshot** any area → instantly copied to clipboard.  
**Clipboard history** → browse, search, and re-copy previous items.

## Shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `Ctrl+Alt+S` | Start area screenshot | Global |
| `Alt+H` | Open clipboard history | Global |
| `Click + Drag` | Select capture area | Screenshot Overlay |
| `Release` | Confirm capture | Screenshot Overlay |
| `Click` | Restore item to clipboard | History Dialog |
| `Esc` | Cancel / close window | Any Context |

## How to Use

1. **Start**: Run `clipsnap`. A notification will confirm it's active in the background.
2. **Capture**: Press `Ctrl+Alt+S`. The screen will dim, and a crosshair cursor will appear. 
   - Click and drag to select the area you want to capture.
   - Release the mouse button to instantly copy the selection to your clipboard and save it to history.
   - Press `Esc` if you want to cancel the capture.
3. **History**: Press `Alt+H` to browse your clipboard history.
   - The window shows previews of both text and images.
   - Use the **Search** bar to filter text entries (matches as you type).
   - **Click** any item to restore it to your clipboard and close the dialog.
   - Press `Esc` to close the history window without selecting anything.

---

## Install

### 1. Install dependencies

**Ubuntu / Debian:**
```bash
sudo apt install libgtk-4-dev libgraphene-1.0-dev libx11-dev libxcb1-dev \
  libxcb-randr0-dev libdbus-1-dev pkg-config
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libX11-devel libxcb-devel sqlite-devel dbus-devel pkg-config
```

**Arch:**
```bash
sudo pacman -S gtk4 libx11 libxcb sqlite dbus pkgconf
```

### 2. Install Rust (if not already)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. Build & install

```bash
cd clipboard-capture
cargo build --release
sudo cp target/release/clipsnap /usr/local/bin/
```

### 4. Run

```bash
clipsnap
```

A notification confirms it's running. Use the shortcuts above.

---

## Autostart (optional)

Run ClipSnap automatically on login:

```bash
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/clipsnap.service << 'EOF'
[Unit]
Description=ClipSnap
After=graphical-session.target

[Service]
ExecStart=/usr/local/bin/clipsnap
Restart=on-failure

[Install]
WantedBy=default.target
EOF

systemctl --user enable --now clipsnap.service
```

---

## Configuration

Config is auto-created at `~/.config/clipboard-capture/config.toml` on first run.

Key options you can change:

```toml
[shortcuts]
screenshot = "Ctrl+Alt+S"
history = "Alt+H"

[history]
max_entries = 500
retention_days = 7
```

Restart ClipSnap after editing config.

---

## Uninstall

```bash
systemctl --user disable --now clipsnap.service 2>/dev/null
sudo rm /usr/local/bin/clipsnap
rm -rf ~/.config/clipboard-capture
rm -f ~/.config/systemd/user/clipsnap.service
```

## License

MIT
