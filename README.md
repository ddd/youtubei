
# youtubei

A Rust wrapper for YouTube's InnerTube API that provides access to channel information, videos, and other YouTube data without requiring an API key.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
youtubei = "0.1.2"
```

## Quick Start

```rust
use youtubei::InnertubeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Get basic channel information
    let channel = client.get_channel("UCX6OQ3DkcsbYNE6H8uQQuVA".to_string()).send().await?;
    println!("Channel: {}", channel.display_name);
    
    Ok(())
}
```

## Examples

### Getting Channel Information

```rust
use youtubei::{InnertubeClient, InnerTubeRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Get basic channel info
    let mut channel = client.get_channel("UCX6OQ3DkcsbYNE6H8uQQuVA".to_string()).send().await?;
    
    // Get extended channel information (requires additional API call)
    client.get_channel_extended(&mut channel).send().await?;
    
    println!("Channel: {}", channel.display_name);
    println!("Subscribers: {:?}", channel.subscribers);
    println!("Verified: {}", channel.verified);
    println!("Description: {}", channel.description);
    
    Ok(())
}
```

### Fetching Channel Videos

```rust
use youtubei::{InnertubeClient, browse::videos::ChannelTab};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Get videos from the main Videos tab
    let (videos, continuation) = client
        .get_videos_extended("UCX6OQ3DkcsbYNE6H8uQQuVA".to_string(), ChannelTab::Videos)
        .send()
        .await?;
    
    for video in videos.iter().take(5) {
        println!("Video: {} - {} views", video.video_id, video.views);
    }
    
    // Use continuation to get more videos if available
    if let Some(token) = continuation {
        let (more_videos, _) = client.get_videos_extended_continued(token).send().await?;
        println!("Got {} more videos", more_videos.len());
    }
    
    Ok(())
}
```

### Getting Popular Videos

```rust
use youtubei::InnertubeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Get most popular videos from a channel
    let videos = client
        .get_popular_videos("UCX6OQ3DkcsbYNE6H8uQQuVA".to_string())
        .send()
        .await?;
    
    println!("Popular videos:");
    for video in videos.iter().take(3) {
        println!("- {} ({} views)", video.video_id, video.views);
    }
    
    Ok(())
}
```

### Video Recommendations

```rust
use youtubei::InnertubeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Get watch next recommendations for a video
    let recommendations = client
        .get_watch_next("0e3GPea1Tyg".to_string())
        .send()
        .await?;
    
    println!("Recommended videos:");
    for rec in recommendations.iter().take(5) {
        println!("- Video: {} from channel: {}", rec.video_id, rec.user_id);
    }
    
    Ok(())
}
```

### URL Resolution

```rust
use youtubei::InnertubeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Resolve a YouTube handle or URL to channel ID
    let result = client
        .resolve_url("https://www.youtube.com/@MrBeast".to_string())
        .send()
        .await?;
    
    if let Some(resolved) = result {
        if let Some(channel_id) = resolved.browse_endpoint {
            println!("Channel ID: {}", channel_id);
        }
    }
    
    Ok(())
}
```

### Using Authentication (for Creator API features)

```rust
use youtubei::{InnertubeClient, InnerTubeRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    // Search public creator entities (requires authentication)
    let channels = client
        .search_public_creator_entities("MrBeast".to_string())
        .with_cookie("your_cookie_here")
        .with_authorization("your_auth_token_here")
        .send()
        .await?;
    
    println!("Found {} channels", channels.len());
    
    Ok(())
}
```

### IPv6 Subnet Support

```rust
use youtubei::InnertubeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client with IPv6 subnet for IP rotation
    let mut client = InnertubeClient::new(
        Some("2001:db8::/32"), 
        "youtubei.googleapis.com".to_string(), 
        Some(1234)
    ).await;
    
    // Rotate to a new IP within the subnet
    client.rotate_ipv6();
    
    let channel = client.get_channel("UCX6OQ3DkcsbYNE6H8uQQuVA".to_string()).send().await?;
    println!("Channel: {}", channel.display_name);
    
    Ok(())
}
```

## Error Handling

The library provides comprehensive error handling:

```rust
use youtubei::{InnertubeClient, YouTubeError};

#[tokio::main]
async fn main() {
    let mut client = InnertubeClient::new(None, "youtubei.googleapis.com".to_string(), None).await;
    
    match client.get_channel("invalid_id".to_string()).send().await {
        Ok(channel) => println!("Channel: {}", channel.display_name),
        Err(YouTubeError::NotFound) => println!("Channel not found"),
        Err(YouTubeError::Ratelimited) => println!("Rate limited"),
        Err(YouTubeError::Unauthorized) => println!("Unauthorized access"),
        Err(e) => println!("Other error: {}", e),
    }
}
```