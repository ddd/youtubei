use crate::models::*;
use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use scylla::frame::value::CqlTimestamp;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct HiddenUser {
    pub display_name: String,
    pub channel_id: String,
    pub avatar_url: Option<String>,
}

pub struct GetCreatorChannelsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_ids: Vec<String>,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetCreatorChannelsRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetCreatorChannelsRequest<'a> {

    fn convert_timestamp(seconds: i64, nanos: i64) -> i64 {
        seconds * 1_000_000_000 + nanos
    }

    pub async fn send(self) -> Result<Vec<Channel>, YouTubeError> {
        let request = youtube::GetCreatorChannelsRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { client_name: 1, client_version: "2.20240614.01.00".to_string() })
            }),
            channel_ids: self.channel_ids,
            mask: Some(youtube::CreatorChannelMask {
                channel_id: true,
                title: true,
                thumbnail_details: Some(youtube::creator_channel_mask::ThumbnailDetailsMask{
                    all: true
                }),
                metric: Some(youtube::creator_channel_mask::MetricsMask{
                    all: true
                }),
                time_created_seconds: true,
                is_name_verified: true,
                channel_handle: true,
                comments_settings: None
            }),
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/creator/get_creator_channels", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("Authorization", self.fields.authorization.unwrap())
            .header("X-Goog-Fieldmask", "channels(channelId,title,thumbnailDetails.thumbnails.url,metric,timeCreatedSeconds,contentOwnerAssociation,isNameVerified,channelHandle)")
            .body(Full::new(Bytes::from(payload)))
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        let status = resp.status();
        
        match status {
            StatusCode::NOT_FOUND => Err(YouTubeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unauthorized error response: {}", body_str);
                return Err(YouTubeError::Unauthorized);
            },
            StatusCode::INTERNAL_SERVER_ERROR => {
                return Err(YouTubeError::InternalServerError);
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                return Err(YouTubeError::InternalServerError);
            }
            StatusCode::OK => Ok(()),
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            },
        }?;

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let decoded_bytes = BASE64.decode(body_bytes)
        .map_err(|e| YouTubeError::Other(Box::new(e)))?;
        let response = youtube::GetCreatorChannelsResponse::decode(Bytes::from(decoded_bytes))?;

        let mut channels = Vec::new();

        // Parse each channel in the response
        for channel_data in response.channels {
            let mut channel = Channel {
                user_id: String::new(),
                handle: None,
                display_name: String::new(),
                description: String::new(),
                profile_picture: None,
                banner: None,
                verified: false,
                oac: false,
                monetized: None,
                subscribers: None,
                views: 0,
                videos: 0,
                created_at: CqlTimestamp(0),
                country: None,
                has_business_email: false,
                links: Vec::new(),
                tags: Vec::new(),
                deleted: false,
                hidden: false,
                terminated: false,
                termination_reason: String::new(),
                no_index: false,
                unlisted: false,
                family_safe: false,
                blocked_countries: Vec::new(),
                channel_tabs: Vec::new(),
                has_carousel: false,
                cms_association: None,
            };

            channel.user_id = channel_data.channel_id;
            channel.display_name = channel_data.title;
            channel.handle = Some(channel_data.channel_handle.strip_prefix('@').unwrap_or(&channel_data.channel_handle).to_string());
            channel.verified = channel_data.is_name_verified;
            
            // Get thumbnail URL if available
            if let Some(thumbnail_details) = channel_data.thumbnail_details {
                if let Some(first_thumbnail) = thumbnail_details.thumbnails.first() {
                    let avatar_url = first_thumbnail.url.clone();
                    // Check if it's a default profile picture
                    if !avatar_url.starts_with("https://yt") {
                        // Remove the base URL prefix by finding the first occurrence of googleusercontent.com/
                        if let Some(index) = avatar_url.find(".ggpht.com/") {
                            let stripped_url = &avatar_url[(index + ".ggpht.com/".len())..];
                            // Remove everything after and including '='
                            let clean_url = stripped_url.split('=').next().unwrap_or(stripped_url);
                            channel.profile_picture = Some(clean_url.to_string());
                        }
                    } else {
                        channel.profile_picture = None;
                    }
                }
            }

            // Get metrics if available
            if let Some(metric) = channel_data.metric {
                channel.subscribers = Some(metric.subscriber_count);
                channel.views = metric.total_video_view_count;
                channel.videos = metric.video_count as i32;
            }

            // Convert creation timestamp
            channel.created_at = CqlTimestamp(channel_data.time_created_seconds * 1000); // Convert seconds to milliseconds

            channels.push(channel);
        }

        Ok(channels)
    }

}

pub struct GetHiddenUsersRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetHiddenUsersRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetHiddenUsersRequest<'a> {
    pub async fn send(self) -> Result<Vec<HiddenUser>, YouTubeError> {
        // Create JSON payload instead of protobuf
        let json_payload = json!({
            "context": {
                "client": {
                    "clientName": 62,
                    "clientVersion": "1.20250731.01.00"
                }
            },
            "channelIds": [self.channel_id],
            "mask": {
                "commentsSettings": {
                    "hiddenUsers": {
                        "all": true
                    }
                }
            }
        });

        let payload = serde_json::to_string(&json_payload)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let mut req_builder = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/creator/get_creator_channels?alt=json", self.ip))
            .header("Host", "studio.youtube.com")
            .header("Origin", "https://studio.youtube.com")
            .header("Content-Type", "application/json")
            .header("X-Goog-Fieldmask", "channels.commentsSettings.hiddenUsers");

        // Add authorization header if provided
        if let Some(token) = self.fields.authorization {
            req_builder = req_builder.header("Authorization", token);
        }

        // Add cookie if provided
        if let Some(cookie) = self.fields.cookie {
            req_builder = req_builder.header("Cookie", cookie);
        }

        let req = req_builder
            .body(Full::new(Bytes::from(payload)))
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        let status = resp.status();
        match status {
            StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unauthorized error response: {}", body_str);
                return Err(YouTubeError::Unauthorized);
            },
            StatusCode::INTERNAL_SERVER_ERROR => return Err(YouTubeError::InternalServerError),
            StatusCode::SERVICE_UNAVAILABLE => return Err(YouTubeError::InternalServerError),
            StatusCode::OK => (), // Continue processing
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                tracing::error!("Unknown status code {}: {}", status.as_u16(), body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            }
        };

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        let json_response: Value = serde_json::from_str(&body_str)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let mut hidden_users = Vec::new();

        // Parse hidden users from the JSON response
        if let Some(channels) = json_response["channels"].as_array() {
            if let Some(channel) = channels.first() {
                if let Some(hidden_users_array) = channel["commentsSettings"]["hiddenUsers"].as_array() {
                    for hidden_user_data in hidden_users_array {
                        let display_name = hidden_user_data["displayName"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        
                        let channel_id = hidden_user_data["externalChannelId"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        
                        let mut avatar_url = None;
                        
                        // Extract avatar URL if available
                        if let Some(thumbnails) = hidden_user_data["avatarThumbnail"]["thumbnails"].as_array() {
                            if let Some(first_thumbnail) = thumbnails.first() {
                                if let Some(url) = first_thumbnail["url"].as_str() {
                                    // Process avatar URL similar to profile picture processing
                                    if !url.starts_with("https://yt") {
                                        if let Some(index) = url.find(".ggpht.com/") {
                                            let stripped_url = &url[(index + ".ggpht.com/".len())..];
                                            let clean_url = stripped_url.split('=').next().unwrap_or(stripped_url);
                                            avatar_url = Some(clean_url.to_string());
                                        }
                                    }
                                }
                            }
                        }

                        let hidden_user = HiddenUser {
                            display_name,
                            channel_id,
                            avatar_url,
                        };

                        hidden_users.push(hidden_user);
                    }
                }
            }
        }

        Ok(hidden_users)
    }
}