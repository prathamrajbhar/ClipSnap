# Technology Stack Document
## Area Screenshot & Clipboard Manager for Linux

**Version:** 1.0  
**Last Updated:** February 7, 2026

---

## 1. Technology Selection Summary

### 1.1 Recommended Stack (Option A - Rust)

```
Language:           Rust 1.60+
Display Server:     X11 (XCB bindings)
UI Framework:       GTK 4 (gtk-rs)
Image Processing:   image crate (PNG encoding/decoding)
Clipboard:          x11-clipboard / arboard
Database:           rusqlite (SQLite wrapper)
Hotkeys:            rdev / global-hotkey
Notifications:      notify-rust
Config:             serde + toml
Testing:            cargo test + integration tests
Build:              cargo
```

**Rationale:** Memory safety, performance, single binary output, minimal runtime dependencies.

### 1.2 Alternative Stack (Option B - Python)

```
Language:           Python 3.8+
Display Server:     X11 (python-xlib)
UI Framework:       PyQt5 / PySide6
Image Processing:   Pillow (PIL)
Clipboard:          PyQt5.QtGui.QClipboard / pyclip
Database:           sqlite3 (built-in)
Hotkeys:            pynput / keyboard
Notifications:      plyer / notify2
Config:             toml / configparser
Testing:            pytest
Packaging:          PyInstaller / Poetry
```

**Rationale:** Rapid development, easier for contributors, extensive libraries.

### 1.3 Recommendation
**Primary:** Rust (Option A)  
**Reason:** Better performance, memory safety, single binary distribution, lower resource usage.

---

## 2. Core Technologies

### 2.1 Programming Language: Rust

**Crate:** `rustc 1.60+`

**Advantages:**
- Memory safety without garbage collection
- Zero-cost abstractions
- Excellent performance (near C/C++ levels)
- Strong type system prevents common bugs
- Single binary output (no runtime needed)
- Active ecosystem for Linux system programming

**Disadvantages:**
- Steeper learning curve
- Slower initial development
- Fewer GUI framework options than Python

**Key Dependencies:**
```toml
[dependencies]
gtk4 = "0.7"
x11 = "2.21"
xcb = "1.2"
image = "0.24"
rusqlite = { version = "0.30", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
global-hotkey = "0.5"
notify-rust = "4.10"
arboard = "3.3"
chrono = "0.4"
```

---

### 2.2 Display Server & Windowing

#### X11 Support (Phase 1)

**Library:** `xcb` (X C Bindings) + `x11rb`

**Why XCB over Xlib:**
- Modern, maintained API
- Better error handling
- Asynchronous operations
- Smaller memory footprint
- Thread-safe

**Key Modules:**
```rust
xcb::xproto      // Core X11 protocol
xcb::randr       // Multi-monitor support (RandR)
xcb::xfixes      // Selection/clipboard handling
xcb::composite   // Transparent overlay window
xcb::shape       // Irregular window shapes
```

**Functionality:**
- Screen capture via `xcb::shm` (shared memory)
- Overlay window creation with transparency
- Multi-monitor geometry detection
- Clipboard ownership (`CLIPBOARD` and `PRIMARY`)

#### Wayland Support (Phase 3 - Future)

**Approach:** xdg-desktop-portal + PipeWire

**Libraries:**
```toml
ashpd = "0.6"         # Desktop portal client
pipewire = "0.7"      # Screen capture streams
wayland-client = "0.31"
```

**Portals Required:**
- `org.freedesktop.portal.Screenshot`
- `org.freedesktop.portal.Screencast`
- `org.freedesktop.portal.Clipboard`

**Challenges:**
- User permission dialogs (can't bypass)
- No direct pixel access
- Platform inconsistencies

---

### 2.3 UI Framework: GTK 4

**Library:** `gtk4-rs` (Rust bindings for GTK 4)

**Why GTK 4:**
- Native Linux look and feel
- Excellent Wayland support
- Smaller binary size than Qt
- Strong Rust bindings (gtk-rs)
- Built-in accessibility

**Alternative Considered:**
- **Qt5/Qt6:** Larger, more complex, C++ interop
- **Egui:** Immediate mode, less native feel
- **Iced:** Younger ecosystem, fewer widgets

**GTK Components Used:**
```rust
gtk4::Window              // Main history dialog
gtk4::Entry               // Search box
gtk4::ScrolledWindow      // History list container
gtk4::Grid / GridView     // Thumbnail grid
gtk4::Label               // Text previews
gtk4::Image               // Screenshot thumbnails
gtk4::ApplicationWindow   // Overlay window (fullscreen)
```

**Styling:**
- CSS-based theming
- Respect system theme (light/dark)
- Custom CSS for overlay

---

### 2.4 Image Processing

**Library:** `image` crate (v0.24+)

**Features:**
```toml
image = { version = "0.24", features = ["png", "jpeg"] }
```

**Capabilities:**
- PNG encoding/decoding (lossless)
- JPEG support (optional, smaller history entries)
- In-memory image manipulation
- Thumbnail generation
- Format conversion

**Workflow:**
```rust
// Capture raw pixels from X11
let pixels: Vec<u8> = capture_screen_region(x, y, w, h);

// Encode to PNG
let image_buffer = ImageBuffer::from_raw(w, h, pixels)?;
let mut png_bytes = Vec::new();
image_buffer.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;

// Store or copy to clipboard
clipboard.set_image(png_bytes);
```

**Thumbnail Generation:**
```rust
let thumbnail = image::imageops::resize(
    &original_image,
    150, 150,
    image::imageops::FilterType::Lanczos3
);
```

---

### 2.5 Clipboard Management

**Library:** `arboard` (v3.3+)

**Why Arboard:**
- Cross-platform (Linux focus for this project)
- Handles both X11 and Wayland
- Supports images and text
- Simple API
- Maintains clipboard ownership

**Alternative:** `x11-clipboard` (X11-only, lower-level)

**Clipboard Operations:**
```rust
use arboard::{Clipboard, ImageData};

// Copy image to clipboard
let mut clipboard = Clipboard::new()?;
let img_data = ImageData {
    width: image.width(),
    height: image.height(),
    bytes: image.into_raw().into(),
};
clipboard.set_image(img_data)?;

// Copy text to clipboard
clipboard.set_text("example text")?;

// Get clipboard content
if let Ok(text) = clipboard.get_text() {
    // Handle text
}
```

**Clipboard Monitoring:**
```rust
// Poll clipboard for changes (separate thread)
loop {
    let current_content = clipboard.get_text().ok();
    if current_content != last_content {
        store_to_history(current_content);
        last_content = current_content;
    }
    thread::sleep(Duration::from_millis(500));
}
```

**MIME Types Supported:**
- `image/png`
- `text/plain;charset=utf-8`

---

### 2.6 Database: SQLite

**Library:** `rusqlite` (v0.30+)

**Why SQLite:**
- Serverless, embedded
- ACID compliant
- Single file database
- Battle-tested reliability
- Built-in Rust support

**Schema:**

```sql
CREATE TABLE IF NOT EXISTS clipboard_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,  -- 'text' or 'image'
    content_data BLOB,            -- Image PNG bytes or NULL
    text_content TEXT,            -- Text data or NULL
    thumbnail BLOB,               -- 150x150 thumbnail (images only)
    created_at INTEGER NOT NULL,  -- Unix timestamp
    file_size INTEGER,            -- Bytes
    metadata TEXT                 -- JSON for future extensions
);

CREATE INDEX idx_created_at ON clipboard_history(created_at DESC);
CREATE INDEX idx_content_type ON clipboard_history(content_type);
```

**Queries:**
```rust
// Insert image
conn.execute(
    "INSERT INTO clipboard_history (content_type, content_data, thumbnail, created_at, file_size)
     VALUES (?1, ?2, ?3, ?4, ?5)",
    params!["image", png_bytes, thumbnail_bytes, timestamp, png_bytes.len()],
)?;

// Search text history
let mut stmt = conn.prepare(
    "SELECT id, text_content, created_at FROM clipboard_history
     WHERE content_type = 'text' AND text_content LIKE ?1
     ORDER BY created_at DESC LIMIT 50"
)?;
let results = stmt.query_map([format!("%{}%", search_term)], |row| {
    Ok(HistoryEntry { /* ... */ })
})?;
```

**Database Location:**
`~/.config/clipboard-capture/history.db`

**Maintenance:**
```rust
// Auto-cleanup old entries
conn.execute(
    "DELETE FROM clipboard_history
     WHERE created_at < ?1",
    params![cutoff_timestamp],
)?;

// Vacuum database periodically
conn.execute("VACUUM", [])?;
```

---

### 2.7 Global Hotkeys

**Library:** `global-hotkey` (v0.5+)

**Why global-hotkey:**
- Cross-platform abstraction
- X11 and Wayland support
- Simple registration API
- Event-driven architecture

**Implementation:**
```rust
use global_hotkey::{GlobalHotKeyManager, HotKey, HotKeyModifiers, HotKeyCode};

// Register Ctrl+Win+S
let hotkey_manager = GlobalHotKeyManager::new()?;
let screenshot_hotkey = HotKey::new(
    HotKeyModifiers::CTRL | HotKeyModifiers::SUPER,
    HotKeyCode::KeyS,
);
hotkey_manager.register(screenshot_hotkey)?;

// Register Win+H
let history_hotkey = HotKey::new(
    HotKeyModifiers::SUPER,
    HotKeyCode::KeyH,
);
hotkey_manager.register(history_hotkey)?;

// Event loop
for event in hotkey_manager.receiver() {
    match event.id {
        screenshot_hotkey.id => start_area_selection(),
        history_hotkey.id => open_history_dialog(),
        _ => {}
    }
}
```

**Key Mappings:**
```rust
// Config file: "Ctrl+Super+S" → HotKey
fn parse_hotkey(config_str: &str) -> Result<HotKey> {
    // Parse "Ctrl+Super+S" into modifiers + key
    let parts: Vec<&str> = config_str.split('+').collect();
    let mut modifiers = HotKeyModifiers::empty();
    
    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" => modifiers |= HotKeyModifiers::CTRL,
            "super" | "win" => modifiers |= HotKeyModifiers::SUPER,
            "shift" => modifiers |= HotKeyModifiers::SHIFT,
            "alt" => modifiers |= HotKeyModifiers::ALT,
            _ => return Err(/* ... */),
        }
    }
    
    let key = parse_key_code(parts.last().unwrap())?;
    Ok(HotKey::new(modifiers, key))
}
```

---

### 2.8 Notifications

**Library:** `notify-rust` (v4.10+)

**Features:**
- Desktop notifications (libnotify backend)
- Custom icons
- Action buttons (optional)
- Urgency levels

**Implementation:**
```rust
use notify_rust::Notification;

// Success notification
Notification::new()
    .summary("Screenshot Captured")
    .body("Image copied to clipboard")
    .icon("camera-photo")
    .timeout(2000)
    .show()?;

// Error notification
Notification::new()
    .summary("Screenshot Failed")
    .body("Could not capture screen region")
    .icon("dialog-error")
    .urgency(notify_rust::Urgency::Critical)
    .show()?;
```

---

### 2.9 Configuration

**Library:** `serde` + `toml` (v1.0 + v0.8)

**Format:** TOML (human-readable, easy to edit)

**Location:** `~/.config/clipboard-capture/config.toml`

**Parsing:**
```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    shortcuts: Shortcuts,
    capture: CaptureConfig,
    history: HistoryConfig,
    storage: StorageConfig,
    ui: UiConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct Shortcuts {
    screenshot: String,
    history: String,
}

// Load config
let config_path = dirs::config_dir()
    .unwrap()
    .join("clipboard-capture/config.toml");
let config_str = fs::read_to_string(config_path)?;
let config: Config = toml::from_str(&config_str)?;
```

**Default Config Generation:**
```rust
fn create_default_config() -> Config {
    Config {
        shortcuts: Shortcuts {
            screenshot: "Ctrl+Super+S".to_string(),
            history: "Super+H".to_string(),
        },
        capture: CaptureConfig {
            format: "png".to_string(),
            quality: 95,
            show_dimensions: true,
        },
        // ... etc
    }
}
```

---

## 3. Architecture Components

### 3.1 Daemon Process

**Binary:** `clipboard-capture-daemon`

**Responsibilities:**
- Register global hotkeys
- Monitor clipboard changes
- Manage database
- Handle IPC requests
- Run as background service

**Lifecycle:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = load_config()?;
    
    // Initialize database
    let db = Database::new(&config.storage.database_path)?;
    
    // Register hotkeys
    let hotkey_mgr = setup_hotkeys(&config.shortcuts)?;
    
    // Start clipboard monitor
    tokio::spawn(monitor_clipboard(db.clone()));
    
    // Event loop
    loop {
        tokio::select! {
            event = hotkey_mgr.receiver().recv() => {
                handle_hotkey_event(event).await?;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
        }
    }
    
    Ok(())
}
```

**Systemd Service:**
```ini
[Unit]
Description=Clipboard Capture Daemon
After=graphical.target

[Service]
Type=simple
ExecStart=/usr/bin/clipboard-capture-daemon
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
```

---

### 3.2 Screenshot Module

**File:** `src/screenshot.rs`

**Responsibilities:**
- Create transparent overlay window
- Handle mouse input for area selection
- Capture screen region
- Encode to PNG

**Key Functions:**
```rust
pub struct ScreenshotCapture {
    connection: xcb::Connection,
    screen: xcb::Screen,
}

impl ScreenshotCapture {
    pub fn new() -> Result<Self> { /* ... */ }
    
    pub async fn capture_area(&self) -> Result<(u32, u32, u32, u32)> {
        // Show overlay, get user selection
    }
    
    pub fn capture_region(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Vec<u8>> {
        // Get raw pixels from X11
    }
    
    pub fn encode_png(&self, pixels: Vec<u8>, w: u32, h: u32) -> Result<Vec<u8>> {
        // Convert to PNG
    }
}
```

**Overlay Window:**
```rust
fn create_overlay_window(&self) -> Result<xcb::Window> {
    let window = self.connection.generate_id();
    
    xcb::create_window(
        &self.connection,
        self.screen.root_depth(),
        window,
        self.screen.root(),
        0, 0,  // x, y
        self.screen.width_in_pixels(),
        self.screen.height_in_pixels(),
        0,  // border
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        self.screen.root_visual(),
        &[
            (xcb::CW_BACK_PIXEL, 0x000000),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
            (xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_EXPOSURE |
                xcb::EVENT_MASK_BUTTON_PRESS |
                xcb::EVENT_MASK_BUTTON_RELEASE |
                xcb::EVENT_MASK_POINTER_MOTION),
        ],
    );
    
    // Make window transparent
    // Map window (show it)
    xcb::map_window(&self.connection, window);
    
    Ok(window)
}
```

---

### 3.3 Clipboard Manager

**File:** `src/clipboard.rs`

**Responsibilities:**
- Own clipboard after screenshot
- Monitor clipboard changes
- Store history entries

**Implementation:**
```rust
pub struct ClipboardManager {
    clipboard: Clipboard,
    db: Arc<Mutex<Database>>,
    last_content: Arc<Mutex<Option<String>>>,
}

impl ClipboardManager {
    pub fn set_image(&mut self, png_bytes: Vec<u8>) -> Result<()> {
        let img_data = ImageData {
            width: /* from PNG header */,
            height: /* from PNG header */,
            bytes: png_bytes.clone().into(),
        };
        self.clipboard.set_image(img_data)?;
        
        // Store in history
        self.db.lock().unwrap().insert_image(png_bytes)?;
        
        Ok(())
    }
    
    pub fn monitor_clipboard(&self) {
        loop {
            if let Ok(text) = self.clipboard.get_text() {
                let mut last = self.last_content.lock().unwrap();
                if Some(&text) != last.as_ref() {
                    // New clipboard content
                    self.db.lock().unwrap().insert_text(text.clone())?;
                    *last = Some(text);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}
```

---

### 3.4 History Dialog

**File:** `src/ui/history_dialog.rs`

**Responsibilities:**
- Display clipboard history
- Search/filter entries
- Handle user selection

**GTK Structure:**
```rust
pub struct HistoryDialog {
    window: gtk4::ApplicationWindow,
    search_entry: gtk4::Entry,
    grid_view: gtk4::GridView,
    list_store: gio::ListStore,
}

impl HistoryDialog {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Clipboard History")
            .default_width(600)
            .default_height(400)
            .build();
        
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        
        // Search entry
        let search_entry = gtk4::Entry::builder()
            .placeholder_text("Search clipboard...")
            .hexpand(true)
            .build();
        
        // Grid view for thumbnails
        let grid_view = gtk4::GridView::builder()
            .max_columns(4)
            .build();
        
        vbox.append(&search_entry);
        vbox.append(&grid_view);
        window.set_child(Some(&vbox));
        
        Self { window, search_entry, grid_view, list_store }
    }
    
    pub fn load_history(&self, db: &Database) -> Result<()> {
        let entries = db.get_recent_entries(50)?;
        for entry in entries {
            self.list_store.append(&entry);
        }
        Ok(())
    }
}
```

---

## 4. Build & Distribution

### 4.1 Build System: Cargo

**Cargo.toml:**
```toml
[package]
name = "clipboard-capture"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <email@example.com>"]

[dependencies]
# (see section 2.1)

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**Build Commands:**
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

### 4.2 Distribution

**Binary Size Target:** < 20MB

**Packaging Options:**

1. **AppImage** (recommended)
   - Single file, portable
   - No installation required
   - Includes all dependencies
   
2. **Flatpak**
   - Sandboxed
   - Desktop portal integration
   - Flathub distribution

3. **Snap**
   - Ubuntu ecosystem
   - Auto-updates

4. **Traditional Packages**
   - `.deb` for Debian/Ubuntu
   - `.rpm` for Fedora/RHEL
   - AUR for Arch Linux

**Build Script (AppImage):**
```bash
#!/bin/bash
cargo build --release
linuxdeploy --appdir AppDir --executable target/release/clipboard-capture-daemon
appimagetool AppDir clipboard-capture-1.0.0-x86_64.AppImage
```

---

## 5. Testing Strategy

### 5.1 Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_png_encoding() {
        let pixels = vec![255u8; 100 * 100 * 4];
        let png = encode_png(pixels, 100, 100).unwrap();
        assert!(png.len() > 0);
    }
    
    #[test]
    fn test_config_parsing() {
        let config_str = r#"
            [shortcuts]
            screenshot = "Ctrl+Super+S"
        "#;
        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.shortcuts.screenshot, "Ctrl+Super+S");
    }
}
```

### 5.2 Integration Tests
```rust
#[test]
fn test_full_screenshot_workflow() {
    // Start daemon
    // Trigger screenshot hotkey
    // Verify clipboard contains image
    // Verify history database entry
}
```

### 5.3 Manual Testing Checklist
- [ ] Screenshot on single monitor
- [ ] Screenshot on multi-monitor
- [ ] Screenshot with HiDPI scaling
- [ ] Clipboard paste in 10+ applications
- [ ] History dialog search
- [ ] Config file reload
- [ ] Daemon restart persistence

---

## 6. Development Tools

### 6.1 IDE Recommendations
- **VS Code** + rust-analyzer
- **CLion** with Rust plugin
- **Helix** / **Neovim** with LSP

### 6.2 Debugging
```bash
# Debug build with symbols
cargo build

# Run with logging
RUST_LOG=debug cargo run

# GDB debugging
rust-gdb target/debug/clipboard-capture-daemon
```

### 6.3 Profiling
```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bin clipboard-capture-daemon

# Memory profiling
valgrind --tool=massif target/release/clipboard-capture-daemon
```

---

## 7. Alternative Stacks Comparison

| Aspect | Rust | Python | C++ |
|--------|------|--------|-----|
| Performance | Excellent | Good | Excellent |
| Memory Safety | Guaranteed | GC-based | Manual |
| Binary Size | 10-20MB | 50MB+ (PyInstaller) | 5-15MB |
| Dev Speed | Medium | Fast | Slow |
| Dependencies | Cargo | pip/venv | CMake/vcpkg |
| Distribution | Single binary | Bundle/script | Binary + libs |

**Recommendation:** Rust for production, Python for prototyping.

---

**Document Owner:** Engineering Team  
**Next Review:** March 7, 2026
