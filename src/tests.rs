use std::error::Error;
use tokio;
use crate::InnertubeClient;
use scylla::frame::value::CqlTimestamp;
use crate::InnerTubeRequest;

#[tokio::test]
async fn test_get_channel_extended() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let mut channel = innertube.get_channel(channel_id).send().await?;
    innertube.get_channel_extended(&mut channel).send().await?;
    
    assert_eq!(channel.display_name, "MrBeast");
    assert_eq!(channel.handle.unwrap(), "MrBeast");
    assert!(!channel.description.is_empty());
    assert!(!channel.profile_picture.unwrap().is_empty());
    assert!(!channel.banner.unwrap().is_empty());
    assert!(channel.verified);
    assert!(!channel.oac);
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    assert!(channel.subscribers.unwrap() > 0);
    assert!(channel.views > 0);
    assert!(channel.videos > 0);
    assert_eq!(channel.country.unwrap(), "US");
    assert_eq!(channel.created_at, CqlTimestamp(1329609600*1000));
    assert_eq!(channel.has_business_email, true);
    assert!(channel.links.len() > 0);
    assert_eq!(channel.blocked_countries.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_blocked_countries() -> Result<(), Box<dyn Error>> {
    // Channel with blocked countries
    let channel_id = "UC6pA-fA0pM1e_eTbCXzyHNw".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    // Should have blocked countries
    assert!(!channel.blocked_countries.is_empty());
    // Channel should be active/valid
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    
    Ok(())
}

#[tokio::test]
async fn test_channel_with_carousel() -> Result<(), Box<dyn Error>> {
    // Sports channel (official YT channel) (known to have a carousel)
    let channel_id = "UCEgdi0XIXXZ-qJOFPf4JSKw".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(channel.has_carousel, "Channel should have a carousel");
    // Channel should be active/valid
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    
    Ok(())
}

#[tokio::test]
async fn test_channel_without_carousel() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel (known to not have a carousel)
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(!channel.has_carousel, "Channel should not have a carousel");
    // Channel should be active/valid
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    assert!(channel.verified);
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_terminated() -> Result<(), Box<dyn Error>> {
    // LeafyIsHere's terminated channel
    let channel_id = "UCxJf49T4iTO_jtzWX3rW_jg".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(channel.terminated);
    assert!(!channel.termination_reason.is_empty());
    assert!(!channel.deleted);
    assert!(!channel.hidden);
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_hidden() -> Result<(), Box<dyn Error>> {
    // /user/Z's hidden channel
    let channel_id = "UCk3PBU7EtwVhotDzGvwUtAg".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(channel.hidden);
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_deleted() -> Result<(), Box<dyn Error>> {
    // Some deleted channel ID
    let channel_id = "UC0123456789ABCDEFGHIJ".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    assert!(!channel.verified);
    assert!(!channel.oac);
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_music() -> Result<(), Box<dyn Error>> {
    // Ed Sheeran's channel (music artist)
    let channel_id = "UC0C-w0YjGpqDXGB8IHb662A".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(!channel.verified);
    assert!(channel.oac);
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_hidden_subcount() -> Result<(), Box<dyn Error>> {
    // sandroensonymusic's channel (music artist)
    let channel_id = "UC6-GyjvNVs-SGWLeEDR84WA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(!channel.deleted);
    assert!(!channel.terminated);
    assert!(!channel.hidden);
    assert_eq!(channel.subscribers, None);
    
    Ok(())
}

#[tokio::test]
async fn test_get_special_tab() -> Result<(), Box<dyn Error>> {
    // YouTube Music
    let channel_id = "UClgRkhTL3_hImCAmdLfDE4g".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let channel = innertube.get_channel(channel_id).send().await?;
    
    assert!(channel.channel_tabs.contains(&"Browse".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_get_channel_extended_small_channel() -> Result<(), Box<dyn Error>> {
    // A smaller channel to test different number formats
    let channel_id = "UCyj-EUmmEfIlUg-pYVn-vxw".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let mut channel = innertube.get_channel(channel_id).send().await?;
    innertube.get_channel_extended(&mut channel).send().await?;
    
    assert!(!channel.verified);
    assert!(!channel.oac);
    assert!(channel.subscribers.unwrap() >= 0);
    assert!(channel.views >= 0);
    assert!(channel.videos >= 0);
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_extended() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let (videos, continuation) = innertube.get_videos_extended(channel_id, crate::browse::videos::ChannelTab::Videos).send().await?;
    
    // Should get some videos
    assert!(!videos.is_empty());
    
    // MrBeast's channel should have a continuation token as it has many videos
    assert!(continuation.is_some());
    
    // Test format and validity of returned videos
    for video in &videos {
        // Video IDs should be 11 characters
        assert_eq!(video.video_id.len(), 11);
        // Video IDs should only contain valid characters
        assert!(video.video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        // MrBeast doesn't hide view counts
        assert!(!video.hidden_view_count);
        // His videos should have views
        assert!(video.views > 0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_extended_hidden_views() -> Result<(), Box<dyn Error>> {
    // Channel known to have hidden view counts
    let channel_id = "UCpF2RRkCVrZL7CNO5THyjIA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let (videos, continuation) = innertube.get_videos_extended(channel_id, crate::browse::videos::ChannelTab::Videos).send().await?;
    
    // Should get videos back
    assert!(!videos.is_empty());
    
    // Channel should have continuation since it has many videos
    assert!(continuation.is_none());
    
    let mut found_hidden = false;
    let mut found_visible = false;
    
    for video in &videos {
        // Basic format checks
        assert_eq!(video.video_id.len(), 11);
        assert!(video.video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        
        if video.hidden_view_count {
            found_hidden = true;
        } else {
            found_visible = true;
            assert!(video.views >= 0);
        }
    }
    
    // This channel should have both hidden and visible view counts
    assert!(found_hidden, "Should find at least one video with hidden view count");
    assert!(found_visible, "Should find at least one video with visible view count");
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_extended_live_badges() -> Result<(), Box<dyn Error>> {
    // Adele's channel - testing Live tab
    let channel_id = "UCsRM0YB_dabtEPGPTKo-gcw".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let (videos, _) = innertube.get_videos_extended(channel_id, crate::browse::videos::ChannelTab::Live).send().await?;
    
    // Should get some videos back
    assert!(!videos.is_empty());
    
    // At least one video should have a badge
    let has_badge = videos.iter().any(|v| v.badge.is_some());
    assert!(has_badge, "Expected at least one video to have a badge");
    
    // Test format of videos
    for video in &videos {
        // Video IDs should be 11 characters
        assert_eq!(video.video_id.len(), 11);
        // Video IDs should only contain valid characters
        assert!(video.video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_extended_terminated() -> Result<(), Box<dyn Error>> {
    // LeafyIsHere's terminated channel
    let channel_id = "UCxJf49T4iTO_jtzWX3rW_jg".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let (videos, continuation) = innertube.get_videos_extended(channel_id, crate::browse::videos::ChannelTab::Videos).send().await?;
    
    // Terminated channels should return empty video list and no continuation
    assert!(videos.is_empty());
    assert!(continuation.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_get_popular_videos() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel, known for having highly viewed videos
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let videos = innertube.get_popular_videos(channel_id).send().await?;
    
    // Should get a list of popular videos
    assert!(!videos.is_empty());
    
    // Test format and validity of returned videos
    for video in &videos {
        // Video IDs should be 11 characters
        assert_eq!(video.video_id.len(), 11);
        // Video IDs should only contain valid characters
        assert!(video.video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        // MrBeast doesn't hide view counts
        assert!(!video.hidden_view_count);
        // His popular videos should have significant view counts
        assert!(video.views > 1_000_000);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_popular_videos_hidden_views() -> Result<(), Box<dyn Error>> {
    // Channel known to have hidden view counts
    let channel_id = "UCpF2RRkCVrZL7CNO5THyjIA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let videos = innertube.get_popular_videos(channel_id).send().await?;
    
    // Should get videos back
    assert!(!videos.is_empty());
    
    let mut found_hidden = false;
    let mut found_visible = false;
    
    for video in &videos {
        // Basic format checks
        assert_eq!(video.video_id.len(), 11);
        assert!(video.video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        
        if video.hidden_view_count {
            found_hidden = true;
        } else {
            found_visible = true;
            assert!(video.views >= 0);
        }
    }
    
    // This channel should have both hidden and visible view counts
    assert!(found_hidden, "Should find at least one video with hidden view count");
    assert!(found_visible, "Should find at least one video with visible view count");
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let response = innertube.get_videos(channel_id).send().await?;
    
    // Check that we got some videos
    assert!(!response.video_ids.is_empty());
    
    // Video IDs should be valid format (11 characters)
    for video_id in &response.video_ids {
        assert_eq!(video_id.len(), 11);
    }
    
    // A channel with many videos should have a continuation token
    assert!(response.continuation.is_some());
    
    // Test with continuation token
    if let Some(continuation) = response.continuation {
        assert!(!continuation.is_empty());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_small_channel() -> Result<(), Box<dyn Error>> {
    // Using the smaller test channel from the channel tests
    let channel_id = "UCyj-EUmmEfIlUg-pYVn-vxw".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let response = innertube.get_videos(channel_id).send().await?;
    
    // Should still get some videos
    assert!(!response.video_ids.is_empty());
    
    // Video IDs should still be valid format
    for video_id in &response.video_ids {
        assert_eq!(video_id.len(), 11);
    }

    // No continuation token
    assert!(response.continuation.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_terminated_channel() -> Result<(), Box<dyn Error>> {
    // LeafyIsHere's terminated channel
    let channel_id = "UCxJf49T4iTO_jtzWX3rW_jg".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let response = innertube.get_videos(channel_id).send().await?;
    
    // Terminated channels should return empty video list
    assert!(response.video_ids.is_empty());
    assert!(response.continuation.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_continuation() -> Result<(), Box<dyn Error>> {
    // continuation token from loading MrBeast's videos tab continuation
    let continuation_token = "4qmFsgKrCBIYVUNYNk9RM0RrY3NiWU5FNkg4dVFRdVZBGo4IOGdhRUJocUJCbnItQlFyNUJRclFCVUZrY2tnelRGQkhXRkkzZUhFNGNHWTVibDh3TjNBd09WTkxjamd6UjFwcmMweE1haTF2Y1ZKVlFtbzRPSFkxTVdSNGFEaExVWGRTU0dwZmEyWnZNVEJ5WjJSSk0wSklTV2xhVGxGSk56WkxaRGxGWmxaQ1VETnlhR05RYlRaYU4zSlZabWhTUm5vMGJubFlURGRCYlVaQmVWOXlNSEV6WjBoV2IxRmZabkExY0dGNmJtMUtaMEZMUzNSeGQzRlhZVEU0VjBjME5GbENTWEJ2ZEVSdk9YWmpPREYwTkhZMlNVSjBORmg0YVZwZmRURm5hVEI0TWxaMFZVdFdXVlIzTnpkQ01qRlhkbXN3ZDA5eFJsUlBVbU5YVGtObGFtMXlTVGwwVjB3emJHZFJORlpyVUhGT05XaDVOMnRSWkU5T2JUWkpiSEF4VHpGNFlqVnhOWGR3YVVONWRYWlhTVGR0UVZSZlNEVjNPRWRPWXpkZk1IWnVSRzlFYlU5elNtODBSbFpFTUhOUWNrMVRiVkpVUkY5RFNscFhjSGhVY3pGRVYxSjZkVnBQYmxORVIwWmtaeTFEZUdsNVgxbHhabnBRVTI5MmNFOUZMWGx3WTNKMlNpMWFlak5vVjBSSWFURnhNbUZXYVhrM1ZqSjRUbW94YnpoVVNrNVdkRmsxV1RoMFdtdHJWME5aYjA5M1gyWm5XVE51ZEUxZlduUktORE5oUW5KdWQyRmxXbmRsTm5oWVExWm1hMlZ4V0hwSlpsbGljVWhETTFsR1Z6ZG9Na1pXY1Mxb2EyMUJVa0ZFVVdKYVJXbzVTVkJCV2xCcmJHWktjM2RxYm5GQ1QwTlhTRVZzV25sWVp6ZG9hRE5sT0VocGRrbDVWVVpFUVZsdlp6SkhYMUpuYVY5cGFXVlpPRlpOWmtjd1dtaDRTR2hPYlZnd1dUUkdiV0phUWxCVVFVMTRWbTlJVW1Ga1IwcDFibU0yY1ZOMFlqTm1OWEZMT0hkd1pXcDJWWHBCUlRZeE9YRk9iVGN3Y1cxTlluUkJRWEJJU0hOVWNuQlBSR2RGWTFnd09GVmpWa05pY0ZVMVpqWkxZakZtUkVKM01rUkhWRlJmY0c5bFowbzNlWFE0V0ZCbmJHdHdMWE5TYjNNMmJYTjVTa2xEVERGWFpYQTJaMGhVZEhWVFdFRXpZM2xCVFRsaFRWbHZjQzFTUlVJM2FuQTVZbUZCYVRCSVJrOXViRlJaZEhNNVdrRkJabXBaUTNnME9VVmFSemxCWWpOcFZFZHpZaElrTmpkbE5qRmpOamd0TURBd01DMHlZalExTFdFek56RXRZV016WldJeE5XUXpaRGd3SUFRJTNE".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let response = innertube.get_videos_continued(continuation_token).send().await?;
    
    // Verify we got video results
    assert!(!response.video_ids.is_empty());
    
    // Check that video IDs are valid format (11 chars)
    for video_id in &response.video_ids {
        assert_eq!(video_id.len(), 11);
        // Video IDs should only contain valid characters
        assert!(video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
    }
    
    // Since this is MrBeast's channel which has many videos,
    // we expect another continuation token
    assert!(response.continuation.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_get_videos_extended_continuation() -> Result<(), Box<dyn Error>> {
    // MrBeast's continuation token from videos tab
    let continuation_token = "4qmFsgKrCBIYVUNYNk9RM0RrY3NiWU5FNkg4dVFRdVZBGo4IOGdhRUJocUJCbnItQlFyNUJRclFCVUZrY2tnelRGQkhXRkkzZUhFNGNHWTVibDh3TjNBd09WTkxjamd6UjFwcmMweE1haTF2Y1ZKVlFtbzRPSFkxTVdSNGFEaExVWGRTU0dwZmEyWnZNVEJ5WjJSSk0wSklTV2xhVGxGSk56WkxaRGxGWmxaQ1VETnlhR05RYlRaYU4zSlZabWhTUm5vMGJubFlURGRCYlVaQmVWOXlNSEV6WjBoV2IxRmZabkExY0dGNmJtMUtaMEZMUzNSeGQzRlhZVEU0VjBjME5GbENTWEJ2ZEVSdk9YWmpPREYwTkhZMlNVSjBORmg0YVZwZmRURm5hVEI0TWxaMFZVdFdXVlIzTnpkQ01qRlhkbXN3ZDA5eFJsUlBVbU5YVGtObGFtMXlTVGwwVjB3emJHZFJORlpyVUhGT05XaDVOMnRSWkU5T2JUWkpiSEF4VHpGNFlqVnhOWGR3YVVONWRYWlhTVGR0UVZSZlNEVjNPRWRPWXpkZk1IWnVSRzlFYlU5elNtODBSbFpFTUhOUWNrMVRiVkpVUkY5RFNscFhjSGhVY3pGRVYxSjZkVnBQYmxORVIwWmtaeTFEZUdsNVgxbHhabnBRVTI5MmNFOUZMWGx3WTNKMlNpMWFlak5vVjBSSWFURnhNbUZXYVhrM1ZqSjRUbW94YnpoVVNrNVdkRmsxV1RoMFdtdHJWME5aYjA5M1gyWm5XVE51ZEUxZlduUktORE5oUW5KdWQyRmxXbmRsTm5oWVExWm1hMlZ4V0hwSlpsbGljVWhETTFsR1Z6ZG9Na1pXY1Mxb2EyMUJVa0ZFVVdKYVJXbzVTVkJCV2xCcmJHWktjM2RxYm5GQ1QwTlhTRVZzV25sWVp6ZG9hRE5sT0VocGRrbDVWVVpFUVZsdlp6SkhYMUpuYVY5cGFXVlpPRlpOWmtjd1dtaDRTR2hPYlZnd1dUUkdiV0phUWxCVVFVMTRWbTlJVW1Ga1IwcDFibU0yY1ZOMFlqTm1OWEZMT0hkd1pXcDJWWHBCUlRZeE9YRk9iVGN3Y1cxTlluUkJRWEJJU0hOVWNuQlBSR2RGWTFnd09GVmpWa05pY0ZVMVpqWkxZakZtUkVKM01rUkhWRlJmY0c5bFowbzNlWFE0V0ZCbmJHdHdMWE5TYjNNMmJYTjVTa2xEVERGWFpYQTJaMGhVZEhWVFdFRXpZM2xCVFRsaFRWbHZjQzFTUlVJM2FuQTVZbUZCYVRCSVJrOXViRlJaZEhNNVdrRkJabXBaUTNnME9VVmFSemxCWWpOcFZFZHpZaElrTmpkbE5qRmpOamd0TURBd01DMHlZalExTFdFek56RXRZV016WldJeE5XUXpaRGd3SUFRJTNE".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let (videos, continuation) = innertube.get_videos_extended_continued(continuation_token).send().await?;
    
    // Should get videos back
    assert!(!videos.is_empty());
    
    // Test format of videos
    for video in &videos {
        // Video IDs should be 11 characters
        assert_eq!(video.video_id.len(), 11);
        // Video IDs should only contain valid characters
        assert!(video.video_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        // Since this is MrBeast's channel, no hidden view counts
        assert!(!video.hidden_view_count);
        // His videos should have views
        assert!(video.views > 0);
    }
    
    // Since this is MrBeast's channel which has many videos,
    // we expect another continuation token
    assert!(continuation.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_get_watch_next() -> Result<(), Box<dyn Error>> {
    // Use MrBeast's $456,000 Squid Game In Real Life! video
    let video_id = "0e3GPea1Tyg".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let watch_next = innertube.get_watch_next(video_id).send().await?;
    
    // Should get some recommendations
    assert!(!watch_next.is_empty());
    
    // Check first recommendation
    let first = watch_next.first().expect("Should have at least one recommendation");
    
    // The user_id should be a valid length (22 chars for channel IDs)
    assert_eq!(first.user_id.len(), 22);
    
    // The video_id should be valid (11 chars)
    assert_eq!(first.video_id.len(), 11);
    
    // The user_id should not contain the UC prefix
    assert!(!first.user_id.starts_with("UC"));
    
    Ok(())
}

#[tokio::test]
async fn test_get_watch_next_same_channel() -> Result<(), Box<dyn Error>> {
    // Use a video that only recommends from the same channel
    let video_id = "StM9FVdIgig".to_string(); 
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let watch_next = innertube.get_watch_next(video_id).send().await?;
    
    // Should get recommendations
    assert!(!watch_next.is_empty());
    
    // All recommendations should be from the same channel
    let expected_channel = "VJHDqW6It6yVCVsxIYWxcQ";  // Without UC prefix
    
    for recommendation in watch_next {
        assert_eq!(recommendation.user_id, expected_channel);
        assert_eq!(recommendation.video_id.len(), 11);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_watch_no_recommend() -> Result<(), Box<dyn Error>> {
    // Use a video that only recommends from the same channel but has no other videos, so it doesn't have any recommended
    let video_id = "zLZ7f2Nrpvk".to_string(); 
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let watch_next = innertube.get_watch_next(video_id).send().await?;
    
    // Should get recommendations
    assert!(watch_next.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_get_watch_next_age_restricted() -> Result<(), Box<dyn Error>> {
    // Use an age-restricted video
    let video_id = "24XWKNxmdiw".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Should return AgeRestricted error
    let result = innertube.get_watch_next(video_id).send().await;
    assert!(matches!(result, Err(crate::YouTubeError::WatchNextRendererUnavailable)));
    
    Ok(())
}

#[tokio::test]
async fn test_get_watch_next_short_video() -> Result<(), Box<dyn Error>> {
    // Use a YouTube Short video ID
    let video_id = "YlvcFJOE-OE".to_string();  // Some short video
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let watch_next = innertube.get_watch_next(video_id).send().await?;
    
    // Should still get recommendations
    assert!(!watch_next.is_empty());
    
    // Test format of recommendations
    for recommendation in watch_next {
        assert_eq!(recommendation.user_id.len(), 22);
        assert_eq!(recommendation.video_id.len(), 11);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_resolve_url() -> Result<(), Box<dyn Error>> {
    // Test with a channel URL that should resolve to a channel ID
    let url = "https://www.youtube.com/@MrBeast".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let result = innertube.resolve_url(url).send().await?;
    
    // Should resolve to MrBeast's channel ID
    assert!(result.is_some());
    assert_eq!(result.unwrap().browse_endpoint.unwrap(), "UCX6OQ3DkcsbYNE6H8uQQuVA");
    
    Ok(())
}

#[tokio::test]
async fn test_has_public_subscriptions() -> Result<(), Box<dyn Error>> {
    // Jawed's channel (known to have public subscriptions)
    let channel_id = "UC4QobU6STFB0P71PMvOGN5A".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let has_public = innertube.has_public_subscriptions(channel_id).send().await?;
    
    assert!(has_public, "Jawed's channel should have public subscriptions");
    
    Ok(())
}

#[tokio::test]
async fn test_no_public_subscriptions() -> Result<(), Box<dyn Error>> {
    // MrBeast's channel (known to have private subscriptions)
    let channel_id = "UCX6OQ3DkcsbYNE6H8uQQuVA".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let has_public = innertube.has_public_subscriptions(channel_id).send().await?;
    
    assert!(!has_public, "Channel should not have public subscriptions");
    
    Ok(())
}

#[tokio::test]
async fn test_search_public_creator_entities() -> Result<(), Box<dyn Error>> {
    // Search for channels with "MrBeast" in the name
    let query = "MrBeast".to_string();
    
    let mut innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let cookie = "changethis";
    let authorization = "changethis";
    let channel_ids = innertube.search_public_creator_entities(query).with_cookie(cookie).with_authorization(authorization).send().await?;
    
    // Should get some results for a popular search term
    assert!(!channel_ids.is_empty(), "Search should return some channels");
    
    // Test format of returned channel IDs
    for channel_id in &channel_ids {
        // Channel IDs should be 22-24 characters long
        assert!(channel_id.len() >= 22 && channel_id.len() <= 24, 
                "Channel ID should be 22-24 characters, got: {}", channel_id);
        // Channel IDs should start with UC (for channels)
        assert!(channel_id.starts_with("UC"), 
                "Channel ID should start with UC, got: {}", channel_id);
        // Channel IDs should only contain valid characters
        assert!(channel_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "Channel ID contains invalid characters: {}", channel_id);
    }
    
    // Should return at most 10 results based on the expected response format
    assert!(channel_ids.len() <= 10, "Should return at most 10 results");
    
    Ok(())
}

#[tokio::test]
async fn test_resolve_conditional_redirect() -> Result<(), Box<dyn Error>> {
    // Channel known to have conditional redirect
    let channel_id = "UCzucOGEnILK5cAdUUVCIefg".to_string();
    let proxy_url = "socks5://hk-hkg-wg-socks5-201.relays.mullvad.net:1080".to_string(); // Example proxy
    
    use crate::browse::conditional::ConditionalRedirectResult;
    
    let innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let request = innertube.resolve_conditional_redirect(proxy_url, channel_id);
    let result = request.send().await?;
    
    // Should get a conditional redirect result
    assert!(result.is_some(), "Should detect conditional redirect");
    
    match result.unwrap() {
        ConditionalRedirectResult::Channel(redirected_id) => {
            // Should be redirected to a different channel ID
            assert_eq!(redirected_id.len(), 24); // Channel IDs are 24 chars
            assert!(redirected_id.starts_with("UC"));
            assert_ne!(redirected_id, "UCzucOGEnILK5cAdUUVCIefg"); // Should be different from original
        },
        ConditionalRedirectResult::Blocked => {
            panic!("Expected redirect, but got blocked");
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_detect_country_code() -> Result<(), Box<dyn Error>> {
    // Use a proxy to detect country code
    let proxy_url = "socks5://hk-hkg-wg-socks5-201.relays.mullvad.net:1080".to_string(); // Example Hong Kong proxy
    
    let innertube = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    let request = innertube.detect_country_code(proxy_url);
    let result = request.send().await?;
    
    // Should get a country code result
    assert!(!result.country_code.is_empty(), "Country code should not be empty");
    
    // Country codes should be 2 characters (ISO 3166-1 alpha-2)
    assert_eq!(result.country_code.len(), 2, "Country code should be 2 characters");
    
    // Country codes should be uppercase letters
    assert!(result.country_code.chars().all(|c| c.is_ascii_uppercase()), 
            "Country code should contain only uppercase letters: {}", result.country_code);
    
    // For Hong Kong proxy, should likely return HK
    // Note: This might vary depending on proxy availability and routing
    println!("Detected country code: {}", result.country_code);
    
    Ok(())
}
