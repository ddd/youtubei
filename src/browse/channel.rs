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
use std::collections::HashSet;

const ALL_COUNTRIES: &[&str] = &[
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS",
    "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", "DO", "DZ", "EC", "EE",
    "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF",
    "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM",
    "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE", "JM",
    "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC",
    "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK",
    "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA",
    "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG",
    "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW",
    "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS",
    "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO",
    "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
    "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW"
];

pub struct GetChannelRequest<'a> {
    pub client: &'a mut Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    pub ip: &'a str,
    pub fields: InnerTubeRequestFields<'a>,
    pub channel_id: String,
}

impl<'a> AsMut<InnerTubeRequestFields<'a>> for GetChannelRequest<'a> {
    fn as_mut(&mut self) -> &mut InnerTubeRequestFields<'a> {
        &mut self.fields
    }
}

impl<'a> GetChannelRequest<'a> {

    pub async fn send(self) -> Result<Channel, YouTubeError> {
        let request = youtube::BrowseRequest {
            context: Some(youtube::Context {
                client: Some(youtube::Client { client_name: 1, client_version: "2.20240614.01.00".to_string() })
            }),
            browse_id: Some(self.channel_id.clone()),
            params: Some("EgZ2aWRlb3PyBgQKAjoA".parse().unwrap()),
            continuation: None,
        };

        let mut payload = Vec::new();
        request.encode(&mut payload).map_err(|e| YouTubeError::Other(Box::new(e)))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("https://{}/youtubei/v1/browse", self.ip))
            .header("Host", "youtubei.googleapis.com")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Goog-Fieldmask", "contents.twoColumnBrowseResultsRenderer.tabs.tabRenderer.title,header.pageHeaderRenderer.content.pageHeaderViewModel(title.dynamicTextViewModel.text.attachmentRuns.element.type.imageType.image.sources.clientResource.imageName,banner.imageBannerViewModel.image.sources.url),metadata.channelMetadataRenderer(title,description,avatar.thumbnails.url,facebookProfileId),microformat.microformatDataRenderer(noindex,unlisted,familySafe,tags,availableCountries),alerts.alertRenderer.text.simpleText,header(carouselHeaderRenderer.contents.carouselItemRenderer.carouselItems.defaultPromoPanelRenderer.title.runs.text,pageHeaderRenderer)")
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

        // Start with a default channel struct
        let mut channel = Channel {
            user_id: self.channel_id.strip_prefix("UC").unwrap().to_string(),
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
            family_safe: true,
            blocked_countries: Vec::new(),
            channel_tabs: Vec::new(),
            has_carousel: false,
            cms_association: None
        };

        if let Some(contents) = &response.contents {
            if let Some(renderer) = &contents.two_column_browse_results_renderer {
                // Collect all tab titles
                channel.channel_tabs = renderer.tabs.iter()
                    .filter_map(|tab| {
                        tab.tab_renderer.as_ref().map(|tab_renderer| tab_renderer.title.clone())
                    })
                    .collect();
            }
        }

        // Try to extract verification status and OAC badge from header if available
        if let Some(header) = &response.header {
            if let Some(renderer) = &header.page_header_renderer {
                if let Some(content) = &renderer.content {
                    if let Some(view_model) = &content.page_header_view_model {
                        if let Some(title) = &view_model.title {
                            if let Some(text_view_model) = &title.dynamic_text_view_model {
                                if let Some(text) = &text_view_model.text {
                                    if let Some(runs) = &text.attachment_runs {
                                        if let Some(run) = &runs.element {
                                            if let Some(type_field) = &run.r#type {
                                                if let Some(image_type) = &type_field.image_type {
                                                    if let Some(image) = &image_type.image {
                                                        if let Some(sources) = &image.sources {
                                                            if let Some(client_resource) = &sources.client_resource {
                                                                match client_resource.image_name.as_str() {
                                                                    "CHECK_CIRCLE_FILLED" => channel.verified = true,
                                                                    "MUSIC_FILLED" => channel.oac = true,
                                                                    _ => {},
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
        }

        // Try to extract channel metadata if available
        if let Some(metadata) = &response.metadata {
            if let Some(renderer) = &metadata.channel_metadata_renderer {
                channel.display_name = renderer.title.clone();
                channel.description = renderer.description.clone();
                
                if let Some(avatar) = &renderer.avatar {
                    if let Some(thumbnails) = &avatar.thumbnails {
                        let avatar_url = &thumbnails.url;
                        // Check if it's a default profile picture
                        if let Some(index) = avatar_url.find(".googleusercontent.com/") {
                            let stripped_url = &avatar_url[(index + ".googleusercontent.com/".len())..];
                            // Remove everything after and including '='
                            let clean_url = stripped_url.split('=').next().unwrap_or(stripped_url);
                            channel.profile_picture = Some(clean_url.to_string());
                        }
                    }
                }
            }
        }

        // Channel banner
        if let Some(header) = &response.header {
            if let Some(renderer) = &header.page_header_renderer {
                if let Some(content) = &renderer.content {
                    if let Some(view_model) = &content.page_header_view_model {
                        if let Some(banner) = &view_model.banner {
                            if let Some(banner_view_model) = &banner.image_banner_view_model {
                                if let Some(image) = &banner_view_model.image {
                                    if let Some(sources) = &image.sources.first() {
                                        let banner_url = &sources.url;
                                        // Remove the base URL prefix by finding the first occurrence of googleusercontent.com/
                                        if let Some(index) = banner_url.find(".googleusercontent.com/") {
                                            let stripped_url = &banner_url[(index + ".googleusercontent.com/".len())..];
                                            // Remove everything after and including '='
                                            let clean_url = stripped_url.split('=').next().unwrap_or(stripped_url);
                                            channel.banner = Some(clean_url.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for carousel header renderer
        if let Some(header) = &response.header {
            if let Some(_carousel_renderer) = &header.carousel_header_renderer {
                channel.has_carousel = true;
            }
        }

        // Try to extract microformat data if available
        if let Some(microformat) = &response.microformat {
            if let Some(renderer) = &microformat.microformat_data_renderer {
                channel.no_index = renderer.noindex;
                channel.unlisted = renderer.unlisted;
                channel.family_safe = renderer.family_safe;
                channel.tags = renderer.tags.clone();

                // Calculate blocked countries
                let available: HashSet<_> = renderer.available_countries.iter().cloned().collect();
                let all: HashSet<_> = ALL_COUNTRIES.iter().map(|&s| s.to_string()).collect();
                channel.blocked_countries = all.difference(&available)
                    .cloned()
                    .collect::<Vec<_>>();
            }
        }

        // Check for channel status from alerts if present
        if let Some(alerts) = &response.alerts {
            if let Some(alert) = &alerts.alert_renderer {
                if let Some(text) = &alert.text {
                    match text.simple_text.as_str() {
                        "This channel is not available." => channel.hidden = true,
                        "This channel does not exist." => channel.deleted = true,
                        other => {
                            channel.terminated = true;
                            channel.termination_reason = other.to_string();
                        }
                    }
                }
            }
        }

        Ok(channel)
    
    }

}

