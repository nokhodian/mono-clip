package tiktok

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/go-rod/rod"
	"github.com/go-rod/rod/lib/proto"
)

// ListUserVideos navigates to a TikTok profile page, scrolls to load the video
// grid, and returns up to maxCount video entries.
func (b *TikTokBot) ListUserVideos(ctx context.Context, page *rod.Page, profileURL string, maxCount int) (interface{}, error) {
	if profileURL == "" {
		return nil, fmt.Errorf("tiktok: profileURL is required")
	}
	if maxCount <= 0 {
		maxCount = 20
	}

	if err := page.Navigate(profileURL); err != nil {
		return nil, fmt.Errorf("tiktok: navigate to %s: %w", profileURL, err)
	}
	if err := page.WaitLoad(); err != nil {
		return nil, fmt.Errorf("tiktok: page load failed: %w", err)
	}
	time.Sleep(3 * time.Second)

	// Click videos tab to ensure the grid is active.
	if tab, err := page.Timeout(5 * time.Second).Element("[data-e2e='videos-tab']"); err == nil && tab != nil {
		_ = tab.Click(proto.InputMouseButtonLeft, 1)
		time.Sleep(2 * time.Second)
	}

	// Scroll to load video grid items.
	var videos []map[string]interface{}
	prevCount := 0
	noChangeRounds := 0

	for len(videos) < maxCount && noChangeRounds < 3 {
		result, err := page.Eval(`() => {
			const items = document.querySelectorAll('[data-e2e="user-post-item"]');
			return JSON.stringify(Array.from(items).map(el => {
				const a = el.querySelector('a');
				const img = el.querySelector('img');
				return {
					url: a ? a.href : '',
					thumbnail: img ? img.src : '',
				};
			}).filter(v => v.url !== ''));
		}`)
		if err == nil && result != nil {
			var parsed []map[string]interface{}
			if jsonErr := json.Unmarshal([]byte(result.Value.Str()), &parsed); jsonErr == nil {
				videos = parsed
			}
		}

		if len(videos) == prevCount {
			noChangeRounds++
		} else {
			noChangeRounds = 0
			prevCount = len(videos)
		}

		if len(videos) < maxCount {
			_, _ = page.Eval("() => window.scrollBy(0, 800)")
			time.Sleep(1500 * time.Millisecond)
		}
	}

	if len(videos) > maxCount {
		videos = videos[:maxCount]
	}

	out := make([]interface{}, len(videos))
	for i, v := range videos {
		out[i] = v
	}
	return out, nil
}

// FollowUser navigates to a TikTok profile page and clicks the Follow button.
// Returns nil if already following (idempotent).
func (b *TikTokBot) FollowUser(ctx context.Context, page *rod.Page, profileURL string) error {
	if profileURL == "" {
		return fmt.Errorf("tiktok: profileURL is required")
	}

	if err := page.Navigate(profileURL); err != nil {
		return fmt.Errorf("tiktok: navigate to %s: %w", profileURL, err)
	}
	if err := page.WaitLoad(); err != nil {
		return fmt.Errorf("tiktok: page load failed: %w", err)
	}
	time.Sleep(3 * time.Second)

	btn, err := page.Timeout(5 * time.Second).Element("[data-e2e='follow-button']")
	if err != nil {
		return fmt.Errorf("tiktok: follow button not found on %s", profileURL)
	}

	text, _ := btn.Text()
	text = strings.TrimSpace(strings.ToLower(text))
	if text == "following" || text == "friends" {
		return nil // already following — idempotent
	}

	if err := btn.Click(proto.InputMouseButtonLeft, 1); err != nil {
		return fmt.Errorf("tiktok: failed to click follow button: %w", err)
	}
	time.Sleep(1 * time.Second)
	return nil
}

func (b *TikTokBot) LikeVideo(ctx context.Context, page *rod.Page, videoURL string) error {
	return fmt.Errorf("tiktok: LikeVideo not yet implemented")
}

func (b *TikTokBot) CommentOnVideo(ctx context.Context, page *rod.Page, videoURL string, commentText string) error {
	return fmt.Errorf("tiktok: CommentOnVideo not yet implemented")
}

func (b *TikTokBot) ListVideoComments(ctx context.Context, page *rod.Page, videoURL string, maxCount int) (interface{}, error) {
	return nil, fmt.Errorf("tiktok: ListVideoComments not yet implemented")
}

func (b *TikTokBot) LikeComment(ctx context.Context, page *rod.Page, videoURL string, commentID string) error {
	return fmt.Errorf("tiktok: LikeComment not yet implemented")
}

func (b *TikTokBot) StitchVideo(ctx context.Context, page *rod.Page, videoURL string) error {
	return fmt.Errorf("tiktok: StitchVideo not yet implemented")
}

func (b *TikTokBot) DuetVideo(ctx context.Context, page *rod.Page, videoURL string) error {
	return fmt.Errorf("tiktok: DuetVideo not yet implemented")
}

func (b *TikTokBot) ShareVideo(ctx context.Context, page *rod.Page, videoURL string) (interface{}, error) {
	return nil, fmt.Errorf("tiktok: ShareVideo not yet implemented")
}
