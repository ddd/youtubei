pub mod utils;

use std::error::Error;
use browse::videos_continued::GetVideosExtendedContinuationRequest;
use hyper::body::Bytes;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::Full;
use hyper::StatusCode;
use rand::Rng;
use resolve_url::ResolveUrlRequest;
use thiserror::Error;
use native_tls::TlsConnector;
use hyper_util::rt::TokioExecutor;

#[cfg(test)]
mod tests;

mod youtube {
    include!(concat!(env!("OUT_DIR"), "/youtube.innertube.rs"));
}

pub mod browse;
pub mod creator;
pub mod next;
use crate::browse::channel::GetChannelRequest;
use crate::browse::conditional::{ResolveConditionalRedirectRequest, DetectCountryCodeRequest};
use crate::browse::channel_extended::GetChannelExtendedRequest;
use crate::browse::videos::GetVideosRequest;
use crate::browse::videos::GetVideosExtendedRequest;
use crate::browse::videos::GetPopularVideosRequest;
use crate::browse::videos_continued::GetVideosContinuationRequest;
use crate::browse::subscriptions::HasPublicSubscriptionsRequest;
use crate::creator::channels::{GetCreatorChannelsRequest, GetHiddenUsersRequest};
use crate::creator::search::SearchPublicCreatorEntitiesRequest;
use crate::next::watch_next::GetWatchNextRequest;
use crate::hide_user::UpdateHideUserStatusRequest;
pub mod models;
pub mod resolve_url;
pub mod hide_user;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(#[from] native_tls::Error),
    #[error("Invalid IP: {0}")]
    InvalidIp(String)
}

pub fn initialize_client(subnet: Option<&str>, subnet_id: Option<u16>) -> Result<Client<HttpsConnector<HttpConnector>, Full<Bytes>>, ClientError> {
    let mut http = HttpConnector::new();
    http.enforce_http(false);
    
    if let Some(subnet) = subnet {
        let random_ip = crate::utils::get_rand_ipv6(subnet, subnet_id.unwrap_or_default()).map_err(|e| ClientError::InvalidIp(e.to_string()))?;
        http.set_local_address(Some(random_ip));
    }
    
    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Create an HTTPS connector using the HTTP connector and the custom TLS connector
    let https = HttpsConnector::from((http, tls.into()));
    
    // Create the client with the custom service
    let client = Client::builder(TokioExecutor::new())
        .build::<_, Full<Bytes>>(https);

    Ok(client)
}

pub struct InnerTubeRequestFields<'a> {
    pub authorization: Option<&'a str>,
    pub cookie: Option<&'a str>,
}

pub trait InnerTubeRequest<'a> {
    fn authorization(&mut self) -> &mut Option<&'a str>;

    fn cookie(&mut self) -> &mut Option<&'a str>;

    fn with_authorization(mut self, authorization: &'a str) -> Self
    where
        Self: Sized,
    {
        *self.authorization() = Some(authorization);
        self
    }

    fn with_cookie(mut self, cookie: &'a str) -> Self
    where
        Self: Sized,
    {
        *self.cookie() = Some(cookie);
        self
    }
}

impl<'a, T> InnerTubeRequest<'a> for T
where
    T: AsMut<InnerTubeRequestFields<'a>>,
{
    fn authorization(&mut self) -> &mut Option<&'a str> {
        &mut self.as_mut().authorization
    }

    fn cookie(&mut self) -> &mut Option<&'a str> {
        &mut self.as_mut().cookie
    }
}

#[derive(Error, Debug)]
pub enum YouTubeError {
    #[error("Not found")]
    NotFound,
    #[error("Ratelimited")]
    Ratelimited,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Unknown Status Code")]
    UnknownStatusCode(StatusCode),
    #[error("Watch next renderer")]
    WatchNextRendererUnavailable,
    #[error("Parse error")]
    ParseError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] hyper::Error),
    #[error("Legacy HTTP error: {0}")]
    LegacyHttpError(#[from] hyper_util::client::legacy::Error),
    #[error("Protobuf error: {0}")]
    ProtobufError(#[from] prost::DecodeError),
    #[error("Other error: {0}")]
    Other(Box<dyn Error + Send + Sync>),
}

pub struct InnertubeClient {
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    subnet: String,
    ip: String
}

impl InnertubeClient {
    pub async fn new(subnet: Option<&str>, ip: String, subnet_id: Option<u16>) -> Self {
        let client: Client<HttpsConnector<HttpConnector>, Full<Bytes>> = initialize_client(subnet, subnet_id).unwrap();
        InnertubeClient {
            client,
            subnet: subnet.unwrap_or_default().to_string(),
            ip
        }
    }

    pub async fn from_client(client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>, ip: String, subnet: Option<&str>) -> Self {
        InnertubeClient {
            client,
            subnet: subnet.unwrap_or_default().to_string(),
            ip
        }
    }

    pub fn rotate_ipv6(&mut self) {
        let random_u16: u16 = rand::thread_rng().gen();
        self.client = initialize_client(Some(&self.subnet), Some(random_u16)).unwrap();
    }

    pub fn get_channel(&mut self, channel_id: String) -> GetChannelRequest {
        GetChannelRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            channel_id,
        }
    }

    pub fn get_channel_extended<'client, 'channel>(
        &'client mut self,
        channel: &'channel mut models::Channel
    ) -> GetChannelExtendedRequest<'client> 
    where
        'channel: 'client
    {
        GetChannelExtendedRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            channel,
        }
    }

    pub fn get_videos(&mut self, channel_id: String) -> GetVideosRequest {
        GetVideosRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            channel_id,
        }
    }

    pub fn get_videos_extended(&mut self, channel_id: String, tab: browse::videos::ChannelTab) -> GetVideosExtendedRequest {
        GetVideosExtendedRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            channel_id,
            tab,
        }
    }

    pub fn get_popular_videos(&mut self, channel_id: String) -> GetPopularVideosRequest {
        GetPopularVideosRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            channel_id,
        }
    }

    pub fn get_videos_continued(&mut self, continuation_token: String) -> GetVideosContinuationRequest {
        GetVideosContinuationRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            continuation_token,
        }
    }

    pub fn get_videos_extended_continued(&mut self, continuation_token: String) -> GetVideosExtendedContinuationRequest {
        GetVideosExtendedContinuationRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            continuation_token,
        }
    }

    pub fn resolve_url(&mut self, url: String) -> ResolveUrlRequest {
        ResolveUrlRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            url,
        }
    }

    pub fn get_watch_next(&mut self, video_id: String) -> GetWatchNextRequest {
        GetWatchNextRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            video_id,
        }
    }

    // Add inside impl InnertubeClient:
    pub fn has_public_subscriptions(&mut self, channel_id: String) -> HasPublicSubscriptionsRequest {
        HasPublicSubscriptionsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            channel_id,
        }
    }

    // Authentication required
    pub fn get_creator_channels(&mut self, channel_ids: Vec<String>) -> GetCreatorChannelsRequest {
        GetCreatorChannelsRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields{
                authorization: None,
                cookie: None
            },
            channel_ids,
        }
    }

    // Add this method to the impl InnertubeClient block
    pub fn search_public_creator_entities(&mut self, query: String) -> SearchPublicCreatorEntitiesRequest {
        SearchPublicCreatorEntitiesRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            query,
        }
    }

    pub fn update_hide_user_status(&mut self, channel_id: String, hide_user: bool) -> UpdateHideUserStatusRequest {
        UpdateHideUserStatusRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            channel_id,
            hide_user,
        }
    }

    pub fn get_hidden_users(&mut self, channel_id: String) -> GetHiddenUsersRequest {
        GetHiddenUsersRequest {
            client: &mut self.client,
            ip: &self.ip,
            fields: InnerTubeRequestFields {
                authorization: None,
                cookie: None
            },
            channel_id,
        }
    }

    pub fn resolve_conditional_redirect(&self, proxy_url: String, channel_id: String) -> ResolveConditionalRedirectRequest {
        ResolveConditionalRedirectRequest {
            proxy_url,
            channel_id,
        }
    }

    pub fn detect_country_code(&self, proxy_url: String) -> DetectCountryCodeRequest {
        DetectCountryCodeRequest {
            proxy_url,
        }
    }

}