# Architecture: MonoClip

## Tech Stack

| Technology | Role | Reason |
|---|---|---|
| Tauri 2.x | App shell + native bridge | Smallest binary, Rust OS APIs, window management |
| Rust 1.88+ | Backend logic | Clipboard watcher, global shortcuts, SQLite, tray |
| Svelte 5 (runes) | Frontend UI | Zero-runtime reactivity, tiny bundle, smooth animations |
| TypeScript | Type safety | Typed Tauri command bindings |
| Tailwind CSS 4 | Styling | Utility-first, no runtime, JIT |
| shadcn-svelte | UI primitives | Accessible components (dialog, tooltip, popover) |
| SQLite (rusqlite) | Local database | Zero-config, single-user, WAL mode |
| pnpm | Package manager | Fast, disk-efficient |

**Why NOT tauri-plugin-sql**: Direct rusqlite gives us full control over schema,
migrations, and query optimization. We embed migrations in Rust code.

## Project Structure

```
monoclip/
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/                    # App icons
│   └── src/
│       ├── main.rs               # Entry point, app builder
│       ├── lib.rs                # Library root (for tests)
│       ├── db/
│       │   ├── mod.rs            # DB module root
│       │   ├── connection.rs     # SQLite connection pool setup
│       │   ├── migrations.rs     # Embedded SQL migrations
│       │   ├── models.rs         # Rust structs (Folder, ClipItem, Settings)
│       │   └── queries.rs        # All SQL query functions
│       ├── clipboard/
│       │   ├── mod.rs
│       │   ├── watcher.rs        # Background polling thread
│       │   └── detector.rs      # Content type detection
│       ├── shortcuts/
│       │   ├── mod.rs
│       │   └── manager.rs        # Register/deregister global shortcuts
│       ├── tray/
│       │   ├── mod.rs
│       │   └── setup.rs          # System tray + menu
│       ├── window/
│       │   ├── mod.rs
│       │   └── manager.rs        # Show/hide/toggle window logic
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── folders.rs        # Tauri commands: folder CRUD
│       │   ├── clips.rs          # Tauri commands: clip CRUD
│       │   ├── settings.rs       # Tauri commands: settings R/W
│       │   └── utility.rs        # Tauri commands: stats, cleanup, window
│       └── state.rs              # AppState struct (DB pool, settings cache)
│
├── src/                          # Svelte frontend
│   ├── app.html
│   ├── app.css                   # Global styles + CSS variables
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/               # shadcn-svelte primitives (auto-generated)
│   │   │   ├── ClipCard.svelte   # Single clip item card
│   │   │   ├── ClipGrid.svelte   # Masonry/list grid of clips
│   │   │   ├── Sidebar.svelte    # Folder navigation sidebar
│   │   │   ├── SearchBar.svelte  # Unified search input
│   │   │   ├── FolderModal.svelte # Create/edit folder modal
│   │   │   ├── SettingsPanel.svelte
│   │   │   ├── Toast.svelte      # Transient copy confirmation
│   │   │   └── EmptyState.svelte
│   │   ├── stores/
│   │   │   ├── clips.svelte.ts   # Clips state (rune-based)
│   │   │   ├── folders.svelte.ts # Folders state
│   │   │   └── settings.svelte.ts
│   │   ├── api/
│   │   │   └── tauri.ts          # Typed wrappers around invoke()
│   │   └── utils/
│   │       ├── time.ts           # Relative timestamps
│   │       ├── animations.ts     # Spring/stagger helpers
│   │       └── keyboard.ts       # Keyboard nav utilities
│   └── routes/
│       └── +page.svelte          # Main app shell (SvelteKit-style OR single page)
│
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tailwind.config.ts
├── tsconfig.json
├── requirements.md
├── architecture.md
└── README.md
```

## Database Schema (SQL)

```sql
-- migrations/001_initial.sql

CREATE TABLE IF NOT EXISTS folders (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    icon        TEXT    NOT NULL DEFAULT '📋',
    color       TEXT    NOT NULL DEFAULT '#6366f1',
    global_shortcut TEXT,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Inbox is always id=1 (AUTOINCREMENT starts at 1)
INSERT OR IGNORE INTO folders (id, name, icon, color, position)
VALUES (1, 'Inbox', '📋', '#6366f1', 0);

CREATE TABLE IF NOT EXISTS clip_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    content      TEXT    NOT NULL,
    content_type TEXT    NOT NULL DEFAULT 'text',
    preview      TEXT    NOT NULL DEFAULT '',
    folder_id    INTEGER NOT NULL DEFAULT 1,
    is_pinned    INTEGER NOT NULL DEFAULT 0,
    is_deleted   INTEGER NOT NULL DEFAULT 0,
    source_app   TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET DEFAULT
);

CREATE INDEX IF NOT EXISTS idx_clips_folder ON clip_items(folder_id);
CREATE INDEX IF NOT EXISTS idx_clips_deleted_created ON clip_items(is_deleted, created_at);
CREATE INDEX IF NOT EXISTS idx_clips_content ON clip_items(content);

CREATE TABLE IF NOT EXISTS settings (
    id                  INTEGER PRIMARY KEY DEFAULT 1,
    master_shortcut     TEXT    NOT NULL DEFAULT 'CmdOrCtrl+Shift+V',
    auto_clean_enabled  INTEGER NOT NULL DEFAULT 1,
    auto_clean_days     INTEGER NOT NULL DEFAULT 30,
    max_history_items   INTEGER NOT NULL DEFAULT 500,
    paste_on_click      INTEGER NOT NULL DEFAULT 1,
    theme               TEXT    NOT NULL DEFAULT 'system',
    launch_at_login     INTEGER NOT NULL DEFAULT 0,
    ignored_apps        TEXT    NOT NULL DEFAULT '[]',
    updated_at          TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO settings (id) VALUES (1);
```

## Rust Module Dependency Graph

```
MODULE              DEPENDS ON              WAVE
db::models          (none)                  1
db::migrations      (none)                  1
db::connection      db::migrations          2
db::queries         db::connection, models  3
clipboard::detector (none)                  1
state               db::connection          3
commands::folders   db::queries, state      4
commands::clips     db::queries, state      4
commands::settings  db::queries, state      4
commands::utility   db::queries, state      4
clipboard::watcher  db::queries, state      4
shortcuts::manager  commands, state         5
tray::setup         commands, state         5
window::manager     state                   4
main                ALL                     6
```

## Frontend Architecture

### State Management (Svelte 5 Runes)

```typescript
// stores/folders.svelte.ts
let folders = $state<Folder[]>([]);
let activeFolder = $state<number>(1); // Inbox

// stores/clips.svelte.ts
let clips = $state<ClipItem[]>([]);
let searchQuery = $state('');
let isLoading = $state(false);
let flashingId = $state<number | null>(null); // for copy flash animation
```

### Tauri Event Listeners

```typescript
// Main window listens for:
listen('clip:new', handler)      // new clip saved → prepend to clips list
listen('folder:saved', handler)  // folder shortcut triggered → show toast
listen('cleanup:done', handler)  // auto-cleanup ran → show count
```

### Component Hierarchy

```
+page.svelte (app shell)
├── SearchBar.svelte
├── Sidebar.svelte
│   ├── FolderItem.svelte (×N)
│   └── FolderModal.svelte (portal)
├── ClipGrid.svelte
│   ├── ClipCard.svelte (×N)
│   └── EmptyState.svelte
├── SettingsPanel.svelte (slide-in panel)
└── Toast.svelte (overlay, portal)
```

### Keyboard Navigation Plan

```
Global:
  Cmd+F / /    → focus search
  Esc          → close window / close modal / deselect
  Tab          → sidebar → clips → settings button

Sidebar focused:
  ↑ ↓          → navigate folders
  Enter        → select folder
  N            → new folder

Clips focused:
  ↑ ↓ ← →     → navigate cards
  Enter        → copy + flash + (optional paste)
  P            → toggle pin
  Delete       → soft-delete (with undo toast)
  M            → move to folder (folder picker popover)
```

## Tauri Configuration (tauri.conf.json key settings)

```json
{
  "app": {
    "withGlobalTauri": true,
    "trayIcon": { "iconPath": "icons/tray@2x.png" }
  },
  "bundle": {
    "active": true,
    "identifier": "com.monoes.monoclip",
    "macOS": { "minimumSystemVersion": "13.0" }
  },
  "windows": [{
    "label": "main",
    "title": "MonoClip",
    "width": 680,
    "height": 520,
    "resizable": true,
    "minWidth": 480,
    "minHeight": 400,
    "decorations": false,
    "transparent": true,
    "alwaysOnTop": true,
    "skipTaskbar": true,
    "visible": false,
    "focus": true,
    "hiddenTitle": true,
    "titleBarStyle": "Transparent"
  }]
}
```

## Animation Spec

```
Window Reveal:
  - transform: scale(0.96) → scale(1.0)
  - opacity: 0 → 1
  - duration: 180ms, easing: cubic-bezier(0.34, 1.56, 0.64, 1) (spring)

Clip Card Stagger:
  - Each card: opacity 0 → 1, translateY 8px → 0
  - Delay: index * 30ms (capped at 300ms)
  - Duration: 200ms, easing: ease-out

Copy Flash:
  - Background: current → accent (#6366f1 20% opacity) → current
  - Duration: 150ms total, 2-keyframe

Sidebar Active Indicator:
  - Left border slides via `translate` transition
  - Duration: 120ms, easing: ease-out
```

## Build Order (Implementation Waves)

```
Wave 1 — Project Scaffold:
  Tauri + Svelte project init, all deps, basic window opens

Wave 2 — Database Layer:
  db/connection.rs, db/migrations.rs, db/models.rs, db/queries.rs

Wave 3 — State + Commands (parallel):
  state.rs
  commands/folders.rs
  commands/clips.rs
  commands/settings.rs
  commands/utility.rs

Wave 4 — System Integration (parallel):
  clipboard/watcher.rs + detector.rs
  shortcuts/manager.rs
  tray/setup.rs
  window/manager.rs

Wave 5 — Frontend Core (parallel):
  lib/api/tauri.ts
  lib/stores/*.svelte.ts
  Sidebar.svelte
  SearchBar.svelte

Wave 6 — Frontend Features (parallel):
  ClipCard.svelte + ClipGrid.svelte
  FolderModal.svelte
  SettingsPanel.svelte
  Toast.svelte + EmptyState.svelte
  +page.svelte (wiring)

Wave 7 — Polish:
  Animations, keyboard nav, error states, README
```
