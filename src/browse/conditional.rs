use crate::{youtube, YouTubeError};
use reqwest;
use prost::Message;

#[derive(Debug, Clone)]
pub enum ConditionalRedirectResult {
    Channel(String),
    Blocked,
}

#[derive(Debug, Clone)]
pub struct CountryCodeResult {
    pub country_code: String,
}

pub struct ResolveConditionalRedirectRequest {
    pub proxy_url: String,
    pub channel_id: String,
}

impl ResolveConditionalRedirectRequest {
    pub async fn send(self) -> Result<Option<ConditionalRedirectResult>, YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 1, 
                    client_version: "2.20240614.01.00".to_string() 
                })
            }),
            browse_id: Some(self.channel_id.clone()),
            params: None,
            continuation: None,
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let proxy = reqwest::Proxy::all(&self.proxy_url)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let response = client
            .post("https://youtubei.googleapis.com/youtubei/v1/browse")
            .header("Host", "youtubei.googleapis.com")
            .header("X-Goog-Fieldmask", "alerts,onResponseReceivedActions.navigateAction.endpoint.browseEndpoint")
            .header("Content-Type", "application/x-protobuf")
            .body(payload)
            .send()
            .await
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        match response.status() {
            reqwest::StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            reqwest::StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            reqwest::StatusCode::OK => {},
            status => return Err(YouTubeError::UnknownStatusCode(hyper::StatusCode::from_u16(status.as_u16()).unwrap())),
        }

        let body_bytes = response.bytes().await
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;
        
        let browse_response = youtube::BrowseResponse::decode(body_bytes)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        // Check for blocked channel in alerts
        if let Some(alerts) = &browse_response.alerts {
            if let Some(alert) = &alerts.alert_renderer {
                if let Some(text) = &alert.text {
                    if text.simple_text == "This channel is not available." {
                        return Ok(Some(ConditionalRedirectResult::Blocked));
                    }
                }
            }
        }

        // Check for conditional redirect in onResponseReceivedActions
        if let Some(actions) = &browse_response.on_response_received_actions {
            if let Some(navigate_action) = &actions.navigate_action {
                if let Some(endpoint) = &navigate_action.endpoint {
                    if let Some(browse_endpoint) = &endpoint.browse_endpoint {
                        return Ok(Some(ConditionalRedirectResult::Channel(browse_endpoint.browse_id.clone())));
                    }
                }
            }
        }

        // No redirect or block detected
        Ok(None)
    }
}

pub struct DetectCountryCodeRequest {
    pub proxy_url: String,
}

impl DetectCountryCodeRequest {
    pub async fn send(self) -> Result<CountryCodeResult, YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 1, 
                    client_version: "2.20240614.01.00".to_string() 
                })
            }),
            browse_id: Some("FEwhat_to_watch".to_string()),
            params: None,
            continuation: None,
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let proxy = reqwest::Proxy::all(&self.proxy_url)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let response = client
            .post("https://youtubei.googleapis.com/youtubei/v1/browse")
            .header("Host", "youtubei.googleapis.com")
            .header("X-Goog-Fieldmask", "topbar.desktopTopbarRenderer.countryCode")
            .header("Content-Type", "application/x-protobuf")
            .body(payload)
            .send()
            .await
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        match response.status() {
            reqwest::StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
            reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            reqwest::StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            reqwest::StatusCode::OK => {},
            status => return Err(YouTubeError::UnknownStatusCode(hyper::StatusCode::from_u16(status.as_u16()).unwrap())),
        }

        let body_bytes = response.bytes().await
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;
        
        let browse_response = youtube::BrowseResponse::decode(body_bytes)
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        // Extract country code from topbar
        if let Some(topbar) = &browse_response.topbar {
            if let Some(desktop_topbar_renderer) = &topbar.desktop_topbar_renderer {
                if !desktop_topbar_renderer.country_code.is_empty() {
                    return Ok(CountryCodeResult {
                        country_code: desktop_topbar_renderer.country_code.clone(),
                    });
                } else {
                    return Ok(CountryCodeResult {
                        country_code: "US".to_string(),
                    });
                }
            }
        }

        // If no country code found, return error
        Err(YouTubeError::ParseError("Country code not found in response".to_string()))
    }
}