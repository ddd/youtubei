use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use serde::{Deserialize, Serialize};

// JSON response structures for search_public_creator_entities
#[derive(Deserialize, Debug)]
struct SearchCreatorEntitiesResponse {
    channels: Vec<SearchCreatorChannel>,
}

#[derive(Deserialize, Debug)]
struct SearchCreatorChannel {
    #[serde(rename = "channelId")]
    channel_id: String,
}

pub struct SearchPublicCreatorEntitiesRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub query: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for SearchPublicCreatorEntitiesRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> SearchPublicCreatorEntitiesRequest<'a> {
    pub async fn send(self) -> Result<Vec<String>, YouTubeError> {
        let request = youtube::SearchPublicCreatorEntitiesRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 62, 
                    client_version: "1.20250527.06.00".to_string() 
                })
            }),
            query: self.query,
            filter: Some(crate::youtube::search_public_creator_entities_request::Filter{
                restrict_result_type: 10
            }),
            channel_mask: None,
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let mut req_builder = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/creator/search_public_creator_entities?alt=json", self.ip))
            .header("Host", "studio.youtube.com")
            .header("Content-Type", "application/x-protobuf")
            .header("Origin", "https://studio.youtube.com")
            .header("X-Goog-Fieldmask", "channels(channelId)");

        // Add bearer token if provided
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
            StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => return Err(YouTubeError::InternalServerError),
            StatusCode::OK => (), // Continue processing
            _ => {
                // For unknown status codes, collect and print the response for debugging
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = std::str::from_utf8(&body_bytes).unwrap_or("<invalid utf8>");
                println!("Unknown status code {} with response: {}", status, body_str);
                return Err(YouTubeError::UnknownStatusCode(status));
            }
        };

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        
        // Parse as JSON instead of protobuf
        let body_str = std::str::from_utf8(&body_bytes)
            .map_err(|e| YouTubeError::ParseError(format!("Invalid UTF-8: {}", e)))?;

        // Handle empty response case where API returns just "{}"
        if body_str.trim() == "{}" {
            return Ok(Vec::new());
        }
            
        let response: SearchCreatorEntitiesResponse = serde_json::from_str(body_str)
            .map_err(|e| YouTubeError::ParseError(format!("JSON parse error: {}", e)))?;

        // Extract channel IDs from response
        let channel_ids: Vec<String> = response.channels
            .into_iter()
            .map(|channel| channel.channel_id[2..].to_string())
            .collect();

        Ok(channel_ids)
    }
}