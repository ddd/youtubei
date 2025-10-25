
use scylla::frame::value::CqlTimestamp;

#[derive(Debug, Clone)]
pub struct Link {
    pub name: String,
    pub url: String
}

#[derive(Debug, Clone)]
pub struct ContentOwnerAssociation {
    pub cms_id: String,
    pub created_at: i64,
    pub activated_at: i64,
    pub can_web_claim: bool,
    pub can_view_revenue: bool,
    pub can_enable_cid: bool,
    pub disable_ad_blocking_settings: bool,
    pub default_channel: bool,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub user_id: String,
    pub handle: Option<String>,
    pub display_name: String,
    pub description: String,
    pub profile_picture: Option<String>,
    pub banner: Option<String>,
    pub verified: bool,
    pub oac: bool,
    pub monetized: Option<bool>,
    pub subscribers: Option<i64>,
    pub views: i64,
    pub videos: i32,
    pub created_at: CqlTimestamp,
    pub country: Option<String>,
    pub has_business_email: bool,
    pub links: Vec<Link>,
    pub tags: Vec<String>,
    pub deleted: bool,
    pub hidden: bool,
    pub terminated: bool,
    pub termination_reason: String,
    pub no_index: bool,
    pub unlisted: bool,
    pub family_safe: bool,
    pub blocked_countries: Vec<String>,
    pub channel_tabs: Vec<String>,
    pub has_carousel: bool,
    pub cms_association: Option<ContentOwnerAssociation>
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub user_id: String,
    pub comment_id: String,
    pub comment_text: String,
    pub created_at: i64,
    pub video_id: String,
    pub likes: i32,
    pub replies: i32,
    pub edited: bool,
}

#[derive(Debug, Clone)]
pub struct Video {
    pub video_id: String,
    pub views: i64,
    pub hidden_view_count: bool,
    pub badge: Option<String>,
    pub length_seconds: Option<i32>,
    pub approx_published_time: Option<i64>
}

#[derive(Debug, Clone)]
pub struct WatchNext {
    pub user_id: String,
    pub video_id: String
}