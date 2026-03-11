# MonoClip — Feature Roadmap & Ideas

> Rough priority tiers: **P1** = next release · **P2** = near-term · **P3** = longer-term / experimental

---

## 🔄 Multi-Device Sync

### Google Drive Sync *(P1)*
- Store `monoclip.db` (or a sync-friendly export) in a user-chosen Google Drive folder
- Merge strategy: last-write-wins per clip, pins always survive
- Background sync daemon — push on new clip, pull on app focus
- Conflict resolution UI: show diffs when the same clip was edited on two machines
- Auth via OAuth2 in a native macOS browser sheet (no external credentials stored in plaintext)

### iCloud Drive Sync *(P2)*
- Use `NSUbiquitousKeyValueStore` for lightweight settings sync
- Mirror `~/.monoclip/` to `~/Library/Mobile Documents/com.monoes.monoclip/`
- Zero-config — just works if iCloud Drive is enabled

### Dropbox / Custom WebDAV *(P3)*
- Generic "sync folder" setting — point it at any folder (Dropbox, Nextcloud, Synology)
- Periodic file-based merge (no server needed)

---

## 🖥️ UI & UX

### Quick-Paste Number Keys *(P1)*
- Press `1`–`9` while MonoClip is open to instantly paste that clip without clicking
- Top 9 clips get a subtle number badge

### Preview Drawer *(P1)*
- Click or hover-hold a clip to expand a full preview pane on the right
- Show full text, syntax-highlighted code, full-size image, or file metadata

### Rich Text & HTML Support *(P2)*
- Preserve bold, italic, links when copying from web / docs
- Toggle between "plain" and "rich" view per clip
- Paste as plain text with `⌥Enter`

### Clip Tags *(P2)*
- Tag clips inline (e.g. `#work`, `#code`, `#password`)
- Filter bar for tags alongside folders
- Auto-tag rules (e.g. URLs → `#link`, code → `#code`)

### Stacked / Merge Paste *(P2)*
- Select multiple clips with `⇧Click` and paste them all in sequence
- Great for building up multi-part content without switching windows

### Themes & Appearance *(P2)*
- Light mode that actually looks good
- Accent colour picker (not just indigo)
- Compact mode: tighter rows, smaller font, fits more clips on screen
- Translucency slider (for the glass effect intensity)

### Clip Notes *(P3)*
- Add a private annotation to any clip — shown as a small tooltip
- Useful for documenting what a snippet is for

---

## 🤖 AI Features

### Smart Auto-Categorise *(P2)*
- On save, run a local model (mlx / llama.cpp) or call an API to suggest a folder
- User can confirm or dismiss — trains a lightweight preference model over time

### Clip Summarise *(P2)*
- Right-click → "Summarise" — calls Claude/GPT to give a one-line summary
- Summary stored as the clip's note, shown in preview

### Transform Actions *(P2)*
- Right-click → Transform: Translate, Fix grammar, Convert JSON→YAML, Minify, Prettify
- Runs against the clip content, creates a new clip with the result

### Natural Language Search *(P3)*
- "the SQL query from yesterday" → semantic vector search over clip content
- Embeddings stored locally via sqlite-vec

---

## ⌨️ Power User

### Snippet Variables *(P2)*
- Mark a clip as a template: `Hello {{name}}, your order {{order_id}} is ready`
- On paste, MonoClip opens a tiny fill-in dialog before pasting

### Clip Expiry *(P2)*
- Set a TTL on any clip: "delete this in 1 hour / 1 day"
- Useful for temporary tokens, one-time codes

### Regex / Transform Rules *(P3)*
- Auto-strip UTM params from copied URLs
- Auto-format phone numbers, trim whitespace on paste
- User-defined regex → replacement rules applied transparently

### Keyboard-Only Mode *(P2)*
- Full arrow-key navigation through clips and folders
- `Enter` to copy, `⌫` to delete, `P` to pin, `F` to move to folder — no mouse needed

---

## 🔐 Privacy & Security

### Sensitive Clip Detection *(P2)*
- Auto-detect passwords, API keys, credit card numbers (regex + entropy scoring)
- Warn before saving; option to skip history for detected secrets
- "Incognito mode" toggle: pause history without quitting

### Encrypted Vault *(P3)*
- Optional AES-256 encryption for the database (unlocked with Touch ID / password)
- Encrypted clips shown as `****` until unlocked

### App Exclusions *(P2)*
- Don't capture clips from specific apps (e.g. 1Password, banking apps)
- Managed via a list in Settings

---

## 📱 Platform

### iOS / iPadOS Companion App *(P3)*
- Read-only view of synced clips on iPhone/iPad
- Paste via Share Sheet extension

### Menu Bar Quick-Paste *(P2)*
- Show last 5 clips in the tray menu — paste without opening the full window

### Alfred / Raycast Plugin *(P2)*
- Surface MonoClip items in Alfred/Raycast search
- Create/delete clips from the launcher

---

## 🛠️ Developer

### REST API / Unix Socket *(P3)*
- Local HTTP server on a loopback port (or Unix socket)
- Lets any script or app query/write clips without the CLI binary
- Swagger UI available at `http://localhost:PORT/docs`

### Webhooks *(P3)*
- Fire a webhook on new clip: integrate with Make, Zapier, n8n
- Use case: auto-save copied links to Notion, Pocket, etc.

### Plugin System *(P3)*
- WASM-based plugin sandbox
- Plugins can register new transform actions, sync backends, and clip types

---

## ✅ Already Shipped

- [x] Multi-folder organisation with emoji + colour
- [x] Global shortcut routing (save clipboard/selection → folder)
- [x] Image clipboard capture with thumbnails
- [x] File & folder path capture (full path, not just filename)
- [x] mclip CLI (list, add, remove, pin, get, folder management)
- [x] AI context copy (`mclip context`) + MCP stdio server (`mclip mcp`)
- [x] Help panel with keyboard shortcuts and CLI reference
- [x] Auto-cleanup with configurable retention
- [x] Launch at login
- [x] Paste on click
- [x] Glass-morphism macOS-native UI
