# MonoClip

A blazing-fast, macOS-native clipboard manager built with Tauri 2 + Svelte 5.

## Features

- **System tray** — runs silently in the macOS menu bar
- **Multi-folder organization** — create custom folders (Code Snippets, Emails, Links…)
- **Global shortcut routing** — assign `CmdOrCtrl+Option+1` to save clipboard directly into a folder
- **Master shortcut** — `CmdOrCtrl+Shift+V` (default) toggles the main window
- **Glass-morphism UI** — frosted glass floating panel with spring animations
- **Keyboard-first** — full arrow-key navigation, `Enter` to copy, `P` to pin, `Delete` to remove
- **Auto-cleanup** — automatically removes old unpinned items (configurable)
- **Content type detection** — URLs, emails, hex colors, code snippets, plain text
- **Deduplication** — identical consecutive clips don't create duplicates
- **Paste on click** — optionally auto-pastes into the active window via AppleScript

## Stack

| Layer | Tech |
|---|---|
| App framework | Tauri 2 |
| Backend | Rust 1.88+ |
| Database | SQLite (rusqlite, bundled) |
| Frontend | Svelte 5 + Vite |
| Styling | Tailwind CSS 3 |
| Package manager | pnpm |

## Project Structure

```
monoclip/
├── src-tauri/src/
│   ├── main.rs              # App entry point, plugin setup, setup hook
│   ├── state.rs             # AppState (DB pool)
│   ├── db/                  # SQLite: models, queries, migrations
│   ├── clipboard/           # Background watcher + content type detection
│   ├── commands/            # Tauri IPC commands (folders, clips, settings, utility)
│   ├── shortcuts/           # Global shortcut registration/deregistration
│   ├── tray/                # System tray icon + menu
│   └── window/              # Window positioning helpers
└── src/
    ├── App.svelte            # Root component: layout, event listeners, keyboard nav
    ├── lib/api/tauri.ts      # Typed invoke() wrappers for all commands
    ├── lib/stores/           # Svelte 5 rune-based reactive state
    ├── lib/components/       # UI components
    └── lib/utils/            # Time formatting, etc.
```

## Quick Start

### Prerequisites
- Rust 1.77+ (`rustup update`)
- Node.js 18+ + pnpm (`npm i -g pnpm`)
- Xcode Command Line Tools (macOS)
- `tauri-cli`: `cargo install tauri-cli --version "^2"`

### Run in development
```bash
pnpm install
cargo tauri dev
```

### Build for release
```bash
cargo tauri build
```

The `.app` bundle appears in `src-tauri/target/release/bundle/macos/`.

## Configuration

All settings are stored in `~/.monoclip/monoclip.db` (SQLite).

| Setting | Default | Description |
|---|---|---|
| Master shortcut | `CmdOrCtrl+Shift+V` | Toggle main window |
| Auto-cleanup | Enabled, 30 days | Remove old unpinned clips |
| Max history | 500 items | Hard cap on stored clips |
| Paste on click | Enabled | Simulate Cmd+V after copying |
| Theme | System | Light / Dark / System |

## Adding Custom Folders

1. Open MonoClip (`CmdOrCtrl+Shift+V`)
2. Click **+ New Folder** in the sidebar
3. Name it, pick an icon and color
4. Optionally assign a global shortcut (e.g., `CmdOrCtrl+Option+1`)
5. Press that shortcut anytime to save the current clipboard to that folder

## Permissions Required (macOS)

- **Accessibility** — needed for "Paste on Click" (simulates Cmd+V via AppleScript)
- **Input Monitoring** — needed for global keyboard shortcuts

Both are requested on first use via macOS system dialogs.

## Architecture Notes

- **Clipboard watcher** polls every 500ms in a background Rust thread. Hash comparison prevents duplicate saves.
- **Global shortcuts** use `tauri-plugin-global-shortcut`. Folder shortcuts save the current clipboard content directly to the target folder.
- **Database** uses WAL mode for safe concurrent access between the watcher thread and main thread.
- **Window** uses `ActivationPolicy::Accessory` (no Dock icon, no Cmd+Tab entry). Always-on-top, frameless, transparent background.
- **Animations** — Spring scale-in on window reveal (`cubic-bezier(0.34, 1.56, 0.64, 1)`), staggered card fade-in, accent flash on copy.
