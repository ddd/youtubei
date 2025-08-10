use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;

#[derive(Debug)]
pub struct ResolveUrlResult {
    pub browse_endpoint: Option<String>,
    pub url_endpoint: Option<String>,
}

pub struct ResolveUrlRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub url: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for ResolveUrlRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> ResolveUrlRequest<'a> {

    pub async fn send(self) -> Result<Option<ResolveUrlResult>, YouTubeError> {
        let request = youtube::ResolveUrlRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { client_name: 1, client_version: "2.20240614.01.00".to_string() })
            }),
            url: self.url
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/navigation/resolve_url", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "endpoint.browseEndpoint.browseId,endpoint.urlEndpoint.url")
            .body(Full::new(Bytes::from(payload)))
            .map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::NOT_FOUND => return Ok(None), // Return None for 404
            StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
            StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
            StatusCode::OK => (), // Continue processing
            status => return Err(YouTubeError::UnknownStatusCode(status)),
        };

        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let response = youtube::ResolveUrlResponse::decode(body_bytes)?;

        let endpoint = match response.endpoint {
            Some(ep) => ep,
            None => return Ok(None),
        };

        let browse_endpoint = endpoint.browse_endpoint
            .and_then(|browse_endpoint| {
                if browse_endpoint.browse_id.is_empty() {
                    None
                } else {
                    Some(browse_endpoint.browse_id)
                }
            });

        let url_endpoint = endpoint.url_endpoint
            .and_then(|url_endpoint| {
                if url_endpoint.url.is_empty() {
                    None
                } else {
                    Some(url_endpoint.url)
                }
            });

        if browse_endpoint.is_none() && url_endpoint.is_none() {
            Ok(None)
        } else {
            Ok(Some(ResolveUrlResult {
                browse_endpoint,
                url_endpoint,
            }))
        }
    
    }

}