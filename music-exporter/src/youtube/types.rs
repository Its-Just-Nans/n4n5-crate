//! YouTube API types

/// Youtube Related Playlists
#[derive(Debug, serde::Deserialize)]
pub struct RelatedPlaylists {
    /// Liked videos
    pub likes: String,
    // pub uploads: String,
}

/// content details
#[derive(Debug, serde::Deserialize)]
pub struct ContentDetails {
    /// Related playlists
    #[serde(rename = "relatedPlaylists")]
    pub related_playlists: RelatedPlaylists,
}

/// channel
#[derive(Debug, serde::Deserialize)]
pub struct Channel {
    /// contentDetails
    #[serde(rename = "contentDetails")]
    pub content_details: ContentDetails,
}

/// API response
#[derive(Debug, serde::Deserialize)]
pub struct APIResponse {
    /// Items
    pub items: Vec<Channel>,
}

/// Google access token
#[derive(Debug, serde::Deserialize)]
pub struct GoogleAccessToken {
    /// Access token
    pub access_token: String,
    // pub expires_in: i32,
    // pub scope: String,
    // pub token_type: String,
}

/// Resource ID Playlist item
#[derive(Debug, serde::Deserialize)]
pub struct ResourceIdPlaylistItem {
    /// Video ID
    #[serde(rename = "videoId")]
    pub video_id: String,
}

/// Snippet playlist item
#[derive(Debug, serde::Deserialize)]
pub struct SnippetPlaylistItem {
    /// Title
    pub title: String,

    /// Published at
    #[serde(rename = "publishedAt")]
    pub published_at: String,

    /// Channel title
    #[serde(rename = "videoOwnerChannelTitle")]
    pub video_owner_channel_title: Option<String>,

    /// Resource ID
    #[serde(rename = "resourceId")]
    pub resource_id: ResourceIdPlaylistItem,
}

/// See https://developers.google.com/youtube/v3/docs/playlistItems#resource
#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItem {
    /// Snippet
    pub snippet: SnippetPlaylistItem,
}

/// Playlist items
#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItems {
    /// Next page token
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    /// Items
    pub items: Vec<PlaylistItem>,
}
