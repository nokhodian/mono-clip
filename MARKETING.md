# MonoClip — Marketing Strategy

---

## 🎯 Target Audience

### Primary
- **Developers & engineers** — already live in the terminal, will love the CLI + MCP angle
- **Power users** — people who've outgrown the default clipboard and want control
- **AI-workflow users** — people using Claude, Cursor, Windsurf daily who want their AI to access clipboard history

### Secondary
- **Designers & content creators** — copy references, hex codes, copy snippets constantly
- **Writers & researchers** — gather quotes, links, notes across many sources
- **Students** — research and note-taking across apps

---

## 📣 Distribution Channels

### Tier 1 — High impact, do these first

| Channel | Action | Expected reach |
|---|---|---|
| **Product Hunt** | Full launch with screenshots, demo video, tagline | 500–5,000 installs on launch day if featured |
| **Hacker News** | "Show HN: MonoClip — clipboard manager with a CLI and MCP server" | High engagement with dev audience |
| **r/macapps** | Post with screenshots + short feature summary | Dedicated macOS app community |
| **MacMenuBar.com** | Submit app for listing | Evergreen discovery for menu bar apps |
| **AlternativeTo** | Add as alternative to Paste, Clipy, Flycut, Pasty | Captures people actively searching for alternatives |

### Tier 2 — Developer communities

| Channel | Action |
|---|---|
| **r/rust** | Post about the Tauri + Rust architecture decisions |
| **r/commandline** | Post about `mclip` CLI + MCP integration angle |
| **r/ClaudeAI / r/ChatGPT** | Post about giving AI assistants clipboard access via MCP |
| **dev.to** | Technical writeup (see Content section below) |
| **Hashnode** | Mirror of dev.to post |
| **X / Twitter** | Short demo video thread, tag @tauri_apps @sveltejs |
| **Mastodon (fosstodon.org)** | Open source friendly audience |

### Tier 3 — Directories & aggregators

| Channel | Action |
|---|---|
| **Homebrew Cask (official)** | PR to `homebrew/homebrew-cask` — massive organic reach |
| **Awesome macOS** (GitHub) | PR to iCHAIT/awesome-macOS under Productivity |
| **Awesome Tauri** (GitHub) | PR to tauri-apps/awesome-tauri |
| **Setapp** | Pitch to editorial team — if accepted, recurring revenue per active user |
| **tldr pages** | Submit `mclip` man page once CLI has enough users |

---

## 📝 Content Strategy

### Blog posts to write (ordered by ROI)

1. **"I built a clipboard manager in Rust + Tauri because nothing else did what I wanted"**
   - Personal story angle, honest about the journey
   - Include architecture decisions, what was hard, what Tauri gets right
   - Post to dev.to, HN, personal blog

2. **"Give your AI assistant access to your clipboard history with MCP"**
   - Very timely — MCP is hot right now
   - Show how to wire up `mclip mcp` with Claude Desktop and Cursor
   - Post to dev.to, r/ClaudeAI, r/cursor

3. **"Why I distribute a macOS app outside the App Store (and what I learned)"**
   - Honest breakdown of MAS restrictions vs direct distribution
   - Homebrew tap, Sparkle updates, code signing
   - Interesting to the indie dev community

4. **"Svelte 5 runes in a real app — what actually changed"**
   - Technical deep-dive for the Svelte audience
   - Post to dev.to, r/sveltejs

### Demo video (60–90 seconds)
The single highest-ROI asset. Should show:
1. Copy a few things (text, image, file)
2. Open with `⌘⇧V` — everything is there
3. Search to find something old instantly
4. Folder shortcut capturing selected text
5. Terminal: `mclip list`, `mclip get 3 | pbcopy`

Post to: X/Twitter, YouTube (short), Product Hunt gallery, README.

---

## 🚀 Launch Plan

### Pre-launch (1–2 weeks before)
- [ ] Record demo video
- [ ] Take polished screenshots (all feature areas)
- [ ] Write Product Hunt tagline + description
- [ ] Submit to MacMenuBar.com (takes a few days to approve)
- [ ] Add to AlternativeTo
- [ ] PR to `homebrew-cask` official repo
- [ ] Write the "I built a clipboard manager" dev.to post (don't publish yet)
- [ ] Set up a simple landing page (GitHub Pages is fine)

### Launch day
- [ ] Post on Product Hunt — aim for Tuesday–Thursday morning PST
- [ ] Publish dev.to post, link from HN "Show HN"
- [ ] Post to r/macapps, r/rust, r/commandline
- [ ] Post demo video thread on X/Twitter
- [ ] Share in relevant Slack/Discord communities (Tauri Discord, Svelte Discord)

### Post-launch (ongoing)
- [ ] Respond to every comment and review personally — builds community trust
- [ ] Write follow-up posts based on questions people ask
- [ ] Post changelog updates for each release
- [ ] Keep Homebrew tap updated — users who update regularly stay engaged

---

## 💰 Monetisation Options (when ready)

### Freemium model (recommended)
Keep the core app free forever. Add a **Pro tier** for power features:

| Free | Pro (~$5 one-time or $2/mo) |
|---|---|
| Unlimited clips | ✅ |
| Folders + shortcuts | ✅ |
| CLI + MCP server | ✅ |
| Image + file capture | ✅ |
| Multi-device sync (Google Drive / iCloud) | Pro |
| Encrypted vault (Touch ID) | Pro |
| AI transforms (summarise, translate, format) | Pro |
| Snippet variables with fill-in dialogs | Pro |
| Unlimited clip history (free = 1,000 clips) | Pro |

### Platforms to sell through
- **Gumroad** — simplest, 10% cut, good for one-time purchases
- **Paddle / Lemon Squeezy** — handles VAT globally, better for subscriptions
- **Direct** — Stripe + custom license check, most control, most work

### Revenue projection (rough)
If Product Hunt launch gets 1,000 installs and 5% convert to Pro at $5:
→ $250 one-time. Small, but grows with each post and directory listing.
The real bet is on sync + AI features driving conversion.

---

## 📊 Metrics to Track

| Metric | Tool | Goal |
|---|---|---|
| GitHub stars | GitHub | 500 in first month |
| Homebrew installs | `brew tap` analytics | 200 in first month |
| Product Hunt upvotes | Product Hunt | Top 5 of the day |
| Website visitors | Plausible / Fathom (privacy-friendly) | — |
| Discord / community members | Discord server | 100 in first month |

---

## 🏷️ Positioning & Messaging

### One-liner
> "A blazing-fast clipboard manager for macOS that remembers everything — with a full CLI and AI integration built in."

### vs competitors

| App | Their angle | MonoClip edge |
|---|---|---|
| **Paste** | Beautiful, iCloud sync | Free, CLI, MCP, open source |
| **Clipy** | Free, open source | Better UI, image support, CLI |
| **Flycut** | Minimal, developer-focused | More powerful, AI-ready |
| **Alfred Clipboard** | Part of Alfred ecosystem | Standalone, terminal-native |
| **Raycast Clipboard** | Part of Raycast ecosystem | Standalone, no subscription |

### Key messages by audience

**Developers:** "Your clipboard, from the terminal. `mclip list`, `mclip get 3 | pbcopy`. And your AI can use it too."

**Power users:** "Everything you've ever copied, organised exactly how you want, retrieved in under a second."

**AI users:** "One config line and Claude Desktop can read and write your clipboard history directly."

---

## 🤝 Community Building

- Open a **Discord server** — create channels for `#feedback`, `#showcase`, `#cli-tips`
- Add a **GitHub Discussions** board for feature requests
- Respond to every issue and PR — early contributors become long-term advocates
- Tag releases with detailed changelogs — users who follow releases stay loyal
- Credit contributors in release notes
