# System Requirements: MonoClip

## 1. Overview
MonoClip is a macOS-native, blazing-fast clipboard manager desktop application built with
Tauri (Rust) and Svelte. It runs silently in the system menu bar, continuously monitors the
clipboard, and lets users organize saved clips into custom folders — each with optional
dedicated global shortcuts. The UI is invoked via a master global shortcut or menu bar click,
presenting a glass-morphism floating panel with full keyboard navigation.

## 2. User Roles
Single-user desktop app. No authentication. The OS user IS the user.

Permission model: all operations are local and trusted.

## 3. Data Model

### ENTITY: Folder
```
FIELDS:
  - id:             INTEGER   PK, autoincrement
  - name:           TEXT      NOT NULL, max 50 chars
  - icon:           TEXT      default "📋" (emoji)
  - color:          TEXT      default "#6366f1" (hex, for sidebar dot)
  - globalShortcut: TEXT      nullable (e.g. "CmdOrCtrl+Option+1")
  - position:       INTEGER   default 0 (for sidebar sort order)
  - createdAt:      DATETIME  default CURRENT_TIMESTAMP
  - updatedAt:      DATETIME  auto-updated on write

SPECIAL:
  - id=0 is the reserved "Inbox" folder (auto-created, not deletable)

INDEXES:
  - position (for ordered sidebar listing)
```

### ENTITY: ClipItem
```
FIELDS:
  - id:          INTEGER   PK, autoincrement
  - content:     TEXT      NOT NULL (raw clipboard text)
  - contentType: TEXT      default "text" — enum: text | url | code | email | color
  - preview:     TEXT      truncated to 200 chars for display
  - folderId:    INTEGER   FK → Folder.id, default 0 (Inbox)
  - isPinned:    BOOLEAN   default false (pinned items skip auto-cleanup)
  - isDeleted:   BOOLEAN   default false (soft delete)
  - sourceApp:   TEXT      nullable (e.g. "com.apple.Safari")
  - createdAt:   DATETIME  default CURRENT_TIMESTAMP
  - updatedAt:   DATETIME  auto-updated on write

INDEXES:
  - folderId (for folder-based queries)
  - isDeleted + createdAt (for auto-cleanup queries)
  - content LIKE search (full-text for search)

CONTENT TYPE DETECTION:
  - url:   starts with http:// or https://
  - email: matches email regex
  - color: matches #RRGGBB or #RGB
  - code:  contains { } or () or ; patterns suggesting code
  - text:  default fallback
```

### ENTITY: Settings (single row, id=1 always)
```
FIELDS:
  - id:                  INTEGER   PK = 1 always
  - masterShortcut:      TEXT      default "CmdOrCtrl+Shift+V"
  - autoCleanEnabled:    BOOLEAN   default true
  - autoCleanDays:       INTEGER   default 30
  - maxHistoryItems:     INTEGER   default 500
  - pasteOnClick:        BOOLEAN   default true
  - theme:               TEXT      default "system" — enum: light | dark | system
  - launchAtLogin:       BOOLEAN   default false
  - ignoredApps:         TEXT      JSON array of bundle IDs to ignore
  - updatedAt:           DATETIME  auto-updated on write
```

## 4. Tauri Commands (IPC Surface)

### Folder Commands
```
get_folders() → Vec<Folder>
  Returns all folders ordered by position, including Inbox (id=0)

create_folder(name: String, icon: String, color: String, shortcut: Option<String>) → Folder
  Validates: name non-empty, shortcut not already in use
  Side effect: registers global shortcut if provided

update_folder(id: i64, name: Option<String>, icon: Option<String>,
              color: Option<String>, shortcut: Option<String>) → Folder
  Validates: id != 0 (Inbox not renameable via this command)
  Side effect: re-registers shortcut if changed

delete_folder(id: i64) → ()
  Validates: id != 0 (Inbox not deletable)
  Side effect: moves all clips in folder to Inbox, deregisters shortcut

reorder_folders(ids: Vec<i64>) → ()
  Updates position field in order of provided ids
```

### Clip Commands
```
get_clips(folder_id: Option<i64>, search: Option<String>,
          limit: Option<i32>, offset: Option<i32>) → Vec<ClipItem>
  folder_id=None → all folders (for search)
  Default limit=50

get_clip(id: i64) → ClipItem

pin_clip(id: i64) → ()
unpin_clip(id: i64) → ()
delete_clip(id: i64) → ()          // soft delete (sets isDeleted=true)
restore_clip(id: i64) → ()         // undelete
hard_delete_clip(id: i64) → ()     // permanent delete

move_clip(id: i64, folder_id: i64) → ()

copy_to_clipboard(id: i64) → ()
  Sets system clipboard to clip.content
  If settings.pasteOnClick=true: simulate Cmd+V into active window

save_current_clipboard_to_folder(folder_id: i64) → ClipItem
  Reads current system clipboard content
  Deduplicates: if identical content already in folder, updates timestamp
  Saves and returns new ClipItem
```

### Settings Commands
```
get_settings() → Settings
update_settings(patch: SettingsPatch) → Settings
  Side effects:
  - If masterShortcut changed: re-register global shortcut
  - If launchAtLogin changed: update OS autostart entry
  - If autoCleanEnabled/Days changed: re-schedule cleanup job
```

### Utility Commands
```
run_auto_cleanup() → i64        // returns count of deleted items
get_stats() → AppStats          // { total_clips, folders_count, pinned_count, db_size_bytes }
show_main_window() → ()         // called from tray click
hide_main_window() → ()
toggle_main_window() → ()
```

## 5. Business Rules

### Clipboard Watcher
- Poll system clipboard every 500ms
- On change detection: compare hash of new content vs last-saved hash
- Ignore change if: content is empty, content length > 50000 chars, source app is in ignoredApps
- Auto-detect content type
- Save to Inbox (folder_id=0)
- Emit Tauri event "clip:new" with the ClipItem to update UI reactively
- Do NOT save duplicate consecutive clips (same content as most recent Inbox item)

### Global Shortcut: Folder Routing
- When folder shortcut triggered: read current system clipboard → save_current_clipboard_to_folder
- Visual feedback: emit event "folder:saved" with folder name for UI toast

### Auto-Cleanup
- Run on app startup (if enabled)
- Run daily via background timer
- Delete (hard delete) non-pinned items older than autoCleanDays in all folders
- Exception: never delete pinned items
- Exception: always keep the most recent 10 items in Inbox regardless of age
- Emit event "cleanup:done" with count

### Window Behavior
- Main window is a frameless, always-on-top floating panel
- Hides when it loses focus (clicked outside) — configurable
- Does NOT appear in Cmd+Tab application switcher
- Does NOT appear in Dock
- Activates on global shortcut or tray click

### Inbox
- Folder id=0 "Inbox" is auto-created on first launch
- Cannot be deleted or renamed
- Is the default destination for clipboard watcher saves
- Can have items moved out of it into other folders

### Content Deduplication
- Before saving any clip: check if identical content already exists in same folder
- If duplicate found: update its updatedAt timestamp instead of creating new entry
- This keeps the "most recent" float to the top

## 6. Pages & UI Flows

### SCREEN: Main Window (Floating Panel)
```
Layout:
  - Frameless, 680×520px default, resizable
  - Semi-transparent frosted glass background (vibrancy)
  - Rounded corners (16px), subtle drop shadow

Zones:
  ┌────────────────────────────────────────────┐
  │  [🔍 Search bar — full width at top]       │
  ├──────────────┬─────────────────────────────┤
  │  Sidebar     │  Clips Panel                │
  │  (200px)     │  (scrollable grid/list)     │
  │              │                             │
  │  📋 Inbox    │  [ClipCard] [ClipCard]      │
  │  📁 Folder1  │  [ClipCard] [ClipCard]      │
  │  📁 Folder2  │  [ClipCard] ...             │
  │  ─────────   │                             │
  │  ⚙ Settings │                             │
  └──────────────┴─────────────────────────────┘

Keyboard Nav:
  - Tab: cycles between search, sidebar, clips panel
  - Arrow keys: navigate within focused zone
  - Enter: activate selected item (copy/open)
  - Cmd+F or /: focus search bar
  - Esc: close/hide window
  - Delete/Backspace: delete selected clip (with confirmation)
  - P: toggle pin on selected clip

States:
  - Loading: skeleton cards with shimmer animation
  - Empty: illustration + "Nothing saved yet" message
  - Search empty: "No clips match '[query]'"
  - Error: "Something went wrong" with retry button
```

### SCREEN: Sidebar
```
Components:
  - Folder list (scrollable if many)
  - Each item: color dot + icon + name + item count badge
  - Active folder: highlighted with accent background
  - Inbox always first
  - "Add Folder" button at bottom
  - "Settings" gear at very bottom

User Actions:
  - Click folder → filters clips panel
  - Right-click folder → context menu (rename, assign shortcut, delete)
  - Drag folder → reorder
  - Click "Add Folder" → inline creation form
```

### SCREEN: Clips Panel
```
Layout: masonry grid (2 columns on wider window, 1 on narrow)
  Or: compact list toggle

Components per ClipCard:
  - Content preview (max 4 lines, monospace for code, truncated)
  - Content type icon (🔗 url, 📧 email, 🎨 color swatch, </> code)
  - Relative timestamp ("2m ago", "yesterday")
  - Source app icon (if available)
  - Pin indicator (📌 if pinned)
  - Hover actions: copy button, pin button, move button, delete button

User Actions:
  - Click card → copy to clipboard + visual "pop" flash animation + (optional) auto-paste
  - Cmd+Click → copy without closing window
  - Right-click → context menu (copy, pin, move to folder, delete)
  - Hover → show hover actions
```

### SCREEN: Folder Creation/Edit Modal
```
Fields:
  - Name (text input, required)
  - Icon (emoji picker, 5×5 grid of common emojis)
  - Color (6 preset colors + custom hex)
  - Global Shortcut (keyboard recorder input)

Actions:
  - Save, Cancel
  - Shortcut conflict detection (show warning if shortcut already used)
```

### SCREEN: Settings Panel
```
Sections:
  1. General
     - Launch at login toggle
     - Theme selector (light/dark/system)
     - Hide window on focus loss toggle

  2. Clipboard
     - Paste on click toggle
     - Max history items slider (50–2000)
     - Ignored apps (add/remove list)

  3. Shortcuts
     - Master shortcut recorder
     - Per-folder shortcuts (table view)

  4. Auto-cleanup
     - Enable/disable toggle
     - Days slider (1–365)
     - "Clean Now" button → shows count deleted

  5. About
     - Version, GitHub link
```

## 7. Non-Functional Requirements

### Performance
- App startup: < 200ms to tray ready
- Window reveal: < 50ms visual response (pre-render content)
- Clipboard polling: 500ms interval, < 1ms CPU per poll
- Search: < 50ms for up to 10,000 clips
- DB operations: < 10ms for all queries

### Storage
- SQLite database at: `~/.monoclip/monoclip.db`
- Images stored as base64 in content field (future: file references)
- Max DB size soft limit: 100MB (warn user)

### Security
- All data local, no network calls
- SQLite WAL mode for concurrent access safety
- Parameterized queries only (no string interpolation)

### Animations
- Window reveal: spring scale from 0.95 → 1.0 + opacity 0 → 1, duration 200ms
- List item stagger: each card fades in with 30ms offset
- Copy flash: card briefly flashes accent color (150ms)
- Sidebar active: slide indicator transitions between items

### macOS Specifics
- NSVisualEffectView / vibrancy effect (`.hudWindow` material)
- Activation policy: `.accessory` (no Dock icon, no Cmd+Tab)
- Window level: `.floating`
- Support for Dark/Light mode switching

### Tauri Plugin Dependencies
- `tauri-plugin-global-shortcut@2` — global hotkeys
- `tauri-plugin-clipboard-manager@2` — read/write clipboard
- `tauri-plugin-autostart@2` — launch at login
- `tauri-plugin-sql@2` (with SQLite feature) — database
- `tauri-plugin-store@2` — settings fallback / migration

## 8. Tech Decisions

| Technology | Role | Reason |
|---|---|---|
| Tauri 2.x | App framework | Rust backend, tiny binary, native OS APIs |
| Rust | Backend logic | Clipboard watcher, shortcuts, DB, tray |
| Svelte 5 + Vite | Frontend UI | Minimal JS, fast reactivity, rune-based |
| TypeScript | Frontend types | Type safety for Tauri command calls |
| Tailwind CSS 4 | Styling | Utility-first, no runtime CSS |
| shadcn-svelte | UI components | Accessible, unstyled primitives |
| SQLite (via tauri-plugin-sql) | Persistence | Local, zero-config, fast for single-user |
| pnpm | Package manager | Fast, disk-efficient |
