use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;

pub struct HasPublicSubscriptionsRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for HasPublicSubscriptionsRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> HasPublicSubscriptionsRequest<'a> {
    pub async fn send(self) -> Result<bool, YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { 
                    client_name: 7, // TVHTML5
                    client_version: "7.20250126.17.00".to_string()
                })
            }),
            browse_id: Some(self.channel_id),
            params: None,
            continuation: None,
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "contents.tvBrowseRenderer.content.tvSurfaceContentRenderer.content.sectionListRenderer.contents.shelfRenderer.headerRenderer.shelfHeaderRenderer.avatarLockup.avatarLockupRenderer.title.simpleText")
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

        if let Some(contents) = response.contents {
            if let Some(tv_browse) = contents.tv_browse_renderer {
                if let Some(content) = tv_browse.content {
                    if let Some(surface) = content.tv_surface_content_renderer {
                        if let Some(surface_content) = surface.content {
                            if let Some(section) = surface_content.section_list_renderer {
                                for content in section.contents {
                                    if let Some(shelf) = content.shelf_renderer {
                                        if let Some(header) = shelf.header_renderer {
                                            if let Some(shelf_header) = header.shelf_header_renderer {
                                                if let Some(avatar) = shelf_header.avatar_lockup {
                                                    if let Some(avatar_renderer) = avatar.avatar_lockup_renderer {
                                                        if let Some(title) = avatar_renderer.title {
                                                            // Return true if simple text is "Subscriptions"
                                                            if title.simple_text == "Subscriptions" {
                                                                return Ok(true);
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
                }
            }
        }

        Ok(false)
    }
}