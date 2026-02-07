# Task List - Clipboard Capture Project
## Complete Implementation Checklist

**Project:** Area Screenshot & Clipboard Manager for Linux  
**Language:** Rust  
**Estimated Time:** 60 hours  
**Last Updated:** February 7, 2026

---

## Quick Reference

- **Total Tasks:** 87
- **P0 (Critical):** 45 tasks
- **P1 (High):** 28 tasks
- **P2 (Medium):** 14 tasks

---

## Phase 1: Foundation (Hours 1-10)

### 1.1 Project Setup
- [ ] **P0** Initialize Rust project with cargo
- [ ] **P0** Configure Cargo.toml with all dependencies
- [ ] **P0** Set up project directory structure
- [ ] **P0** Create .gitignore file
- [ ] **P1** Set up Rust release profile optimizations
- [ ] **P2** Create LICENSE file
- [ ] **P2** Create initial README.md

### 1.2 Configuration System (`src/config.rs`)
- [ ] **P0** Define Config struct with all fields
- [ ] **P0** Define nested config structs (Shortcuts, CaptureConfig, etc.)
- [ ] **P0** Implement Config::load() from TOML file
- [ ] **P0** Implement Config::load_or_create_default()
- [ ] **P0** Implement Config::save()
- [ ] **P0** Create default config values
- [ ] **P0** Implement config file path expansion (~/.config)
- [ ] **P1** Add config validation logic
- [ ] **P1** Handle TOML parse errors gracefully
- [ ] **P1** Add unit tests for config parsing
- [ ] **P2** Create example config.toml file

### 1.3 Database Layer (`src/database.rs`)
- [ ] **P0** Define Database struct with SQLite connection
- [ ] **P0** Implement Database::new() constructor
- [ ] **P0** Implement init_schema() - create tables
- [ ] **P0** Create clipboard_history table with correct schema
- [ ] **P0** Add indexes (created_at, content_type, text_search)
- [ ] **P0** Implement insert_image() function
- [ ] **P0** Implement insert_text() function
- [ ] **P0** Implement get_recent_entries() function
- [ ] **P0** Implement search_text() function
- [ ] **P1** Implement delete_entry() function
- [ ] **P1** Implement cleanup_old_entries() function
- [ ] **P1** Implement vacuum_database() function
- [ ] **P1** Add CHECK constraints for data integrity
- [ ] **P1** Handle SQLite error cases (BUSY, LOCKED)
- [ ] **P1** Add connection timeout configuration
- [ ] **P1** Write unit tests for all CRUD operations
- [ ] **P2** Implement database migration system (future)

### 1.4 Data Models (`src/models/`)
- [ ] **P0** Define ContentType enum (Image, Text)
- [ ] **P0** Define HistoryEntry struct
- [ ] **P0** Define Rectangle struct
- [ ] **P1** Define Monitor struct
- [ ] **P1** Define custom error types with thiserror
- [ ] **P1** Implement Display for error types
- [ ] **P1** Add From implementations for error conversions

### 1.5 Phase 1 Validation
- [ ] **P0** Config loads from file successfully
- [ ] **P0** Config creates default if missing
- [ ] **P0** Database creates all tables
- [ ] **P0** Can insert and retrieve test data
- [ ] **P1** All error cases return Result types
- [ ] **P1** Unit tests pass for config and database

---

## Phase 2: X11 Screenshot Capture (Hours 11-25)

### 2.1 X11 Connection (`src/screenshot.rs`)
- [ ] **P0** Define ScreenshotCapture struct
- [ ] **P0** Implement connection to X11 display
- [ ] **P0** Get default screen information
- [ ] **P0** Implement get_screen_geometry()
- [ ] **P1** Query RandR for multi-monitor layout
- [ ] **P1** Create Monitor list from RandR
- [ ] **P1** Handle HiDPI scale factors
- [ ] **P1** Handle X11 connection errors gracefully
- [ ] **P2** Support DISPLAY environment variable

### 2.2 Transparent Overlay Window (`src/ui/overlay.rs`)
- [ ] **P0** Define SelectionOverlay struct
- [ ] **P0** Create fullscreen transparent window with XCB
- [ ] **P0** Set window attributes (override_redirect, event_mask)
- [ ] **P0** Enable transparency with composite extension
- [ ] **P0** Map window to display
- [ ] **P0** Grab mouse pointer during selection
- [ ] **P0** Handle mouse button press event
- [ ] **P0** Handle mouse motion event
- [ ] **P0** Handle mouse button release event
- [ ] **P0** Handle ESC key to cancel
- [ ] **P1** Dim background (50% black overlay)
- [ ] **P1** Draw selection rectangle with blue border
- [ ] **P1** Add semi-transparent fill to rectangle
- [ ] **P1** Display dimensions label (e.g., "1920 × 1080")
- [ ] **P1** Update rectangle in real-time during drag
- [ ] **P1** Change cursor to crosshair
- [ ] **P1** Handle click without drag (cancel)
- [ ] **P1** Destroy overlay window after selection
- [ ] **P2** Add visual feedback animations

### 2.3 Screen Capture
- [ ] **P0** Implement capture_region() function
- [ ] **P0** Use xcb::get_image for screen capture
- [ ] **P0** Handle raw pixel data from X11
- [ ] **P0** Convert BGR/BGRA to RGBA format
- [ ] **P0** Support multi-monitor captures
- [ ] **P1** Use XShm for better performance (if available)
- [ ] **P1** Handle capture errors (invalid region, etc.)
- [ ] **P2** Optimize memory allocation for large captures

### 2.4 Image Processing
- [ ] **P0** Implement encode_png() function
- [ ] **P0** Use image crate for PNG encoding
- [ ] **P0** Create ImageBuffer from RGBA pixels
- [ ] **P0** Write PNG to memory buffer
- [ ] **P0** Implement create_thumbnail() function
- [ ] **P0** Resize image to max 150x150 pixels
- [ ] **P0** Use Lanczos3 filter for quality
- [ ] **P1** Handle image encoding errors
- [ ] **P1** Optimize PNG compression level
- [ ] **P2** Add JPEG support (configurable)

### 2.5 Phase 2 Validation
- [ ] **P0** Overlay window appears fullscreen
- [ ] **P0** Background dims correctly
- [ ] **P0** Selection rectangle draws smoothly
- [ ] **P0** Dimensions label displays correctly
- [ ] **P0** ESC cancels selection
- [ ] **P0** Captured image matches selected area
- [ ] **P0** PNG encoding produces valid images
- [ ] **P1** Multi-monitor support works
- [ ] **P1** HiDPI scaling handled correctly
- [ ] **P1** Thumbnails generate correctly
- [ ] **P2** Performance: capture completes in <500ms

---

## Phase 3: Clipboard Management (Hours 26-35)

### 3.1 Clipboard Operations (`src/clipboard.rs`)
- [ ] **P0** Define ClipboardManager struct
- [ ] **P0** Initialize Arboard clipboard instance
- [ ] **P0** Implement set_image() function
- [ ] **P0** Convert PNG bytes to ImageData format
- [ ] **P0** Set image to clipboard via Arboard
- [ ] **P0** Implement set_text() function
- [ ] **P0** Implement get_text() function
- [ ] **P0** Implement get_image() function
- [ ] **P1** Handle clipboard ownership
- [ ] **P1** Handle clipboard errors gracefully
- [ ] **P2** Support multiple MIME types

### 3.2 Clipboard Monitoring
- [ ] **P0** Implement clipboard monitoring loop
- [ ] **P0** Poll clipboard every 500ms
- [ ] **P0** Detect text clipboard changes
- [ ] **P0** Detect image clipboard changes
- [ ] **P0** Store new content in database
- [ ] **P0** Run monitoring in background thread
- [ ] **P1** Implement deduplication logic
- [ ] **P1** Calculate content hash (SHA256)
- [ ] **P1** Compare with previous hash
- [ ] **P1** Skip duplicate consecutive entries
- [ ] **P1** Handle monitoring thread errors
- [ ] **P2** Make polling interval configurable

### 3.3 Database Integration
- [ ] **P0** Call database.insert_image() after capture
- [ ] **P0** Call database.insert_text() on text copy
- [ ] **P0** Store timestamp with each entry
- [ ] **P0** Store file size with each entry
- [ ] **P1** Generate and store thumbnail
- [ ] **P1** Handle database insert errors
- [ ] **P2** Implement batch inserts for performance

### 3.4 Phase 3 Validation
- [ ] **P0** Screenshot copies to clipboard successfully
- [ ] **P0** Can paste in Firefox
- [ ] **P0** Can paste in GIMP
- [ ] **P0** Can paste in LibreOffice
- [ ] **P0** Clipboard monitoring detects text copies
- [ ] **P0** Duplicate entries are not stored
- [ ] **P0** Database entries have correct timestamps
- [ ] **P1** Can paste in 10+ different applications
- [ ] **P1** Thumbnails generate correctly
- [ ] **P2** Monitoring has <5% CPU usage

---

## Phase 4: Global Hotkeys (Hours 36-40)

### 4.1 Hotkey Manager (`src/hotkeys.rs`)
- [ ] **P0** Define HotkeyManager struct
- [ ] **P0** Initialize GlobalHotKeyManager from crate
- [ ] **P0** Implement parse_hotkey() function
- [ ] **P0** Parse "Ctrl+Super+S" format
- [ ] **P0** Map modifiers (Ctrl, Alt, Shift, Super)
- [ ] **P0** Map key codes (A-Z, 0-9, F1-F12)
- [ ] **P0** Register screenshot hotkey
- [ ] **P0** Register history hotkey
- [ ] **P0** Implement event receiver loop
- [ ] **P0** Dispatch events to callbacks
- [ ] **P1** Handle hotkey registration errors
- [ ] **P1** Support hotkey unregistration
- [ ] **P1** Support hotkey re-registration (config reload)
- [ ] **P1** Add unit tests for hotkey parsing
- [ ] **P2** Support additional modifiers

### 4.2 Event Dispatching
- [ ] **P0** Create callback for screenshot hotkey
- [ ] **P0** Create callback for history hotkey
- [ ] **P0** Spawn async task for screenshot
- [ ] **P0** Open history dialog on hotkey
- [ ] **P1** Handle callback errors
- [ ] **P1** Prevent concurrent screenshot captures
- [ ] **P2** Add logging for hotkey events

### 4.3 Phase 4 Validation
- [ ] **P0** Ctrl+Win+S triggers screenshot overlay
- [ ] **P0** Win+H opens history dialog
- [ ] **P0** Hotkeys work from any application
- [ ] **P1** Config changes update hotkeys
- [ ] **P1** No hotkey conflicts with system
- [ ] **P2** Response time <300ms

---

## Phase 5: History Dialog UI (Hours 41-50)

### 5.1 GTK Application Setup (`src/main.rs`)
- [ ] **P0** Initialize GTK4 Application
- [ ] **P0** Set application ID
- [ ] **P0** Connect activate signal
- [ ] **P0** Run GTK event loop
- [ ] **P1** Handle GTK initialization errors
- [ ] **P2** Add application menu (future)

### 5.2 History Dialog Window (`src/ui/history_dialog.rs`)
- [ ] **P0** Define HistoryDialog struct
- [ ] **P0** Create ApplicationWindow
- [ ] **P0** Set window title ("Clipboard History")
- [ ] **P0** Set default window size (600×400)
- [ ] **P0** Create vertical Box layout
- [ ] **P0** Add margins to layout
- [ ] **P0** Create search Entry widget
- [ ] **P0** Set search placeholder text
- [ ] **P0** Create ScrolledWindow
- [ ] **P0** Create FlowBox for grid layout
- [ ] **P0** Set FlowBox max columns (4)
- [ ] **P0** Set FlowBox selection mode
- [ ] **P0** Add FlowBox to ScrolledWindow
- [ ] **P0** Add widgets to vertical Box
- [ ] **P0** Set Box as window child
- [ ] **P1** Add status bar with item count
- [ ] **P1** Make window resizable
- [ ] **P1** Remember window size/position
- [ ] **P2** Add window icon

### 5.3 Entry Widgets
- [ ] **P0** Implement create_entry_widget() function
- [ ] **P0** Implement create_image_widget() function
- [ ] **P0** Load thumbnail from database
- [ ] **P0** Create GdkPixbuf from PNG bytes
- [ ] **P0** Create Image widget from Pixbuf
- [ ] **P0** Add timestamp Label
- [ ] **P0** Implement create_text_widget() function
- [ ] **P0** Create Label with text preview
- [ ] **P0** Truncate text to 100 characters
- [ ] **P0** Set label wrapping and alignment
- [ ] **P1** Format timestamps (e.g., "2m ago")
- [ ] **P1** Add CSS styling to widgets
- [ ] **P1** Show ellipsis for long text
- [ ] **P2** Add hover effects

### 5.4 History Loading
- [ ] **P0** Implement load_history() function
- [ ] **P0** Query database for recent 50 entries
- [ ] **P0** Iterate over entries
- [ ] **P0** Create widget for each entry
- [ ] **P0** Append widgets to FlowBox
- [ ] **P1** Handle database query errors
- [ ] **P1** Show loading indicator
- [ ] **P1** Implement lazy loading (scroll pagination)
- [ ] **P2** Cache widgets for performance

### 5.5 Search Functionality
- [ ] **P0** Implement setup_search() function
- [ ] **P0** Connect to Entry "changed" signal
- [ ] **P0** Get search query text
- [ ] **P0** Clear FlowBox when searching
- [ ] **P0** Query database with search term
- [ ] **P0** Populate FlowBox with results
- [ ] **P1** Implement real-time filtering
- [ ] **P1** Highlight search matches
- [ ] **P1** Handle empty search results
- [ ] **P2** Add search history suggestions

### 5.6 Entry Selection & Clipboard Restore
- [ ] **P0** Implement copy_entry_to_clipboard() function
- [ ] **P0** Get full entry data from database
- [ ] **P0** Check entry content type
- [ ] **P0** For images: decode PNG to ImageData
- [ ] **P0** For images: set to clipboard
- [ ] **P0** For text: set text to clipboard
- [ ] **P0** Close dialog after selection
- [ ] **P0** Show "Copied to clipboard" notification
- [ ] **P1** Add click event handler to widgets
- [ ] **P1** Add keyboard navigation (arrow keys)
- [ ] **P1** Support Enter key to select
- [ ] **P1** Handle clipboard set errors
- [ ] **P2** Add context menu (delete, etc.)

### 5.7 Phase 5 Validation
- [ ] **P0** Dialog opens on Win+H
- [ ] **P0** Shows image thumbnails correctly
- [ ] **P0** Shows text previews correctly
- [ ] **P0** Search filters results in real-time
- [ ] **P0** Clicking item copies to clipboard
- [ ] **P0** Dialog closes after selection
- [ ] **P0** Can paste copied item
- [ ] **P1** Dialog opens in <200ms
- [ ] **P1** Thumbnails load quickly
- [ ] **P1** Search has no lag
- [ ] **P2** UI is visually polished

---

## Phase 6: Main Daemon Loop (Hours 51-55)

### 6.1 Daemon Integration (`src/main.rs`)
- [ ] **P0** Add tokio async runtime
- [ ] **P0** Load configuration
- [ ] **P0** Initialize database connection
- [ ] **P0** Call database.init_schema()
- [ ] **P0** Initialize GTK application
- [ ] **P0** Create ClipboardManager instance
- [ ] **P0** Create ScreenshotCapture instance
- [ ] **P0** Create HotkeyManager instance
- [ ] **P0** Wrap shared state in Arc<Mutex>
- [ ] **P0** Start clipboard monitoring thread
- [ ] **P0** Register hotkey callbacks
- [ ] **P0** Run GTK application event loop
- [ ] **P1** Handle initialization errors
- [ ] **P1** Implement graceful shutdown
- [ ] **P1** Clean up resources on exit
- [ ] **P2** Add daemon status command

### 6.2 Screenshot Hotkey Handler
- [ ] **P0** Implement handle_screenshot_hotkey() function
- [ ] **P0** Create SelectionOverlay instance
- [ ] **P0** Call overlay.show_and_select()
- [ ] **P0** Capture screen region
- [ ] **P0** Encode pixels to PNG
- [ ] **P0** Copy to clipboard
- [ ] **P0** Store in database
- [ ] **P0** Show success notification
- [ ] **P1** Handle user cancellation
- [ ] **P1** Handle capture errors
- [ ] **P1** Show error notification on failure
- [ ] **P2** Add screenshot preview option

### 6.3 History Hotkey Handler
- [ ] **P0** Implement handle_history_hotkey() function
- [ ] **P0** Create or get HistoryDialog instance
- [ ] **P0** Call dialog.show()
- [ ] **P0** Load history entries
- [ ] **P1** Handle dialog already open
- [ ] **P1** Focus existing dialog if open
- [ ] **P2** Add dialog animation

### 6.4 Phase 6 Validation
- [ ] **P0** Daemon starts without errors
- [ ] **P0** All hotkeys work correctly
- [ ] **P0** Clipboard monitoring runs in background
- [ ] **P0** Screenshots save to database
- [ ] **P0** Text copies save to database
- [ ] **P1** No crashes during normal operation
- [ ] **P1** No memory leaks over 24 hours
- [ ] **P1** Graceful shutdown on Ctrl+C
- [ ] **P2** Daemon auto-restarts on crash

---

## Phase 7: Notifications & Polish (Hours 56-60)

### 7.1 Notification System (`src/notifications.rs`)
- [ ] **P0** Define notification functions
- [ ] **P0** Implement notify_screenshot_success()
- [ ] **P0** Implement notify_screenshot_error()
- [ ] **P0** Implement notify_clipboard_restored()
- [ ] **P0** Set notification icons
- [ ] **P0** Set notification timeouts
- [ ] **P1** Add notification urgency levels
- [ ] **P1** Make notifications configurable
- [ ] **P2** Add notification sounds

### 7.2 Error Handling
- [ ] **P0** Replace all unwrap() with proper error handling
- [ ] **P0** Use anyhow::Result for application errors
- [ ] **P0** Use thiserror for custom error types
- [ ] **P0** Add error context with .context()
- [ ] **P1** Implement logging with tracing crate
- [ ] **P1** Set up log levels (info, warn, error)
- [ ] **P1** Write logs to file (optional)
- [ ] **P1** Handle all error cases gracefully
- [ ] **P2** Add error recovery mechanisms

### 7.3 Configuration Reload
- [ ] **P1** Implement config file watching
- [ ] **P1** Reload config on file change
- [ ] **P1** Re-register hotkeys on config change
- [ ] **P2** Validate config before reload
- [ ] **P2** Show notification on config reload

### 7.4 Performance Optimization
- [ ] **P1** Profile with flamegraph
- [ ] **P1** Identify bottlenecks
- [ ] **P1** Optimize database queries
- [ ] **P1** Optimize image encoding
- [ ] **P2** Memory profiling with valgrind
- [ ] **P2** Reduce allocations in hot paths

### 7.5 Phase 7 Validation
- [ ] **P0** All operations show notifications
- [ ] **P0** Errors display helpful messages
- [ ] **P1** Logs written to appropriate location
- [ ] **P1** No crashes on invalid input
- [ ] **P1** Config reload works correctly
- [ ] **P2** Performance meets targets

---

## Testing Tasks

### Unit Tests
- [ ] **P0** Config parsing tests (valid cases)
- [ ] **P0** Config parsing tests (invalid cases)
- [ ] **P0** Database insert tests
- [ ] **P0** Database query tests
- [ ] **P0** Database delete tests
- [ ] **P0** PNG encoding tests
- [ ] **P0** PNG decoding tests
- [ ] **P0** Hotkey parsing tests
- [ ] **P0** Rectangle calculation tests
- [ ] **P1** Error handling tests
- [ ] **P1** Edge case tests
- [ ] **P2** Property-based tests

### Integration Tests
- [ ] **P0** Screenshot → clipboard → database flow
- [ ] **P0** Text copy → history → restore flow
- [ ] **P1** Search functionality test
- [ ] **P1** Config reload test
- [ ] **P1** Multi-monitor test
- [ ] **P2** Long-running stability test

### Manual Testing
- [ ] **P0** Single monitor screenshot
- [ ] **P0** Multi-monitor screenshot
- [ ] **P0** HiDPI/fractional scaling
- [ ] **P0** Clipboard paste in Firefox
- [ ] **P0** Clipboard paste in GIMP
- [ ] **P0** Clipboard paste in LibreOffice
- [ ] **P0** Clipboard paste in VSCode
- [ ] **P0** History search with various queries
- [ ] **P0** Config file modification
- [ ] **P1** Daemon restart (history persists)
- [ ] **P1** System reboot (history persists)
- [ ] **P1** Paste in 10+ applications
- [ ] **P2** 24-hour stress test
- [ ] **P2** 1000+ clipboard operations

---

## Documentation Tasks

### Code Documentation
- [ ] **P1** Add module-level doc comments
- [ ] **P1** Add function doc comments
- [ ] **P1** Add struct doc comments
- [ ] **P1** Add inline comments for complex logic
- [ ] **P1** Generate API docs (cargo doc)
- [ ] **P2** Add code examples in docs

### User Documentation
- [ ] **P0** Create comprehensive README.md
- [ ] **P0** Add project description
- [ ] **P0** Add features list
- [ ] **P0** Add installation instructions
- [ ] **P0** Add usage guide
- [ ] **P0** Add configuration documentation
- [ ] **P1** Create INSTALL.md
- [ ] **P1** Create USAGE.md
- [ ] **P1** Add troubleshooting section
- [ ] **P1** Add FAQ section
- [ ] **P2** Add screenshots/GIFs
- [ ] **P2** Create video tutorial

### Developer Documentation
- [ ] **P1** Create CONTRIBUTING.md
- [ ] **P1** Add build instructions
- [ ] **P1** Add development setup guide
- [ ] **P1** Document architecture decisions
- [ ] **P2** Create CHANGELOG.md
- [ ] **P2** Create CODE_OF_CONDUCT.md

---

## Build & Distribution Tasks

### Build System
- [ ] **P0** Configure release profile in Cargo.toml
- [ ] **P0** Enable LTO (Link Time Optimization)
- [ ] **P0** Enable strip symbols
- [ ] **P0** Set codegen-units = 1
- [ ] **P1** Verify binary size (<20MB)
- [ ] **P1** Test release build
- [ ] **P2** Set up cross-compilation

### Packaging
- [ ] **P0** Create .desktop file
- [ ] **P0** Create systemd service file
- [ ] **P1** Create AppImage
- [ ] **P1** Create .deb package
- [ ] **P1** Create .rpm package
- [ ] **P1** Create AUR package (PKGBUILD)
- [ ] **P2** Create Flatpak manifest
- [ ] **P2** Create Snap package

### Distribution
- [ ] **P1** Create GitHub releases
- [ ] **P1** Upload binaries to releases
- [ ] **P1** Create installation script
- [ ] **P1** Test packages on Ubuntu 22.04
- [ ] **P1** Test packages on Fedora 39
- [ ] **P1** Test packages on Arch Linux
- [ ] **P2** Submit to AUR
- [ ] **P2** Submit to Flathub
- [ ] **P2** Submit to Snapcraft

---

## Quality Assurance Tasks

### Code Quality
- [ ] **P0** Run cargo fmt (format code)
- [ ] **P0** Run cargo clippy (linting)
- [ ] **P0** Fix all clippy warnings
- [ ] **P1** Run cargo audit (security)
- [ ] **P1** Fix security vulnerabilities
- [ ] **P1** Set up pre-commit hooks
- [ ] **P2** Set up CI/CD (GitHub Actions)

### Performance Testing
- [ ] **P1** Measure startup time (<2s)
- [ ] **P1** Measure screenshot latency (<500ms)
- [ ] **P1** Measure history dialog open time (<200ms)
- [ ] **P1** Measure idle memory usage (<50MB)
- [ ] **P1** Profile with flamegraph
- [ ] **P2** Profile with valgrind
- [ ] **P2** Optimize hot paths

### Security Audit
- [ ] **P1** Review all unsafe code blocks
- [ ] **P1** Check file permissions
- [ ] **P1** Validate all user inputs
- [ ] **P1** Check for SQL injection (use prepared statements)
- [ ] **P1** Review error messages (no sensitive data leak)
- [ ] **P2** Penetration testing
- [ ] **P2** Third-party security audit

---

## Deployment Tasks

### Installation
- [ ] **P0** Test installation on clean system
- [ ] **P0** Verify all dependencies installed
- [ ] **P0** Verify config directory created
- [ ] **P0** Verify database initialized
- [ ] **P1** Test systemd service enable/start
- [ ] **P1** Test auto-start on login
- [ ] **P1** Verify file permissions correct
- [ ] **P2** Test uninstall script

### User Onboarding
- [ ] **P1** Create first-run experience
- [ ] **P1** Show welcome notification
- [ ] **P1** Display keyboard shortcuts
- [ ] **P2** Create interactive tutorial
- [ ] **P2** Add tips and tricks

### Monitoring
- [ ] **P2** Add telemetry (opt-in)
- [ ] **P2** Track error rates
- [ ] **P2** Monitor performance metrics
- [ ] **P2** Set up crash reporting

---

## Post-Release Tasks

### Community
- [ ] **P1** Announce on Reddit (r/linux, r/rust)
- [ ] **P1** Post on Hacker News
- [ ] **P1** Share on Twitter/X
- [ ] **P1** Submit to awesome-rust list
- [ ] **P2** Create project website
- [ ] **P2** Set up Discord/Matrix server

### Maintenance
- [ ] **P1** Respond to issues on GitHub
- [ ] **P1** Review pull requests
- [ ] **P1** Fix critical bugs
- [ ] **P1** Update dependencies
- [ ] **P2** Add new features (from feedback)
- [ ] **P2** Improve documentation

### Future Enhancements
- [ ] **P2** Wayland support (via portals)
- [ ] **P2** OCR support (text from images)
- [ ] **P2** Cloud sync option
- [ ] **P2** Plugin system
- [ ] **P2** Advanced image editing
- [ ] **P2** Encrypted clipboard history

---

## Task Tracking Legend

**Priority Levels:**
- **P0 (Critical):** Must have for MVP, blocks other work
- **P1 (High):** Important for good user experience
- **P2 (Medium):** Nice to have, polish, future features

**Task States:**
- [ ] Not started
- [~] In progress
- [x] Completed
- [!] Blocked
- [-] Skipped/Deferred

---

## Estimated Time by Phase

| Phase | Hours | Key Deliverables |
|-------|-------|------------------|
| Phase 1: Foundation | 10 | Config, Database, Models |
| Phase 2: Screenshot | 15 | X11 capture, Overlay, PNG encoding |
| Phase 3: Clipboard | 10 | Clipboard ops, Monitoring |
| Phase 4: Hotkeys | 5 | Global hotkey registration |
| Phase 5: UI | 10 | GTK history dialog |
| Phase 6: Integration | 5 | Main daemon loop |
| Phase 7: Polish | 5 | Notifications, Error handling |
| **Total** | **60** | **Complete working application** |

---

## Quick Start Checklist (Minimum Viable Product)

For AI agent: Start with these tasks to get a working prototype:

- [ ] Project setup and dependencies
- [ ] Basic config loading
- [ ] Database schema creation
- [ ] X11 screenshot capture (single monitor)
- [ ] PNG encoding
- [ ] Copy to clipboard (image only)
- [ ] Store in database
- [ ] Global hotkey (screenshot only)
- [ ] Basic history dialog (show images)
- [ ] Click to restore
- [ ] Basic error handling

**MVP Estimate:** ~30 hours (50% of total tasks)

---

**Last Updated:** February 7, 2026  
**Total Tasks:** 87  
**Completion:** 0/87 (0%)
