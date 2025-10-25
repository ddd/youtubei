use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use crate::models::Video;
use crate::utils::{generate_continuation_token, parse_numeric_string};
use std::collections::HashSet;
use once_cell::sync::Lazy;
use std::time::{SystemTime, UNIX_EPOCH};

pub static BLACKLISTED_BADGE_LABELS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "360°", "VR180", "Fundraiser"
     ])
});

pub fn parse_length_text(length_text: &str) -> Option<i32> {
    let parts: Vec<&str> = length_text.split(':').collect();
    
    match parts.len() {
        2 => {
            // Format: MM:SS
            let minutes = parts[0].parse::<i32>().ok()?;
            let seconds = parts[1].parse::<i32>().ok()?;
            Some(minutes * 60 + seconds)
        }
        3 => {
            // Format: HH:MM:SS
            let hours = parts[0].parse::<i32>().ok()?;
            let minutes = parts[1].parse::<i32>().ok()?;
            let seconds = parts[2].parse::<i32>().ok()?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        _ => None
    }
}

pub fn parse_published_time_text(published_text: &str) -> Option<i64> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    
    let text = published_text.to_lowercase();
    
    if text.contains("second") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(seconds) = num_str.parse::<i64>() {
                return Some(current_time - seconds);
            }
        }
    } else if text.contains("minute") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(minutes) = num_str.parse::<i64>() {
                return Some(current_time - (minutes * 60));
            }
        }
    } else if text.contains("hour") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(hours) = num_str.parse::<i64>() {
                return Some(current_time - (hours * 3600));
            }
        }
    } else if text.contains("day") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(days) = num_str.parse::<i64>() {
                return Some(current_time - (days * 86400));
            }
        }
    } else if text.contains("week") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(weeks) = num_str.parse::<i64>() {
                return Some(current_time - (weeks * 604800));
            }
        }
    } else if text.contains("month") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(months) = num_str.parse::<i64>() {
                return Some(current_time - (months * 2629746)); // ~30.44 days per month
            }
        }
    } else if text.contains("year") {
        if let Some(num_str) = text.split_whitespace().next() {
            if let Ok(years) = num_str.parse::<i64>() {
                return Some(current_time - (years * 31556952)); // ~365.25 days per year
            }
        }
    }
    
    None
}

pub struct GetVideosRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetVideosRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

pub struct GetVideosResponse {
    pub video_ids: Vec<String>,
    pub continuation: Option<String>
}

impl<'a> GetVideosRequest<'a> {

    pub async fn send(self) -> Result<GetVideosResponse, YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { client_name: 1, client_version: "2.20240614.01.00".to_string() })
            }),
            browse_id: Some(self.channel_id),
            params: Some("EgZ2aWRlb3PyBgQKAjoA".to_string()),
            continuation: None
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "contents.twoColumnBrowseResultsRenderer.tabs.tabRenderer.content.richGridRenderer.contents(richItemRenderer.content.videoRenderer.videoId,continuationItemRenderer.continuationEndpoint.continuationCommand.token)")
            .body(Full::new(Bytes::from(payload)))
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::NOT_FOUND => Err(YouTubeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => Err(YouTubeError::Unauthorized),
            StatusCode::OK => Ok(()),
            status => Err(YouTubeError::UnknownStatusCode(status)),
        }?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let response = youtube::BrowseResponse::decode(body_bytes)?;

        // Parse about videos data
        let mut video_ids = Vec::new();
        let mut continuation = None;

        // Parse the response to extract video IDs and continuation token
        if let Some(contents) = response.contents {
            if let Some(two_col) = contents.two_column_browse_results_renderer {
                for tab in two_col.tabs {
                    if let Some(tab_renderer) = tab.tab_renderer {
                        if let Some(content) = tab_renderer.content {
                            if let Some(grid) = content.rich_grid_renderer {
                                for grid_content in grid.contents {
                                    // Extract video ID if present
                                    if let Some(rich_item) = grid_content.rich_item_renderer {
                                        if let Some(content) = rich_item.content {
                                            if let Some(video) = content.video_renderer {
                                                if !video.video_id.is_empty() {
                                                    video_ids.push(video.video_id);
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Extract continuation token if present
                                    if let Some(cont_item) = grid_content.continuation_item_renderer {
                                        if let Some(endpoint) = cont_item.continuation_endpoint {
                                            if let Some(command) = endpoint.continuation_command {
                                                if !command.token.is_empty() {
                                                    continuation = Some(command.token);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(GetVideosResponse {
            video_ids,
            continuation
        })
    
    }

}

#[derive(Debug, Clone)]
pub enum ChannelTab {
    Videos,
    Live,
}

impl ChannelTab {
    fn as_str(&self) -> &'static str {
        match self {
            ChannelTab::Videos => "EgZ2aWRlb3PyBgQKAjoA",
            ChannelTab::Live => "EgdzdHJlYW1z8gYECgJ6AA==",
        }
    }
}

pub struct GetVideosExtendedRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
    pub tab: ChannelTab
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetVideosExtendedRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetVideosExtendedRequest<'a> {

    pub async fn send(self) -> Result<(Vec<Video>, Option<String>), YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 1, 
                    client_version: "2.20240614.01.00".to_string() 
                })
            }),
            browse_id: Some(self.channel_id),
            params: Some(self.tab.as_str().to_string()),
            continuation: None
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "contents.twoColumnBrowseResultsRenderer.tabs.tabRenderer.content.richGridRenderer.contents(richItemRenderer.content.videoRenderer(videoId,viewCountText.simpleText,lengthText.simpleText,publishedTimeText.simpleText,badges,upcomingEventData),continuationItemRenderer.continuationEndpoint.continuationCommand.token)")
            .body(Full::new(Bytes::from(payload)))
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::NOT_FOUND => Err(YouTubeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => Err(YouTubeError::Unauthorized),
            StatusCode::OK => Ok(()),
            status => Err(YouTubeError::UnknownStatusCode(status)),
        }?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let response = youtube::BrowseResponse::decode(body_bytes)?;

        let mut videos = Vec::new();
        let mut continuation = None;

        // Parse the response to extract videos with view counts and continuation token
        if let Some(contents) = response.contents {
            if let Some(two_col) = contents.two_column_browse_results_renderer {
                for tab in two_col.tabs {
                    if let Some(tab_renderer) = tab.tab_renderer {
                        if let Some(content) = tab_renderer.content {
                            if let Some(grid) = content.rich_grid_renderer {
                                for grid_content in grid.contents {
                                    if let Some(rich_item) = grid_content.rich_item_renderer {
                                        if let Some(content) = rich_item.content {
                                            if let Some(video) = content.video_renderer {
                                                if video.upcoming_event_data.is_none() && !video.video_id.is_empty() {
                                                    let (views, hidden_view_count) = if let Some(view_count) = video.view_count_text {
                                                        let view_text = view_count.simple_text.trim();
                                                        if view_text == "No views" {
                                                            (0, false)
                                                        } else {
                                                            (parse_numeric_string(
                                                                view_text.trim_end_matches(" views")
                                                            ), false)
                                                        }
                                                    } else {
                                                        (0, true)
                                                    };

                                                    // Get the first non-blacklisted badge label
                                                    let badge = video.badges.iter()
                                                        .find_map(|badge| {
                                                            if let Some(renderer) = &badge.metadata_badge_renderer {
                                                                if !BLACKLISTED_BADGE_LABELS.contains(renderer.label.as_str()) {
                                                                    Some(renderer.label.clone())
                                                                } else {
                                                                    None
                                                                }
                                                            } else {
                                                                None
                                                            }
                                                        });

                                                    let length_seconds = video.length_text
                                                        .as_ref()
                                                        .and_then(|lt| parse_length_text(&lt.simple_text));

                                                    let approx_published_time = video.published_time_text
                                                        .as_ref()
                                                        .and_then(|pt| parse_published_time_text(&pt.simple_text));

                                                    videos.push(Video {
                                                        video_id: video.video_id,
                                                        views,
                                                        hidden_view_count,
                                                        badge,
                                                        length_seconds,
                                                        approx_published_time
                                                    });
                                                }
                                            }
                                        }
                                    }

                                    // Extract continuation token if present
                                    if let Some(cont_item) = grid_content.continuation_item_renderer {
                                        if let Some(endpoint) = cont_item.continuation_endpoint {
                                            if let Some(command) = endpoint.continuation_command {
                                                if !command.token.is_empty() {
                                                    continuation = Some(command.token);
                                                }
                                            }
                                        }
                                    }
                                }   
                            }
                        }
                    }
                }
            }
        }

        Ok((videos, continuation))
    }
}

pub struct GetPopularVideosRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetPopularVideosRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetPopularVideosRequest<'a> {
    pub async fn send(self) -> Result<(Vec<Video>, Option<String>), YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 1, 
                    client_version: "2.20240614.01.00".to_string() 
                })
            }),
            browse_id: None,
            params: None,
            continuation: Some(generate_continuation_token(
                self.channel_id,
                "8gYuGix6KhImCiQ2N2Y1N2IwZi0wMDAwLTJjNjctODA4OC0zYzI4NmQzZTJkYjIgAg%3D%3D".to_string()
            )),
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "onResponseReceivedActions.reloadContinuationItemsCommand.continuationItems(richItemRenderer.content.videoRenderer(videoId,viewCountText.simpleText,lengthText.simpleText,publishedTimeText.simpleText,badges),continuationItemRenderer.continuationEndpoint.continuationCommand.token)")
            .body(Full::new(Bytes::from(payload)))
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::NOT_FOUND => Err(YouTubeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => Err(YouTubeError::Unauthorized),
            StatusCode::OK => Ok(()),
            status => Err(YouTubeError::UnknownStatusCode(status)),
        }?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let response = youtube::BrowseResponse::decode(body_bytes)?;

        let mut videos = Vec::new();
        let mut continuation = None;

        // Parse the response to extract videos with view counts and continuation token
        if let Some(actions) = response.on_response_received_actions {
            if let Some(command) = actions.reload_continuation_items_command {
                for item in command.continuation_ttems {
                    if let Some(rich_item) = item.rich_item_renderer {
                        if let Some(content) = rich_item.content {
                            if let Some(video) = content.video_renderer {
                                if video.upcoming_event_data.is_none() && !video.video_id.is_empty() {
                                    let (views, hidden_view_count) = if let Some(view_count) = video.view_count_text {
                                        let view_text = view_count.simple_text.trim();
                                        if view_text == "No views" {
                                            (0, false)
                                        } else {
                                            (parse_numeric_string(
                                                view_text.trim_end_matches(" views")
                                            ), false)
                                        }
                                    } else {
                                        (0, true)
                                    };

                                    // Get the first non-blacklisted badge label
                                    let badge = video.badges.iter()
                                        .find_map(|badge| {
                                            if let Some(renderer) = &badge.metadata_badge_renderer {
                                                if !BLACKLISTED_BADGE_LABELS.contains(renderer.label.as_str()) {
                                                    Some(renderer.label.clone())
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        });

                                    let length_seconds = video.length_text
                                        .as_ref()
                                        .and_then(|lt| parse_length_text(&lt.simple_text));

                                    let approx_published_time = video.published_time_text
                                        .as_ref()
                                        .and_then(|pt| parse_published_time_text(&pt.simple_text));

                                    videos.push(Video {
                                        video_id: video.video_id,
                                        views,
                                        hidden_view_count,
                                        badge,
                                        length_seconds,
                                        approx_published_time
                                    });
                                }
                            }
                        }
                    }
                    
                    // Extract continuation token if present
                    if let Some(cont_item) = item.continuation_item_renderer {
                        if let Some(endpoint) = cont_item.continuation_endpoint {
                            if let Some(command) = endpoint.continuation_command {
                                if !command.token.is_empty() {
                                    continuation = Some(command.token);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((videos, continuation))
    }
}