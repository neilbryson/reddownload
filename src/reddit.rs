use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct SecureMedia {
    pub(crate) fallback_url: String,
    pub(crate) height: i16,
    pub(crate) width: i16,
}

#[derive(Deserialize, Debug)]
pub struct PostData {
    pub(crate) subreddit: String,
    pub(crate) secure_media: Option<HashMap<String, SecureMedia>>,
}

#[derive(Deserialize, Debug)]
pub struct ListingDataChild {
    pub(crate) data: PostData,
}

#[derive(Deserialize, Debug)]
pub struct ListingData {
    pub(crate) children: Vec<ListingDataChild>,
}

#[derive(Deserialize, Debug)]
pub struct RootResponse {
    pub(crate) data: ListingData,
}
