# AI Agent Build Prompt
## Area Screenshot & Clipboard Manager for Linux

**Target Agent:** Advanced coding AI (GPT-4, Claude, etc.)  
**Expected Output:** Fully functional Linux clipboard manager application  
**Time Estimate:** 40-60 hours of development work

---

## Mission Statement

You are an expert Linux systems engineer tasked with building a lightweight, keyboard-driven screenshot and clipboard history manager for Linux. This tool must be production-ready, well-tested, and follow modern software engineering best practices.

---

## Context Documents

Before starting, carefully read these three documents:
1. **prd.md** - Product requirements, features, user stories
2. **tech_stack.md** - Technology choices, libraries, architecture
3. **system_architecture.md** - System design, component interactions

**Critical:** Understand the entire scope before writing any code.

---

## Technology Stack (REQUIRED)

You MUST use the following stack:

**Primary Language:** Rust 1.60+

**Core Dependencies:**
```toml
[dependencies]
gtk4 = "0.7"                    # UI framework
xcb = "1.2"                     # X11 bindings
x11rb = "0.13"                  # X11 protocol
image = "0.24"                  # Image processing (PNG)
rusqlite = { version = "0.30", features = ["bundled"] }  # SQLite
serde = { version = "1.0", features = ["derive"] }       # Serialization
toml = "0.8"                    # Config file parsing
global-hotkey = "0.5"           # Global keyboard shortcuts
notify-rust = "4.10"            # Desktop notifications
arboard = "3.3"                 # Clipboard management
chrono = "0.4"                  # Timestamps
tokio = { version = "1", features = ["full"] }  # Async runtime
```

**Build System:** Cargo  
**Testing:** `cargo test` + integration tests

---

## Project Structure

Create the following directory structure:

```
clipboard-capture/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   ├── main.rs                 # Daemon entry point
│   ├── config.rs               # Configuration management
│   ├── database.rs             # SQLite operations
│   ├── screenshot.rs           # X11 screen capture
│   ├── clipboard.rs            # Clipboard monitoring
│   ├── hotkeys.rs              # Global hotkey registration
│   ├── notifications.rs        # Desktop notifications
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── overlay.rs          # Screenshot selection overlay
│   │   └── history_dialog.rs  # Clipboard history UI
│   └── models/
│       ├── mod.rs
│       └── history_entry.rs   # Data models
├── tests/
│   ├── integration_tests.rs
│   └── screenshot_tests.rs
├── resources/
│   ├── default_config.toml
│   └── clipboard-capture.desktop
└── docs/
    ├── INSTALL.md
    └── USAGE.md
```

---

## Implementation Phases

### Phase 1: Foundation (Hours 1-10)

**Goal:** Set up project structure, config, database

**Tasks:**

1. **Initialize Rust Project**
   ```bash
   cargo new clipboard-capture --bin
   cd clipboard-capture
   ```

2. **Configure Cargo.toml**
   - Add all dependencies from tech_stack.md
   - Set release profile optimizations
   - Configure metadata (authors, license, description)

3. **Implement Configuration System** (`src/config.rs`)
   ```rust
   #[derive(Debug, Deserialize, Serialize)]
   pub struct Config {
       pub shortcuts: Shortcuts,
       pub capture: CaptureConfig,
       pub history: HistoryConfig,
       pub storage: StorageConfig,
       pub ui: UiConfig,
   }
   
   impl Config {
       pub fn load() -> Result<Self>;
       pub fn load_or_create_default() -> Result<Self>;
       pub fn save(&self) -> Result<()>;
   }
   ```
   - Load from `~/.config/clipboard-capture/config.toml`
   - Create default if missing
   - Validate all fields
   - Handle parse errors gracefully

4. **Implement Database Layer** (`src/database.rs`)
   ```rust
   pub struct Database {
       conn: Connection,
   }
   
   impl Database {
       pub fn new(db_path: &str) -> Result<Self>;
       pub fn init_schema(&self) -> Result<()>;
       pub fn insert_image(&self, png_bytes: Vec<u8>, thumbnail: Vec<u8>) -> Result<i64>;
       pub fn insert_text(&self, text: String) -> Result<i64>;
       pub fn get_recent_entries(&self, limit: usize) -> Result<Vec<HistoryEntry>>;
       pub fn search_text(&self, query: &str) -> Result<Vec<HistoryEntry>>;
       pub fn delete_entry(&self, id: i64) -> Result<()>;
       pub fn cleanup_old_entries(&self, days: i64) -> Result<()>;
   }
   ```
   - Create tables (see tech_stack.md for schema)
   - Add indexes for performance
   - Implement CRUD operations
   - Handle SQLite errors

5. **Define Data Models** (`src/models/history_entry.rs`)
   ```rust
   #[derive(Debug, Clone)]
   pub enum ContentType {
       Image,
       Text,
   }
   
   #[derive(Debug, Clone)]
   pub struct HistoryEntry {
       pub id: i64,
       pub content_type: ContentType,
       pub image_data: Option<Vec<u8>>,
       pub thumbnail: Option<Vec<u8>>,
       pub text_content: Option<String>,
       pub created_at: i64,
       pub file_size: usize,
   }
   ```

**Validation Criteria:**
- [ ] Config file loads and creates default if missing
- [ ] Database creates all tables with correct schema
- [ ] Can insert and retrieve test data
- [ ] All error cases handled with Result<T, Error>

---

### Phase 2: X11 Screenshot Capture (Hours 11-25)

**Goal:** Implement area selection and screen capture

**Tasks:**

1. **X11 Connection Management** (`src/screenshot.rs`)
   ```rust
   pub struct ScreenshotCapture {
       connection: xcb::Connection,
       screen: xcb::Screen,
   }
   
   impl ScreenshotCapture {
       pub fn new() -> Result<Self> {
           // Connect to X11 display
           // Get default screen
       }
       
       pub fn get_screen_geometry(&self) -> (u16, u16) {
           // Return (width, height) of screen
       }
   }
   ```

2. **Transparent Overlay Window** (`src/ui/overlay.rs`)
   ```rust
   pub struct SelectionOverlay {
       window: xcb::Window,
       connection: Arc<xcb::Connection>,
   }
   
   impl SelectionOverlay {
       pub fn new(connection: Arc<xcb::Connection>, screen: &xcb::Screen) -> Result<Self>;
       
       pub async fn show_and_select(&self) -> Result<Rectangle> {
           // 1. Create fullscreen transparent window
           // 2. Dim background (50% black overlay)
           // 3. Capture mouse events
           // 4. Draw selection rectangle on drag
           // 5. Return selected area on mouse release
       }
       
       fn draw_selection_rectangle(&self, x: i16, y: i16, width: u16, height: u16);
       fn handle_mouse_press(&mut self, event: xcb::ButtonPressEvent);
       fn handle_mouse_motion(&mut self, event: xcb::MotionNotifyEvent);
       fn handle_mouse_release(&mut self, event: xcb::ButtonReleaseEvent) -> Rectangle;
   }
   
   #[derive(Debug, Clone)]
   pub struct Rectangle {
       pub x: i16,
       pub y: i16,
       pub width: u16,
       pub height: u16,
   }
   ```
   
   **Implementation Details:**
   - Use `xcb::create_window` with `WINDOW_CLASS_INPUT_OUTPUT`
   - Set `CW_OVERRIDE_REDIRECT` to 1 (bypass window manager)
   - Enable transparency with composite extension
   - Grab mouse pointer during selection
   - Draw rectangle using `xcb::poly_rectangle`
   - Show dimensions label (e.g., "1920 × 1080") at top-left of rectangle

3. **Screen Region Capture**
   ```rust
   impl ScreenshotCapture {
       pub fn capture_region(&self, rect: Rectangle) -> Result<Vec<u8>> {
           // 1. Get image from X11 using xcb::get_image
           // 2. Convert from X11 format to RGBA
           // 3. Return raw pixel data
       }
   }
   ```
   
   **X11 Capture Approach:**
   ```rust
   // Use XGetImage or xcb::shm for better performance
   let cookie = xcb::get_image(
       &self.connection,
       xcb::IMAGE_FORMAT_Z_PIXMAP as u8,
       self.screen.root(),
       rect.x,
       rect.y,
       rect.width,
       rect.height,
       !0, // All planes
   );
   let reply = cookie.get_reply()?;
   let raw_pixels = reply.data();
   
   // Convert BGR/BGRA to RGBA
   let rgba_pixels = self.convert_to_rgba(raw_pixels, rect.width, rect.height);
   ```

4. **PNG Encoding**
   ```rust
   pub fn encode_png(pixels: Vec<u8>, width: u32, height: u32) -> Result<Vec<u8>> {
       use image::{ImageBuffer, RgbaImage};
       
       let img: RgbaImage = ImageBuffer::from_raw(width, height, pixels)
           .ok_or("Invalid image buffer")?;
       
       let mut png_bytes = Vec::new();
       img.write_to(
           &mut std::io::Cursor::new(&mut png_bytes),
           image::ImageFormat::Png
       )?;
       
       Ok(png_bytes)
   }
   ```

5. **Thumbnail Generation**
   ```rust
   pub fn create_thumbnail(png_bytes: &[u8], max_size: u32) -> Result<Vec<u8>> {
       let img = image::load_from_memory(png_bytes)?;
       let thumbnail = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);
       
       let mut thumb_bytes = Vec::new();
       thumbnail.write_to(
           &mut std::io::Cursor::new(&mut thumb_bytes),
           image::ImageFormat::Png
       )?;
       
       Ok(thumb_bytes)
   }
   ```

**Validation Criteria:**
- [ ] Overlay window appears fullscreen
- [ ] Background dims correctly
- [ ] Selection rectangle draws smoothly
- [ ] Dimensions label displays correctly
- [ ] ESC cancels selection
- [ ] Captured image matches selected area
- [ ] PNG encoding produces valid images
- [ ] Multi-monitor support works

---

### Phase 3: Clipboard Management (Hours 26-35)

**Goal:** Implement clipboard copy and monitoring

**Tasks:**

1. **Clipboard Operations** (`src/clipboard.rs`)
   ```rust
   pub struct ClipboardManager {
       clipboard: Clipboard,
       db: Arc<Mutex<Database>>,
       last_text: Arc<Mutex<Option<String>>>,
       last_image_hash: Arc<Mutex<Option<u64>>>,
   }
   
   impl ClipboardManager {
       pub fn new(db: Arc<Mutex<Database>>) -> Result<Self>;
       
       pub fn set_image(&mut self, png_bytes: Vec<u8>) -> Result<()> {
           // 1. Create ImageData from PNG bytes
           // 2. Set clipboard using arboard
           // 3. Generate thumbnail
           // 4. Store in database
           // 5. Send notification
       }
       
       pub fn set_text(&mut self, text: String) -> Result<()>;
       
       pub fn start_monitoring(&self) {
           // Background thread that polls clipboard every 500ms
           // Detects changes and stores new content
       }
   }
   ```

2. **Clipboard Monitoring Thread**
   ```rust
   pub fn monitor_clipboard(clipboard_mgr: Arc<Mutex<ClipboardManager>>) {
       tokio::spawn(async move {
           loop {
               tokio::time::sleep(Duration::from_millis(500)).await;
               
               let mut mgr = clipboard_mgr.lock().await;
               
               // Try to get text
               if let Ok(text) = mgr.clipboard.get_text() {
                   let last = mgr.last_text.lock().await;
                   if Some(&text) != last.as_ref() {
                       mgr.handle_new_text(text).await;
                   }
               }
               
               // Try to get image
               if let Ok(img_data) = mgr.clipboard.get_image() {
                   let hash = calculate_hash(&img_data.bytes);
                   let last_hash = mgr.last_image_hash.lock().await;
                   if Some(hash) != *last_hash {
                       mgr.handle_new_image(img_data).await;
                   }
               }
           }
       });
   }
   ```

3. **Deduplication**
   ```rust
   fn calculate_hash(data: &[u8]) -> u64 {
       use std::collections::hash_map::DefaultHasher;
       use std::hash::{Hash, Hasher};
       
       let mut hasher = DefaultHasher::new();
       data.hash(&mut hasher);
       hasher.finish()
   }
   ```

**Validation Criteria:**
- [ ] Screenshot copies to clipboard successfully
- [ ] Can paste in Firefox, GIMP, LibreOffice
- [ ] Clipboard monitoring detects text copies
- [ ] Duplicate entries are not stored
- [ ] Database entries have correct timestamps
- [ ] Thumbnails generate correctly

---

### Phase 4: Global Hotkeys (Hours 36-40)

**Goal:** Register and handle keyboard shortcuts

**Tasks:**

1. **Hotkey Manager** (`src/hotkeys.rs`)
   ```rust
   use global_hotkey::{GlobalHotKeyManager, HotKey, HotKeyModifiers, HotKeyCode};
   
   pub struct HotkeyManager {
       manager: GlobalHotKeyManager,
       screenshot_hotkey: HotKey,
       history_hotkey: HotKey,
   }
   
   impl HotkeyManager {
       pub fn new(config: &Config) -> Result<Self> {
           let manager = GlobalHotKeyManager::new()?;
           
           // Parse from config: "Ctrl+Super+S"
           let screenshot_hotkey = Self::parse_hotkey(&config.shortcuts.screenshot)?;
           let history_hotkey = Self::parse_hotkey(&config.shortcuts.history)?;
           
           manager.register(screenshot_hotkey)?;
           manager.register(history_hotkey)?;
           
           Ok(Self { manager, screenshot_hotkey, history_hotkey })
       }
       
       fn parse_hotkey(config_str: &str) -> Result<HotKey> {
           // Parse "Ctrl+Super+S" into HotKey
           // Handle: Ctrl, Super, Shift, Alt
           // Map key codes: A-Z, 0-9, F1-F12, etc.
       }
       
       pub fn handle_events<F>(&self, screenshot_cb: F, history_cb: F)
       where
           F: Fn() + Send + 'static,
       {
           let receiver = self.manager.receiver();
           tokio::spawn(async move {
               while let Ok(event) = receiver.recv() {
                   if event.id == self.screenshot_hotkey.id {
                       screenshot_cb();
                   } else if event.id == self.history_hotkey.id {
                       history_cb();
                   }
               }
           });
       }
   }
   ```

**Validation Criteria:**
- [ ] Ctrl+Win+S triggers screenshot overlay
- [ ] Win+H opens history dialog
- [ ] Hotkeys work from any application
- [ ] Config changes update hotkeys on reload

---

### Phase 5: History Dialog UI (Hours 41-50)

**Goal:** Build GTK4 history browser

**Tasks:**

1. **GTK Application Setup** (`src/main.rs`)
   ```rust
   use gtk4::prelude::*;
   use gtk4::{Application, ApplicationWindow};
   
   fn main() {
       let app = Application::builder()
           .application_id("com.clipboard-capture.app")
           .build();
       
       app.connect_activate(|app| {
           // Initialize daemon components
           // Don't create windows yet
       });
       
       app.run();
   }
   ```

2. **History Dialog** (`src/ui/history_dialog.rs`)
   ```rust
   pub struct HistoryDialog {
       window: ApplicationWindow,
       search_entry: gtk4::Entry,
       scrolled_window: gtk4::ScrolledWindow,
       flow_box: gtk4::FlowBox,
       db: Arc<Mutex<Database>>,
   }
   
   impl HistoryDialog {
       pub fn new(app: &Application, db: Arc<Mutex<Database>>) -> Self {
           let window = ApplicationWindow::builder()
               .application(app)
               .title("Clipboard History")
               .default_width(600)
               .default_height(400)
               .build();
           
           let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
           vbox.set_margin_top(12);
           vbox.set_margin_bottom(12);
           vbox.set_margin_start(12);
           vbox.set_margin_end(12);
           
           // Search entry
           let search_entry = gtk4::Entry::builder()
               .placeholder_text("Search clipboard...")
               .build();
           
           // Scrolled window
           let scrolled_window = gtk4::ScrolledWindow::builder()
               .hscrollbar_policy(gtk4::PolicyType::Never)
               .vexpand(true)
               .build();
           
           // FlowBox for grid layout
           let flow_box = gtk4::FlowBox::new();
           flow_box.set_max_children_per_line(4);
           flow_box.set_selection_mode(gtk4::SelectionMode::Single);
           
           scrolled_window.set_child(Some(&flow_box));
           
           vbox.append(&search_entry);
           vbox.append(&scrolled_window);
           
           window.set_child(Some(&vbox));
           
           Self { window, search_entry, scrolled_window, flow_box, db }
       }
       
       pub fn show(&self) {
           self.load_history();
           self.window.present();
       }
       
       fn load_history(&self) {
           let db = self.db.lock().unwrap();
           let entries = db.get_recent_entries(50).unwrap();
           
           for entry in entries {
               let widget = self.create_entry_widget(entry);
               self.flow_box.append(&widget);
           }
       }
       
       fn create_entry_widget(&self, entry: HistoryEntry) -> gtk4::Widget {
           match entry.content_type {
               ContentType::Image => self.create_image_widget(entry),
               ContentType::Text => self.create_text_widget(entry),
           }
       }
       
       fn create_image_widget(&self, entry: HistoryEntry) -> gtk4::Widget {
           let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
           
           // Thumbnail
           if let Some(thumb_bytes) = entry.thumbnail {
               let pixbuf = gdk_pixbuf::Pixbuf::from_read(
                   std::io::Cursor::new(thumb_bytes)
               ).ok();
               
               if let Some(pixbuf) = pixbuf {
                   let image = gtk4::Image::from_pixbuf(Some(&pixbuf));
                   vbox.append(&image);
               }
           }
           
           // Timestamp
           let time_label = gtk4::Label::new(Some(&format_timestamp(entry.created_at)));
           time_label.add_css_class("dim-label");
           vbox.append(&time_label);
           
           // Click handler
           let entry_clone = entry.clone();
           let gesture = gtk4::GestureClick::new();
           gesture.connect_released(move |_, _, _, _| {
               self.copy_entry_to_clipboard(entry_clone.clone());
           });
           vbox.add_controller(gesture);
           
           vbox.upcast()
       }
       
       fn create_text_widget(&self, entry: HistoryEntry) -> gtk4::Widget {
           let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
           
           // Text preview (first 100 chars)
           if let Some(text) = entry.text_content {
               let preview = if text.len() > 100 {
                   format!("{}...", &text[..100])
               } else {
                   text.clone()
               };
               
               let label = gtk4::Label::new(Some(&preview));
               label.set_wrap(true);
               label.set_xalign(0.0);
               vbox.append(&label);
           }
           
           // Similar timestamp and click handler
           
           vbox.upcast()
       }
       
       fn copy_entry_to_clipboard(&self, entry: HistoryEntry) {
           let mut clipboard = arboard::Clipboard::new().unwrap();
           
           match entry.content_type {
               ContentType::Image => {
                   if let Some(img_bytes) = entry.image_data {
                       // Decode PNG and set to clipboard
                       let img = image::load_from_memory(&img_bytes).unwrap();
                       let rgba = img.to_rgba8();
                       let img_data = arboard::ImageData {
                           width: rgba.width() as usize,
                           height: rgba.height() as usize,
                           bytes: rgba.into_raw().into(),
                       };
                       clipboard.set_image(img_data).unwrap();
                   }
               }
               ContentType::Text => {
                   if let Some(text) = entry.text_content {
                       clipboard.set_text(text).unwrap();
                   }
               }
           }
           
           // Close dialog
           self.window.close();
           
           // Notification
           Notification::new()
               .summary("Copied to Clipboard")
               .body("Item restored from history")
               .show().ok();
       }
   }
   ```

3. **Search Implementation**
   ```rust
   impl HistoryDialog {
       fn setup_search(&self) {
           let flow_box = self.flow_box.clone();
           let db = self.db.clone();
           
           self.search_entry.connect_changed(move |entry| {
               let query = entry.text().to_string();
               
               // Clear current entries
               while let Some(child) = flow_box.first_child() {
                   flow_box.remove(&child);
               }
               
               // Search database
               let db = db.lock().unwrap();
               let results = if query.is_empty() {
                   db.get_recent_entries(50).unwrap()
               } else {
                   db.search_text(&query).unwrap()
               };
               
               // Populate with results
               for entry in results {
                   let widget = self.create_entry_widget(entry);
                   flow_box.append(&widget);
               }
           });
       }
   }
   ```

**Validation Criteria:**
- [ ] Dialog opens on Win+H
- [ ] Shows image thumbnails correctly
- [ ] Shows text previews correctly
- [ ] Search filters results in real-time
- [ ] Clicking item copies to clipboard
- [ ] Dialog closes after selection
- [ ] Can paste copied item

---

### Phase 6: Main Daemon Loop (Hours 51-55)

**Goal:** Integrate all components

**Tasks:**

1. **Main Entry Point** (`src/main.rs`)
   ```rust
   use tokio::sync::Mutex;
   use std::sync::Arc;
   
   #[tokio::main]
   async fn main() -> Result<()> {
       // Load config
       let config = Config::load_or_create_default()?;
       
       // Initialize database
       let db_path = expand_path(&config.storage.database_path);
       let db = Arc::new(Mutex::new(Database::new(&db_path)?));
       db.lock().await.init_schema()?;
       
       // Initialize GTK
       let app = gtk4::Application::builder()
           .application_id("com.clipboard-capture.daemon")
           .build();
       
       // Initialize clipboard manager
       let clipboard_mgr = Arc::new(Mutex::new(ClipboardManager::new(db.clone())?));
       
       // Start clipboard monitoring
       let clipboard_clone = clipboard_mgr.clone();
       tokio::spawn(async move {
           clipboard_clone.lock().await.start_monitoring();
       });
       
       // Initialize screenshot capture
       let screenshot = Arc::new(Mutex::new(ScreenshotCapture::new()?));
       
       // Initialize hotkey manager
       let hotkey_mgr = HotkeyManager::new(&config)?;
       
       // Setup hotkey callbacks
       let screenshot_clone = screenshot.clone();
       let clipboard_clone = clipboard_mgr.clone();
       hotkey_mgr.handle_screenshot(move || {
           tokio::spawn(async move {
               handle_screenshot_hotkey(screenshot_clone.clone(), clipboard_clone.clone()).await;
           });
       });
       
       let db_clone = db.clone();
       let app_clone = app.clone();
       hotkey_mgr.handle_history(move || {
           let dialog = HistoryDialog::new(&app_clone, db_clone.clone());
           dialog.show();
       });
       
       // Run GTK app
       app.run();
       
       Ok(())
   }
   
   async fn handle_screenshot_hotkey(
       screenshot: Arc<Mutex<ScreenshotCapture>>,
       clipboard: Arc<Mutex<ClipboardManager>>,
   ) {
       // 1. Show overlay and get selection
       let selection = {
           let sc = screenshot.lock().await;
           let overlay = SelectionOverlay::new(sc.connection.clone(), &sc.screen).unwrap();
           overlay.show_and_select().await
       };
       
       if let Ok(rect) = selection {
           // 2. Capture screen region
           let sc = screenshot.lock().await;
           let pixels = sc.capture_region(rect).await.unwrap();
           
           // 3. Encode to PNG
           let png_bytes = encode_png(pixels, rect.width as u32, rect.height as u32).unwrap();
           
           // 4. Copy to clipboard
           let mut cb = clipboard.lock().await;
           cb.set_image(png_bytes).await.unwrap();
       }
   }
   ```

**Validation Criteria:**
- [ ] Daemon starts without errors
- [ ] All hotkeys work
- [ ] Clipboard monitoring runs
- [ ] Screenshots save to database
- [ ] Text copies save to database
- [ ] No memory leaks over 24 hours

---

### Phase 7: Notifications & Polish (Hours 56-60)

**Goal:** Add user feedback and error handling

**Tasks:**

1. **Notification Manager** (`src/notifications.rs`)
   ```rust
   use notify_rust::Notification;
   
   pub fn notify_screenshot_success() {
       Notification::new()
           .summary("Screenshot Captured")
           .body("Image copied to clipboard")
           .icon("camera-photo")
           .timeout(2000)
           .show()
           .ok();
   }
   
   pub fn notify_screenshot_error(error: &str) {
       Notification::new()
           .summary("Screenshot Failed")
           .body(error)
           .icon("dialog-error")
           .urgency(notify_rust::Urgency::Critical)
           .show()
           .ok();
   }
   
   pub fn notify_clipboard_restored() {
       Notification::new()
           .summary("Clipboard Restored")
           .body("Item copied from history")
           .icon("edit-paste")
           .timeout(2000)
           .show()
           .ok();
   }
   ```

2. **Error Handling Standards**
   - Use `anyhow::Result` for application errors
   - Use `thiserror` for custom error types
   - Log errors with `tracing` or `log` crate
   - Never panic in production code
   - Always show user-friendly error messages

3. **Logging Setup**
   ```rust
   use tracing::{info, error, debug};
   
   fn init_logging() {
       tracing_subscriber::fmt()
           .with_max_level(tracing::Level::INFO)
           .with_target(false)
           .init();
   }
   ```

**Validation Criteria:**
- [ ] All operations show notifications
- [ ] Errors display helpful messages
- [ ] Logs written to appropriate location
- [ ] No crashes on invalid input

---

## Testing Requirements

### Unit Tests

Write tests for:
- Config parsing (valid and invalid)
- Database operations (CRUD)
- PNG encoding/decoding
- Hotkey parsing
- Rectangle calculations

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default_creation() {
        let config = Config::default();
        assert_eq!(config.shortcuts.screenshot, "Ctrl+Super+S");
    }
    
    #[test]
    fn test_database_insert_text() {
        let db = Database::new(":memory:").unwrap();
        db.init_schema().unwrap();
        
        let id = db.insert_text("test text".to_string()).unwrap();
        assert!(id > 0);
        
        let entries = db.get_recent_entries(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text_content, Some("test text".to_string()));
    }
}
```

### Integration Tests

Test full workflows:
- Screenshot → clipboard → database
- Text copy → history → restore
- Search functionality
- Config reload

### Manual Testing Checklist

Create a file `TESTING.md` with:
- [ ] Single monitor screenshot
- [ ] Multi-monitor screenshot
- [ ] HiDPI/fractional scaling
- [ ] Clipboard paste in 10+ apps (Firefox, GIMP, LibreOffice, VSCode, etc.)
- [ ] History search with various queries
- [ ] Config file modification and reload
- [ ] Daemon restart (history persists)
- [ ] System reboot (history persists)
- [ ] 24-hour stress test (no leaks)

---

## Documentation Requirements

### README.md

Create a comprehensive README with:
- Project description
- Features list
- Screenshots/GIFs
- Installation instructions
- Usage guide
- Configuration options
- Building from source
- Contributing guidelines
- License

### INSTALL.md

Platform-specific install instructions for:
- Ubuntu/Debian
- Fedora/RHEL
- Arch Linux
- Generic Linux (AppImage)

### USAGE.md

User guide covering:
- First-time setup
- Keyboard shortcuts
- Taking screenshots
- Accessing history
- Searching clipboard
- Configuration options
- Troubleshooting

---

## Quality Standards

Your implementation must meet these standards:

### Code Quality
- [ ] All code properly formatted (`cargo fmt`)
- [ ] No compiler warnings (`cargo clippy`)
- [ ] No unsafe code (unless absolutely necessary and documented)
- [ ] Comprehensive error handling (no unwrap() in production paths)
- [ ] Meaningful variable and function names
- [ ] Inline comments for complex logic
- [ ] Module-level documentation

### Performance
- [ ] Startup time < 2 seconds
- [ ] Screenshot latency < 500ms
- [ ] History dialog opens < 200ms
- [ ] Idle memory < 50MB
- [ ] No memory leaks (test with valgrind)
- [ ] Database queries < 50ms

### Reliability
- [ ] Handles out-of-memory gracefully
- [ ] Recovers from X11 connection errors
- [ ] Handles missing config files
- [ ] Database corruption recovery
- [ ] No data loss on crashes

### Security
- [ ] No buffer overflows
- [ ] No SQL injection (use prepared statements)
- [ ] Proper file permissions on config/database
- [ ] No unnecessary privileges required
- [ ] Input validation on all user data

---

## Build & Distribution

### Release Build

```bash
cargo build --release
strip target/release/clipboard-capture-daemon
```

**Target binary size:** < 20MB

### Package Formats

Create these packages:

1. **AppImage** (universal Linux)
2. **.deb** (Ubuntu/Debian)
3. **.rpm** (Fedora/RHEL)
4. **AUR package** (Arch Linux)

### Desktop Integration

Create `clipboard-capture.desktop`:
```desktop
[Desktop Entry]
Type=Application
Name=Clipboard Capture
Comment=Screenshot and clipboard history manager
Exec=/usr/bin/clipboard-capture-daemon
Icon=clipboard-capture
Categories=Utility;
StartupNotify=false
Terminal=false
```

### Systemd Service

Create `clipboard-capture.service`:
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

## Common Pitfalls to Avoid

1. **Clipboard Ownership Loss**
   - Solution: Keep process alive as clipboard owner
   - Use Arboard's built-in ownership management

2. **X11 Connection Errors**
   - Always check connection status
   - Handle disconnections gracefully
   - Test with display managers (GDM, SDDM, LightDM)

3. **Memory Leaks**
   - Use Arc/Mutex correctly
   - Drop unused resources
   - Test with long-running daemon (24+ hours)

4. **GTK Threading Issues**
   - All GTK operations must be on main thread
   - Use `glib::idle_add` for cross-thread UI updates

5. **Database Locking**
   - Use connection pools for multi-threaded access
   - Handle SQLITE_BUSY errors
   - Set appropriate timeouts

6. **HiDPI Issues**
   - Query scale factor from X11
   - Multiply coordinates appropriately
   - Test on 1x, 1.5x, 2x scaling

---

## Success Criteria

Your implementation is complete when:

- [ ] All Phase 1-7 tasks completed
- [ ] All validation criteria met
- [ ] 100% of P0 requirements implemented
- [ ] 80%+ code coverage in tests
- [ ] All manual testing checklist items passed
- [ ] Documentation complete (README, INSTALL, USAGE)
- [ ] Binary builds successfully
- [ ] Works on Ubuntu 22.04, Fedora 39, Arch Linux
- [ ] No critical or high-severity bugs
- [ ] Passes 24-hour stress test
- [ ] Code review by senior developer (or AI review)

---

## Final Deliverables

Submit:

1. **Source Code** (GitHub repository)
   - All Rust source files
   - Cargo.toml with dependencies
   - Tests (unit + integration)
   - .gitignore

2. **Documentation**
   - README.md
   - INSTALL.md
   - USAGE.md
   - TESTING.md
   - API documentation (cargo doc)

3. **Binaries**
   - Linux x86_64 release build
   - AppImage (universal)
   - .deb package
   - .rpm package

4. **Configuration**
   - Default config.toml
   - Desktop file
   - Systemd service

5. **Release Notes**
   - Changelog
   - Known issues
   - Future roadmap

---

## Post-Development Tasks

After core implementation:

1. **Performance Profiling**
   ```bash
   cargo flamegraph --bin clipboard-capture-daemon
   valgrind --tool=massif target/release/clipboard-capture-daemon
   ```

2. **Security Audit**
   ```bash
   cargo audit
   cargo clippy -- -D warnings
   ```

3. **User Testing**
   - Recruit 5-10 beta testers
   - Collect feedback
   - Fix critical issues

4. **GitHub Setup**
   - Create issues for known bugs
   - Add contribution guidelines
   - Set up CI/CD (GitHub Actions)

5. **Release v1.0.0**
   - Tag release
   - Publish binaries
   - Announce on Reddit/HN

---

## Resources

**Reference Implementations:**
- CopyQ (Qt clipboard manager)
- Flameshot (screenshot tool)
- Greenclip (Rofi clipboard manager)

**Documentation:**
- X11 Protocol: https://www.x.org/releases/current/doc/
- XCB Documentation: https://xcb.freedesktop.org/
- GTK 4 Tutorial: https://gtk-rs.org/gtk4-rs/stable/latest/book/
- Rust Book: https://doc.rust-lang.org/book/

**Tools:**
- xev (monitor X11 events)
- xprop (inspect window properties)
- xdpyinfo (display information)

---

## Questions for AI Agent

Before starting, please confirm:

1. Do you understand the full scope of the project?
2. Are you familiar with all the technologies in the stack?
3. Do you need clarification on any requirements?
4. Are there any technical constraints I should know about?
5. What's your estimated timeline for completion?

---

**Good luck! Build something amazing.**
