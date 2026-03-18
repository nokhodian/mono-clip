package tiktok

import (
	"context"
	"fmt"

	"github.com/go-rod/rod"
)

func (b *TikTokBot) ListUserVideos(ctx context.Context, page *rod.Page, profileURL string, maxCount int) (interface{}, error) {
	return nil, fmt.Errorf("tiktok: ListUserVideos not yet implemented")
}

func (b *TikTokBot) FollowUser(ctx context.Context, page *rod.Page, profileURL string) error {
	return fmt.Errorf("tiktok: FollowUser not yet implemented")
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
