# Product Requirements Document (PRD)
## Area Screenshot & Clipboard Manager for Linux

**Version:** 1.0  
**Last Updated:** February 7, 2026  
**Project Status:** Planning Phase

---

## 1. Executive Summary

### 1.1 Product Vision
A lightweight, keyboard-driven Linux utility that unifies area screenshot capture and clipboard history management into a single tool. The product aims to provide macOS/Windows-level clipboard functionality on Linux with minimal resource usage.

### 1.2 Product Goals
- Eliminate the need for multiple clipboard/screenshot tools
- Provide instant area screenshot → clipboard workflow
- Maintain searchable history of clipboard content (text + images)
- Remain under 50MB RAM usage during idle
- Support both X11 and Wayland (X11 first)

### 1.3 Success Metrics
- Screenshot capture latency < 500ms
- Clipboard history retrieval < 200ms
- Zero crashes during 7-day continuous operation
- User adoption by 1000+ active users within 6 months
- 4+ star rating on GitHub

---

## 2. User Personas

### 2.1 Primary Persona: "Developer Dave"
- **Role:** Software Engineer
- **OS:** Ubuntu 22.04 / Arch Linux
- **Pain Points:**
  - Constantly screenshots code snippets for documentation
  - Loses clipboard history when copying multiple items
  - Existing tools save files instead of using clipboard
- **Needs:**
  - Fast keyboard shortcuts
  - Clipboard history with text and images
  - No GUI bloat

### 2.2 Secondary Persona: "Designer Dana"
- **Role:** UI/UX Designer
- **OS:** Fedora / Pop!_OS
- **Pain Points:**
  - Takes dozens of screenshots daily
  - Needs to reference previous clipboard items
  - File-based screenshot tools clutter Downloads folder
- **Needs:**
  - Visual thumbnail history
  - Quick re-access to previous screenshots
  - Clean, minimal interface

---

## 3. Functional Requirements

### 3.1 Area Screenshot Capture

#### FR-1.1: Global Shortcut Activation
- **Requirement:** System responds to `Ctrl + Win + S` globally
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Shortcut works in any application
  - Response time < 300ms
  - Does not interfere with application shortcuts
  - Shortcut is configurable via config file

#### FR-1.2: Interactive Area Selection
- **Requirement:** User selects rectangular screen area
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Screen dims/freezes on activation
  - Fullscreen transparent overlay appears
  - Mouse cursor changes to crosshair
  - Live rectangle drawn during drag
  - Rectangle shows dimensions in pixels
  - ESC cancels selection
  - Click without drag cancels selection

#### FR-1.3: Screenshot Capture
- **Requirement:** Capture selected area as PNG
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Captures exact pixel area selected
  - Supports multi-monitor setups
  - Handles HiDPI/scaling correctly
  - Captures at full resolution
  - Format: PNG with transparency support

#### FR-1.4: Clipboard Integration
- **Requirement:** Copy screenshot directly to clipboard
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Image available as `image/png` MIME type
  - Pasteable in all major applications (Firefox, Chrome, LibreOffice, GIMP)
  - No intermediate file required
  - Clipboard ownership maintained until new copy action

#### FR-1.5: Visual Feedback
- **Requirement:** Provide user feedback after capture
- **Priority:** P1 (High)
- **Acceptance Criteria:**
  - Brief notification/flash on successful capture
  - Audio feedback (optional, configurable)
  - Error notification if capture fails

### 3.2 Clipboard History Management

#### FR-2.1: Automatic History Tracking
- **Requirement:** Monitor and store clipboard changes
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Detects all clipboard copy operations (Ctrl+C)
  - Stores text content (UTF-8)
  - Stores image content (PNG)
  - Ignores duplicate consecutive entries
  - Runs as background daemon
  - CPU usage < 5% during monitoring

#### FR-2.2: History Dialog Access
- **Requirement:** Open history via `Win + H`
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Dialog appears within 200ms
  - Shows last 50 entries by default
  - Displays images as thumbnails (max 200x200px)
  - Shows text preview (first 100 characters)
  - Includes timestamp for each entry
  - Keyboard navigable (arrow keys)

#### FR-2.3: History Search
- **Requirement:** Search clipboard history
- **Priority:** P1 (High)
- **Acceptance Criteria:**
  - Search box at top of dialog
  - Real-time filtering as user types
  - Searches text content
  - Searches OCR text in images (future: P2)
  - Case-insensitive search

#### FR-2.4: Clipboard Restoration
- **Requirement:** Re-copy items from history
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Single click/Enter selects item
  - Selected item copied to clipboard
  - Dialog closes after selection
  - User can paste with Ctrl+V
  - Original MIME type preserved

#### FR-2.5: History Persistence
- **Requirement:** Store history between sessions
- **Priority:** P1 (High)
- **Acceptance Criteria:**
  - History survives daemon restart
  - History survives system reboot
  - SQLite database in `~/.config/clipboard-capture/`
  - Images stored as PNG blobs
  - Configurable retention (default: 7 days)
  - Configurable max entries (default: 500)

#### FR-2.6: History Management
- **Requirement:** Delete/clear history items
- **Priority:** P2 (Medium)
- **Acceptance Criteria:**
  - Delete single item via context menu
  - Clear all history via dialog button
  - Confirm before clearing all
  - Deleted items removed from database

### 3.3 Text Clipboard Support

#### FR-3.1: Text Capture
- **Requirement:** Track text clipboard entries
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Detects text/plain clipboard type
  - Stores full UTF-8 text
  - Handles multi-line text
  - Preserves formatting (tabs, newlines)
  - Max text size: 10MB per entry

#### FR-3.2: Text Display
- **Requirement:** Show text previews in history
- **Priority:** P0 (Critical)
- **Acceptance Criteria:**
  - Shows first 100 characters
  - Indicates truncation with "..."
  - Shows full text on hover/selection
  - Preserves line breaks in preview
  - Monospace font for code snippets

---

## 4. Non-Functional Requirements

### 4.1 Performance
- Screenshot capture latency: < 500ms (P0)
- History dialog open time: < 200ms (P0)
- Daemon idle memory: < 50MB (P0)
- Database query time: < 50ms (P1)
- Startup time: < 2 seconds (P1)

### 4.2 Reliability
- Zero data loss during normal operation (P0)
- Graceful handling of out-of-memory (P1)
- Auto-recovery from crashes (P1)
- Uptime: 99.9% during 30-day period (P1)

### 4.3 Usability
- Zero-configuration startup (P0)
- All features accessible via keyboard (P0)
- Clear visual feedback for all actions (P1)
- Error messages are actionable (P1)

### 4.4 Compatibility
- X11 display server (P0)
- Wayland support (P2 - future)
- Multi-monitor support (P0)
- HiDPI/fractional scaling (P1)
- Distribution-agnostic (P0)

### 4.5 Security & Privacy
- No network access (P0)
- User-initiated actions only (P0)
- Local-only data storage (P0)
- No telemetry/analytics (P0)
- Clipboard data encrypted at rest (P2)

---

## 5. User Stories

### 5.1 Screenshot Workflows

**US-1:** Quick Code Snippet Capture
```
AS a developer
I WANT to quickly screenshot a code snippet
SO THAT I can paste it directly into Slack without saving a file

GIVEN I'm viewing code in VS Code
WHEN I press Ctrl+Win+S and select the code area
THEN the screenshot is copied to my clipboard
AND I can paste it directly into Slack
```

**US-2:** Multi-Screenshot Documentation
```
AS a technical writer
I WANT to take multiple screenshots in sequence
SO THAT I can paste them into a document in order

GIVEN I need to capture 5 different UI states
WHEN I take 5 screenshots using Ctrl+Win+S
THEN all 5 are stored in clipboard history
AND I can access them via Win+H to paste in sequence
```

### 5.2 Clipboard History Workflows

**US-3:** Retrieve Previous Copy
```
AS a user
I WANT to access something I copied 5 minutes ago
SO THAT I don't have to find and re-copy it

GIVEN I copied a URL earlier
WHEN I press Win+H and select the URL from history
THEN the URL is copied back to my clipboard
AND I can paste it with Ctrl+V
```

**US-4:** Search Historical Clipboard
```
AS a power user
I WANT to search my clipboard history
SO THAT I can find a specific copied item from yesterday

GIVEN I copied 50+ items today
WHEN I open clipboard history and type "API key"
THEN only entries containing "API key" are shown
AND I can quickly select the correct one
```

---

## 6. User Interface Specifications

### 6.1 Screenshot Overlay
- **Layout:** Fullscreen transparent overlay
- **Background:** Dimmed (50% opacity black)
- **Cursor:** Crosshair
- **Selection Rectangle:**
  - Border: 2px solid blue (#007AFF)
  - Background: Semi-transparent blue (20% opacity)
  - Dimensions label: Top-left corner, white text, 14px
  - Format: "1920 × 1080"

### 6.2 Clipboard History Dialog
- **Window Size:** 600px × 400px (resizable)
- **Position:** Center of screen
- **Components:**
  - **Search Bar:** Top, full width, placeholder "Search clipboard..."
  - **Entry List:** Scrollable grid/list
    - Images: Thumbnail view (150px × 150px max)
    - Text: List view with preview
  - **Status Bar:** Bottom, shows "X items" count
  - **Shortcuts:** Display at bottom (e.g., "Enter: Copy, Del: Remove")

### 6.3 Notifications
- **Style:** System native notifications
- **Duration:** 2 seconds
- **Messages:**
  - "Screenshot copied to clipboard"
  - "Clipboard history is empty"
  - "Error: Failed to capture screenshot"

---

## 7. Configuration

### 7.1 Config File Location
`~/.config/clipboard-capture/config.toml`

### 7.2 Configuration Options
```toml
[shortcuts]
screenshot = "Ctrl+Super+S"
history = "Super+H"

[capture]
format = "png"  # png, jpg
quality = 95    # 1-100 (for jpg)
show_dimensions = true

[history]
max_entries = 500
retention_days = 7
auto_cleanup = true

[storage]
database_path = "~/.config/clipboard-capture/history.db"
image_storage = "database"  # database, files

[ui]
theme = "auto"  # auto, light, dark
thumbnail_size = 150
notification_duration = 2

[privacy]
exclude_passwords = true  # Don't store password manager clipboard
```

---

## 8. Technical Constraints

### 8.1 Platform Requirements
- Linux kernel 4.0+
- X11 display server (Wayland: future)
- GTK 3.0+ or Qt 5.0+
- Python 3.8+ OR Rust 1.60+ OR C++17

### 8.2 Dependencies (Maximum)
- Display server libraries (Xlib/xcb)
- Image processing (Cairo/libpng)
- Clipboard management (X11 Selection)
- Database (SQLite3)
- UI toolkit (GTK3/Qt5)
- Notification system (libnotify)

### 8.3 Resource Limits
- Maximum binary size: 20MB
- Maximum memory usage: 200MB with 500 history items
- Maximum startup time: 3 seconds
- Maximum database size: 1GB (configurable)

---

## 9. Milestones & Roadmap

### Phase 1: MVP (X11 Only)
- [ ] Global shortcut registration
- [ ] Area screenshot capture
- [ ] Clipboard integration (image only)
- [ ] Basic history storage (in-memory)
- [ ] History dialog (basic)
- [ ] Configuration file support

### Phase 2: Full Features
- [ ] Text clipboard support
- [ ] SQLite persistence
- [ ] Search functionality
- [ ] History management (delete)
- [ ] Multi-monitor support
- [ ] HiDPI support
- [ ] System notifications

### Phase 3: Polish
- [ ] Wayland support (via portals)
- [ ] OCR support (optional)
- [ ] Themes (light/dark)
- [ ] Advanced search filters
- [ ] Export/import history
- [ ] Plugin system

---

## 10. Open Questions

1. **Q:** Should we support video clipboard (rare on Linux)?  
   **A:** No - out of scope for v1.0

2. **Q:** Should history sync across machines?  
   **A:** No - adds complexity, local-only for v1.0

3. **Q:** Should we include built-in image editing?  
   **A:** No - use external tools, keep scope minimal

4. **Q:** Maximum image resolution to support?  
   **A:** 8K (7680×4320) - covers 99% of displays

5. **Q:** Should we encrypt clipboard history?  
   **A:** Optional in future, not v1.0

---

## 11. Success Criteria

### 11.1 Launch Criteria
- All P0 requirements implemented
- Zero critical bugs
- Tested on Ubuntu, Fedora, Arch
- Documentation complete
- Passing automated tests (>80% coverage)

### 11.2 Post-Launch Success
- 1000+ GitHub stars in 6 months
- <5 crash reports per 1000 users
- Average 4+ star user rating
- Active community contributions

---

## 12. Risks & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Wayland compatibility issues | High | High | Start with X11, use portals for Wayland |
| Clipboard ownership conflicts | Medium | Medium | Implement robust clipboard handling |
| Memory leaks in long-running daemon | High | Medium | Extensive leak testing, memory profiling |
| Permission issues on some distros | Medium | Low | Clear documentation, fallback modes |

---

## 13. Out of Scope (v1.0)

- Video recording
- Cloud synchronization
- Mobile apps
- Browser extensions
- OCR (text extraction from images)
- Built-in image editing
- Collaborative features
- Web API

---

**Document Owner:** Engineering Team  
**Stakeholders:** Product, Engineering, QA  
**Next Review Date:** March 7, 2026
