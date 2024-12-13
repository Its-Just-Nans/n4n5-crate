use serde::{Deserialize, Serialize};

/// Github Response
#[derive(Deserialize, Default)]
pub struct GhResponse {
    /// Data
    pub data: GhData,
}

/// Data
#[derive(Deserialize, Default)]
pub struct GhData {
    /// User
    pub user: GhUser,
}

/// User
#[derive(Deserialize, Default)]
pub struct GhUser {
    /// Login
    #[serde(rename = "pullRequests")]
    pub pull_requests: GhPullRequests,
}

/// Pull requests
#[derive(Deserialize, Default)]
pub struct GhPullRequests {
    /// Edges
    pub edges: Vec<GhPullRequest>,

    /// Page info
    #[serde(rename = "pageInfo")]
    pub page_info: GhPageInfo,
}

/// Page info
#[derive(Deserialize, Default)]
pub struct GhPageInfo {
    /// End cursor
    #[serde(rename = "endCursor")]
    pub end_cursor: String,

    /// Start cursor
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

/// Pull request
#[derive(Deserialize, Serialize, Default)]
pub struct GhPullRequest {
    /// Node
    node: GhPullRequestNode,
}

/// Pull request node
#[derive(Deserialize, Serialize, Default)]
pub struct GhPullRequestNode {
    /// Id
    id: String,

    /// Number
    number: i32,

    /// Title
    title: String,

    /// Url
    url: String,

    /// State
    state: String,

    /// Created at
    #[serde(rename = "createdAt")]
    created_at: String,

    /// Base repository
    #[serde(rename = "baseRepository")]
    base_repository: GhBaseRepository,
}

/// Base repository
#[derive(Deserialize, Serialize, Default)]
pub struct GhBaseRepository {
    /// Url
    url: String,

    /// Name
    name: String,

    /// Description
    description: Option<String>,

    /// Owner
    owner: GhOwner,

    /// Languages
    languages: GhLanguages,
}

/// Owner
#[derive(Deserialize, Serialize, Default)]
pub struct GhOwner {
    /// Login
    login: String,
}

/// Languages
#[derive(Deserialize, Serialize, Default)]
pub struct GhLanguages {
    /// Nodes
    nodes: Vec<GhLanguage>,
}

/// Language
#[derive(Deserialize, Serialize, Default)]
pub struct GhLanguage {
    /// Name of the language
    name: String,
}
