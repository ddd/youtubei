use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use crate::utils::parse_numeric_string;
use crate::models::Video;
use crate::browse::videos::GetVideosResponse;
use crate::browse::videos::BLACKLISTED_BADGE_LABELS;
use crate::browse::videos::{parse_length_text, parse_published_time_text};

pub struct GetVideosContinuationRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub continuation_token: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetVideosContinuationRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetVideosContinuationRequest<'a> {

    pub async fn send(self) -> Result<GetVideosResponse, YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { client_name: 1, client_version: "2.20240614.01.00".to_string() })
            }),
            browse_id: None,
            params: None,
            continuation: Some(self.continuation_token)
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "onResponseReceivedActions.appendContinuationItemsAction.continuationItems(richItemRenderer.content.videoRenderer.videoId,continuationItemRenderer.continuationEndpoint.continuationCommand.token)")
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
        if let Some(endpoints) = response.on_response_received_actions{
            if let Some(action) = endpoints.append_continuation_items_action {
                for item in action.continuation_items {
                    // Extract video IDs from rich item renderers
                    if let Some(rich_item) = item.rich_item_renderer {
                        if let Some(content) = rich_item.content {
                            if let Some(video_renderer) = content.video_renderer {
                                if !video_renderer.video_id.is_empty() {
                                    video_ids.push(video_renderer.video_id);
                                }
                            }
                        }
                    }

                    // Extract continuation token from continuation item renderer
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

        Ok(GetVideosResponse {
            video_ids,
            continuation
        })
    
    }

}

pub struct GetVideosExtendedContinuationRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub continuation_token: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetVideosExtendedContinuationRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetVideosExtendedContinuationRequest<'a> {
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
            continuation: Some(self.continuation_token),
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "onResponseReceivedActions.appendContinuationItemsAction.continuationItems(richItemRenderer.content.videoRenderer(videoId,viewCountText.simpleText,lengthText.simpleText,publishedTimeText.simpleText,badges),continuationItemRenderer.continuationEndpoint.continuationCommand.token)")
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

        // Parse the response to extract videos and continuation token
        if let Some(actions) = response.on_response_received_actions {
            if let Some(action) = actions.append_continuation_items_action {
                for item in action.continuation_items {
                    // Extract video information from rich item renderers
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

                                    // Check if badges array exists and has at least one item
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