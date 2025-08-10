use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use crate::models::WatchNext;

pub struct GetWatchNextRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub video_id: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetWatchNextRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetWatchNextRequest<'a> {
    pub async fn send(self) -> Result<Vec<WatchNext>, YouTubeError> {
        let request = youtube::NextRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 1, 
                    client_version: "2.20240614.01.00".to_string() 
                })
            }),
            video_id: self.video_id.clone(),
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/next", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "contents.twoColumnWatchNextResults(secondaryResults.secondaryResults.results(compactVideoRenderer(videoId,shortBylineText.runs(navigationEndpoint.browseEndpoint.browseId))))")
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
        let response = youtube::NextResponse::decode(body_bytes)?;

        let mut watch_next = Vec::new();

        // Check for the complete path to results, return WatchNextRendererUnavailable if any part is missing
        let results = response.contents
            .and_then(|contents| contents.two_column_watch_next_results)
            .and_then(|watch_next_results| watch_next_results.secondary_results)
            .and_then(|secondary_results| secondary_results.secondary_results)
            .map(|secondary_results| secondary_results.results)
            .ok_or(YouTubeError::WatchNextRendererUnavailable)?;

        // Now we know results exists, process them
        for result in results {
            if let Some(video) = result.compact_video_renderer {
                // Extract channel ID from short byline text
                if let Some(short_byline) = video.short_byline_text {
                    if let Some(runs) = short_byline.runs {
                        if let Some(navigation) = runs.navigation_endpoint {
                            if let Some(browse) = navigation.browse_endpoint {
                                // Strip "UC" prefix from browse_id to get user_id
                                if let Some(user_id) = browse.browse_id.strip_prefix("UC") {
                                    watch_next.push(WatchNext {
                                        user_id: user_id.to_string(),
                                        video_id: video.video_id.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(watch_next)
    }
}