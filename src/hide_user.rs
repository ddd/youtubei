use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct UpdateHideUserStatusRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
    pub hide_user: bool,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for UpdateHideUserStatusRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> UpdateHideUserStatusRequest<'a> {
    pub async fn send(self) -> Result<(), YouTubeError> {
        let user_id = if self.channel_id.len() >= 2 {
            self.channel_id[2..].to_string()
        } else {
            // Handle edge case where channel_id is too short
            return Err(YouTubeError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Channel ID must be at least 2 characters long"
            ))));
        };

        // Create the HideUserAction with the modified user_id
        let hide_user_action = youtube::HideUserAction {
            user_id,
            options: if self.hide_user {
                youtube::HideUserOptions::HideUser as i32
            } else {
                youtube::HideUserOptions::UnhideUser as i32
            },
        };
        // Encode the action to protobuf bytes
        let mut action_bytes = Vec::new();
        hide_user_action.encode(&mut action_bytes)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        // Base64 encode the protobuf bytes
        let action_b64 = BASE64.encode(action_bytes);

        // Create the flag request
        let request = youtube::FlagRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client {
                    client_name: 1,
                    client_version: "2.20240614.01.00".to_string()
                })
            }),
            action: action_b64,
        };

        let mut payload = Vec::new();
        request.encode(&mut payload)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let mut req_builder = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/flag/flag", self.ip))
            .header("Host", "www.youtube.com")
            .header("Cookie", self.fields.cookie.unwrap())
            .header("Authorization", self.fields.authorization.unwrap())
            .header("Origin", "https://www.youtube.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Encode-Response-If-Executable", "base64");

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

        match resp.status() {
            StatusCode::NOT_FOUND => Err(YouTubeError::NotFound),
            StatusCode::TOO_MANY_REQUESTS => Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => Err(YouTubeError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => Err(YouTubeError::InternalServerError),
            StatusCode::OK => Ok(()),
            status => {
                let body_bytes = resp.into_body().collect().await?.to_bytes();
                let body_str = String::from_utf8_lossy(&body_bytes);
                eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
                Err(YouTubeError::UnknownStatusCode(status))
            }
        }
    }
}