use crate::models::*;
use crate::{youtube, InnerTubeRequestFields, YouTubeError};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode};
use prost::Message;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use scylla::frame::value::CqlTimestamp;
use urlencoding::decode;
use crate::utils::{parse_creation_date, parse_multiplied_string, parse_numeric_string};

static COUNTRY_CODES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../data/countries.json"))
        .expect("Failed to load country codes")
});

pub struct GetChannelExtendedRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel: &'a mut Channel,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetChannelExtendedRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetChannelExtendedRequest<'a> {

    pub async fn send(self) -> Result<(), YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { client_name: 1, client_version: "2.20240614.01.00".to_string() })
            }),
            browse_id: None,
            params: None,
            continuation: Some(crate::utils::generate_continuation_token(format!("UC{}", self.channel.user_id.clone()), "8gYrGimaASYKJDY3M2UzYjY0LTAwMDAtMjRmMy04ZjMyLTU4MjQyOWM2ODNjOA%3D%3D".to_string())),
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "onResponseReceivedEndpoints.appendContinuationItemsAction.continuationItems.aboutChannelRenderer.metadata.aboutChannelViewModel(country,subscriberCountText,viewCountText,joinedDateText.content,canonicalChannelUrl,videoCountText,signInForBusinessEmail.content,links.channelExternalLinkViewModel(title.content,link.content))")
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

        // Parse about channel data
        if let Some(endpoints) = response.on_response_received_endpoints {
            if let Some(action) = endpoints.append_continuation_items_action {
                for continuation_item in action.continuation_items {
                    if let Some(about_renderer) = continuation_item.about_channel_renderer {
                        if let Some(metadata) = about_renderer.metadata {
                            if let Some(view_model) = metadata.about_channel_view_model {
                                // Parse subscriber count (e.g. "324M subscribers")
                                if !view_model.subscriber_count_text.is_empty() {
                                    let sub_count = view_model.subscriber_count_text
                                        .trim_end_matches(" subscribers")
                                        .trim();
                                    self.channel.subscribers = Some(parse_multiplied_string(sub_count));
                                } else {
                                    self.channel.subscribers = Some(0);
                                }

                                // Remove "http://www.youtube.com/" prefix
                                if let Some(path) = view_model.canonical_channel_url.strip_prefix("http://www.youtube.com/") {
                                    // If starts with @, get the handle
                                    if let Some(handle) = path.strip_prefix('@') {
                                        if let Ok(decoded_handle) = decode(handle) {
                                            self.channel.handle = Some(decoded_handle.into_owned());
                                        }
                                    }
                                }
                                
                                // Parse view count (e.g. "61,943,233,845 views")
                                if !view_model.view_count_text.is_empty() {
                                    let view_count = view_model.view_count_text
                                        .trim_end_matches(" views")
                                        .trim();
                                    self.channel.views = parse_numeric_string(view_count);
                                }

                                // Parse video count (e.g. "823 videos")
                                if !view_model.video_count_text.is_empty() {
                                    let video_count = view_model.video_count_text
                                        .trim_end_matches(" videos")
                                        .trim();
                                    self.channel.videos = parse_numeric_string(video_count) as i32;
                                }

                                // Parse country and convert to 2-letter code
                                if !view_model.country.is_empty() {
                                    self.channel.country = COUNTRY_CODES.get(&view_model.country)
                                        .map(|code| code.to_string());
                                }

                                // Parse business email status
                                self.channel.has_business_email = view_model.sign_in_for_business_email.is_some();

                                // Parse external links
                                self.channel.links = view_model.links.into_iter()
                                    .filter_map(|link| {
                                        if let Some(link_view_model) = link.channel_external_link_view_model {
                                            if let (Some(title), Some(url)) = (link_view_model.title, link_view_model.link) {
                                                return Some(Link { name: title.content, url: url.content });
                                            }
                                            None
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                // Parse join date (e.g. "Joined Feb 19, 2012")
                                if let Some(joined_date) = view_model.joined_date_text {
                                    if let Some(date_str) = joined_date.content.strip_prefix("Joined ") {
                                        self.channel.created_at = CqlTimestamp(parse_creation_date(date_str) as i64 * 1000);
                                    }
                                }
                            }
                        }
                    }
                    
                }
            }
        }

        Ok(())
    
    }

}