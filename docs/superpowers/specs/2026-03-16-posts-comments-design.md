# Posts & Comments Feature Design

**Date:** 2026-03-16
**Project:** monoes-agent
**Status:** Approved

---

## Overview

Add first-class support for scraped social media posts and their comments. Posts are displayed as a collapsible section on a person's Profile page. Clicking a post opens a full PostDetail page showing post metadata and a scrollable comments list. The CLI auto-saves post and comment data to SQLite after running `list_user_posts` and `list_post_comments` node types.

---

## Data Layer

### Schema

Two new tables, created at app startup via `CREATE TABLE IF NOT EXISTS`:

```sql
CREATE TABLE IF NOT EXISTS posts (
  id            TEXT PRIMARY KEY,
  person_id     TEXT REFERENCES people(id),
  platform      TEXT NOT NULL,
  shortcode     TEXT NOT NULL,
  url           TEXT NOT NULL,
  thumbnail_url TEXT,
  like_count    INTEGER,
  comment_count INTEGER,
  caption       TEXT,
  posted_at     TEXT,
  scraped_at    TEXT NOT NULL,
  UNIQUE(platform, shortcode)
);

CREATE TABLE IF NOT EXISTS post_comments (
  id          TEXT PRIMARY KEY,
  post_id     TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
  author      TEXT NOT NULL,
  text        TEXT,
  timestamp   TEXT,
  likes_count INTEGER DEFAULT 0,
  reply_count INTEGER DEFAULT 0,
  scraped_at  TEXT NOT NULL
);
```

### CLI Auto-Save Wiring

Same pattern as `scrape_profile_info → people`:

- After `node run instagram.list_user_posts` completes, results are upserted into `posts`. Conflict on `(platform, shortcode)` updates `like_count`, `comment_count`, `thumbnail_url`, `caption`, `scraped_at`.
- After `node run instagram.list_post_comments` completes, results are inserted into `post_comments`. Keyed by `post_id + author + timestamp` to avoid duplicates.
- `person_id` is resolved at save time by looking up `people` by platform + username extracted from the post URL using the existing `bot.PlatformRegistry`.

The auto-save happens in `cmd/monoes/node.go` after `executor.Execute()`, parallel to the existing `scrape_profile_info → people` save block.

---

## Backend API

Three new Wails-bound functions on `App` in `wails-app/app.go`:

```go
func (a *App) GetPersonPosts(personId string) []PostSummary
func (a *App) GetPostDetail(postId string) *PostDetail
func (a *App) GetPostComments(postId string) []PostComment
```

### Response Types

```go
type PostSummary struct {
    ID           string `json:"id"`
    Shortcode    string `json:"shortcode"`
    URL          string `json:"url"`
    ThumbnailURL string `json:"thumbnail_url"`
    LikeCount    int    `json:"like_count"`
    CommentCount int    `json:"comment_count"`
    Caption      string `json:"caption"`
    PostedAt     string `json:"posted_at"`
    ScrapedAt    string `json:"scraped_at"`
    WeLiked      bool   `json:"we_liked"`
    WeCommented  bool   `json:"we_commented"`
}

type PostDetail struct {
    PostSummary
    // same fields — alias for future extension
}

type PostComment struct {
    ID         string `json:"id"`
    Author     string `json:"author"`
    Text       string `json:"text"`
    Timestamp  string `json:"timestamp"`
    LikesCount int    `json:"likes_count"`
    ReplyCount int    `json:"reply_count"`
}
```

### we_liked / we_commented Derivation

`GetPersonPosts` uses a LEFT JOIN on `action_targets` where `link = posts.url` and `action_type` is `like_posts` or `comment_on_posts` with `status = 'COMPLETED'`. Results are aggregated per post into boolean flags.

---

## Frontend

### Navigation (App.jsx)

Add `postId` state and `postDetail` page entry, parallel to the existing `profileId` / `profile` pattern:

- **People** → click person → **Profile** (sets `profileId`)
- **Profile** → click post → **PostDetail** (sets `postId`)
- **PostDetail** → Back → **Profile** (clears `postId`)

### Profile.jsx — Posts Section

A collapsible section inserted **above** the interaction history:

- Header: `Posts · {count}` with a chevron toggle
- Collapsed by default when no posts exist; expanded by default when posts are present
- Each row in the list shows:
  - Shortcode in monospace as a clickable link (navigates to PostDetail)
  - External link icon (opens post URL in browser)
  - Like count with heart icon
  - Comment count with speech bubble icon
  - `♥ liked` badge (cyan) — only when `we_liked = true`
  - `💬 commented` badge (purple) — only when `we_commented = true`
- Empty state: *"No posts scraped yet — run list_user_posts to populate"*

### PostDetail.jsx — New Page

Full-page post detail view:

**Header:**
- Back button → returns to Profile
- Post shortcode as page title
- External link icon to open raw post URL

**Meta row:**
- Platform badge
- Like count
- Comment count
- Scraped date

**Comments section:**
- Label: `Comments ({n})`
- Scrollable list; each row:
  - `@author` in cyan monospace
  - Comment text
  - Timestamp (right-aligned, muted)
  - Like count with heart icon (only shown if > 0)
- Empty state: *"No comments scraped yet — run list_post_comments to populate"*

### api.js Additions

```javascript
getPersonPosts:   (personId) => GoApp.GetPersonPosts(personId).catch(() => [])
getPostDetail:    (postId)   => GoApp.GetPostDetail(postId).catch(() => null)
getPostComments:  (postId)   => GoApp.GetPostComments(postId).catch(() => [])
```

---

## Testing

Use the CLI to verify the full pipeline:

```bash
# 1. Scrape 10 posts for mortezanoes
go run ./cmd/monoes node run instagram.list_user_posts \
  --config '{"username":"onetap","targets":[{"url":"https://www.instagram.com/mortezanoes/","username":"mortezanoes"}],"maxCount":10}'

# 2. Verify posts saved
go run ./cmd/monoes node run people.posts --config '{"personId":"<id>"}'
# or query SQLite directly:
sqlite3 ~/.monoes/monoes.db "SELECT shortcode, like_count, comment_count FROM posts WHERE platform='INSTAGRAM' LIMIT 10;"

# 3. Scrape 10 comments from one of the posts
go run ./cmd/monoes node run instagram.list_post_comments \
  --config '{"username":"onetap","targets":[{"url":"https://www.instagram.com/p/<shortcode>/"}],"maxComments":10}'

# 4. Verify comments saved
sqlite3 ~/.monoes/monoes.db "SELECT author, text, likes_count FROM post_comments LIMIT 10;"
```

---

## Error Handling

- Missing `person_id` at save time (person not yet in `people` table): post is saved with `person_id = NULL`; it will still appear if queried by shortcode but won't show on a profile until re-scraped after the person is added.
- Duplicate comments: upsert on `(post_id, author, timestamp)` — later scrape wins on `text`, `likes_count`, `reply_count`.
- `GetPersonPosts` / `GetPostComments` return empty slice (never null) on DB error to keep the frontend safe.
