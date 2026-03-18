# TikTok Action Suite — Design Spec

**Date:** 2026-03-18
**Status:** Approved

---

## Overview

Implement a full TikTok bot adapter following the same three-layer pattern as LinkedIn and Instagram:

1. **Go bot adapter** — `TikTokBot` with `GetMethodByName` satisfying `action.BotAdapter`
2. **JSON action definitions** — one file per action in `data/actions/tiktok/`
3. **Registration** — wired into `cmd/monoes/node.go` CLI bot registry

**9 actions total:** 6 Instagram-equivalent (renamed to TikTok terms) + 3 TikTok-exclusive.

---

## Actions

| Method name | Type | Description |
|---|---|---|
| `list_user_videos` | list | Collect video URLs from a profile's video grid |
| `like_video` | interaction | Like a video by URL |
| `comment_on_video` | interaction | Post a comment on a video |
| `list_video_comments` | list | List comments on a video |
| `like_comment` | interaction | Like a specific comment on a video |
| `follow_user` | interaction | Follow a TikTok user by profile URL |
| `stitch_video` | tiktok-exclusive | Open stitch creation flow for a video |
| `duet_video` | tiktok-exclusive | Open duet creation flow for a video |
| `share_video` | tiktok-exclusive | Copy video share link to clipboard, return URL |

---

## Architecture

### Go Files

```
internal/bot/tiktok/
  bot.go       — TikTokBot struct, New(), Register(), GetMethodByName() dispatch
  actions.go   — one exported function per action (JS DOM automation via go-rod)
```

`TikTokBot` must implement both:
- `bot.BotAdapter` (Platform, LoginURL, IsLoggedIn, etc.) — for login flow
- `action.BotAdapter` (GetMethodByName) — for `call_bot_method` step dispatch

### JSON Action Files

```
data/actions/tiktok/
  list_user_videos.json
  like_video.json
  comment_on_video.json
  list_video_comments.json
  like_comment.json
  follow_user.json
  stitch_video.json
  duet_video.json
  share_video.json
```

### Registration

In `cmd/monoes/node.go`, register `tiktok` in `cliBotRegistry` alongside `linkedin` and `instagram`.

---

## DOM Selectors

TikTok uses stable `data-e2e` attributes confirmed via live DOM inspection.

### Profile Page (`https://www.tiktok.com/@username`)

```
[data-e2e="user-title"]          → username
[data-e2e="user-subtitle"]       → display name
[data-e2e="user-bio"]            → bio text
[data-e2e="followers-count"]     → follower count
[data-e2e="following-count"]     → following count
[data-e2e="likes-count"]         → total likes
[data-e2e="follow-button"]       → follow/unfollow button
[data-e2e="user-post-item"]      → video thumbnail card in grid
[data-e2e="user-post-item"] a    → anchor with video URL (href contains /video/)
```

### Video Page (`https://www.tiktok.com/@username/video/ID`)

```
[data-e2e="like-icon"]           → like button
[data-e2e="like-count"]          → like count display
[data-e2e="comment-icon"]        → open comments panel
[data-e2e="share-icon"]          → open share modal
[data-e2e="comment-input"]       → comment text input field
[data-e2e="comment-send-btn"]    → submit comment button
[data-e2e="comment-item"]        → individual comment row
[data-e2e="comment-like-btn"]    → like button on a comment
[data-e2e="comment-username"]    → commenter's username text
[data-e2e="comment-content"]     → comment text content
```

### Share Modal

```
[data-e2e="share-stitch"]        → Stitch option
[data-e2e="share-duet"]          → Duet option
[data-e2e="copy-link-icon"]      → Copy link button
```

---

## Data Flow

### List Actions

**`list_user_videos`**
```
loop over selectedListItems (profile URLs):
  navigate to profile URL
  scroll to load video grid
  call_bot_method "list_user_videos" with maxCount
  save_data → results[profileUrl]
  update_progress
  log
```

**`list_video_comments`**
```
loop over selectedListItems (video URLs):
  navigate to video URL
  call_bot_method "list_video_comments" with maxCount
  save_data → results[videoUrl]
  update_progress
  log
```

### Interaction Actions

**`like_video`, `comment_on_video`, `like_comment`, `follow_user`**
```
loop over selectedListItems (video URLs or profile URLs):
  call_bot_method with URL + optional args (comment text, comment ID)
  update_progress
  log
```

### TikTok-Exclusive Actions

**`stitch_video`, `duet_video`**
```
loop over selectedListItems (video URLs):
  navigate to video URL
  call_bot_method → click share icon → click stitch/duet option
  returns true (opens TikTok creator — actual recording out of scope)
  update_progress
  log
```

**`share_video`**
```
loop over selectedListItems (video URLs):
  navigate to video URL
  call_bot_method → click share icon → click copy link
  returns copied URL string
  save_data
  update_progress
  log
```

---

## Go Method Signatures

```go
func (b *TikTokBot) ListUserVideos(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: profileURL string, args[1]: maxCount int
// returns []map[string]interface{} with fields: url, thumbnail, description, likes, comments, shares

func (b *TikTokBot) LikeVideo(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string
// returns bool (true = liked, false = already liked)

func (b *TikTokBot) CommentOnVideo(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string, args[1]: commentText string
// returns bool

func (b *TikTokBot) ListVideoComments(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string, args[1]: maxCount int
// returns []map[string]interface{} with fields: id, username, text, likes, timestamp

func (b *TikTokBot) LikeComment(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string, args[1]: commentID string
// returns bool

func (b *TikTokBot) FollowUser(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: profileURL string
// returns bool (true = followed, false = already following)

func (b *TikTokBot) StitchVideo(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string
// returns bool (true = stitch creator opened)

func (b *TikTokBot) DuetVideo(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string
// returns bool (true = duet creator opened)

func (b *TikTokBot) ShareVideo(ctx context.Context, args ...interface{}) (interface{}, error)
// args[0]: videoURL string
// returns string (copied share URL)
```

---

## Error Handling

- Navigate timeout → return error with URL context
- Element not found → return descriptive error (e.g., "like button not found on page")
- Already liked / already following → return `false` (not an error, idempotent)
- Login required → check `IsLoggedIn()` before each method, return error if not logged in
- Rate limiting (TikTok 429/redirect) → return error, let caller decide retry

---

## Implementation Notes

- Use `{{item.url or item}}` pattern in JSON loops (handles both string URLs and `{url:"..."}` objects)
- `list_user_videos` must scroll the profile page to trigger lazy-loading of the video grid before collecting URLs
- For `list_video_comments`, click `[data-e2e="comment-icon"]` first to open the comments panel, then scroll to load more
- `stitch_video` and `duet_video` are best-effort — they open the creator but can't complete the video recording
- All JS DOM interactions use `page.Eval()` with the marker pattern (`data-monoes-*`) consistent with existing bots

---

## File Summary

**New files:**
- `internal/bot/tiktok/bot.go`
- `internal/bot/tiktok/actions.go`
- `data/actions/tiktok/list_user_videos.json`
- `data/actions/tiktok/like_video.json`
- `data/actions/tiktok/comment_on_video.json`
- `data/actions/tiktok/list_video_comments.json`
- `data/actions/tiktok/like_comment.json`
- `data/actions/tiktok/follow_user.json`
- `data/actions/tiktok/stitch_video.json`
- `data/actions/tiktok/duet_video.json`
- `data/actions/tiktok/share_video.json`

**Modified files:**
- `cmd/monoes/node.go` — register tiktok bot in `cliBotRegistry`
