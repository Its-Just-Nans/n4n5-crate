//! Type for github graphql api
use serde::{Deserialize, Serialize};

/// Github Response
#[derive(Deserialize, Default, Debug)]
pub struct GhResponse {
    /// Data
    pub data: GhData,
}

/// Data
#[derive(Deserialize, Default, Debug)]
pub struct GhData {
    /// User
    pub user: GhUser,
}

/// User
#[derive(Deserialize, Default, Debug)]
pub struct GhUser {
    /// Login
    #[serde(rename = "pullRequests")]
    pub pull_requests: GhPullRequests,
}

/// Pull requests
#[derive(Deserialize, Default, Debug)]
pub struct GhPullRequests {
    /// Edges
    pub edges: Vec<GhPullRequest>,

    /// Page info
    #[serde(rename = "pageInfo")]
    pub page_info: GhPageInfo,
}

/// Page info
#[derive(Deserialize, Default, Debug)]
pub struct GhPageInfo {
    /// End cursor
    #[serde(rename = "endCursor")]
    pub end_cursor: String,

    /// Start cursor
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

/// Pull request
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct GhPullRequest {
    /// Node
    node: GhPullRequestNode,
}

/// Pull request node
#[derive(Deserialize, Serialize, Default, Debug)]
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
#[derive(Deserialize, Serialize, Default, Debug)]
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
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct GhOwner {
    /// Login
    login: String,
}

/// Languages
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct GhLanguages {
    /// Nodes
    nodes: Vec<GhLanguage>,
}

/// Language
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct GhLanguage {
    /// Name of the language
    name: String,

    /// Color of the language
    color: Option<String>,
}

/// License
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct GhLicenseInfo {
    /// Name of the language
    name: String,
}

/// Github Project Response (gist or repository)
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct GhProject {
    /// Project url
    pub url: String,

    /// Project name
    pub name: String,

    /// Project description
    pub description: Option<String>,

    /// stargazerCount
    #[serde(rename = "stargazerCount")]
    pub stargazer_count: i32,

    /// archivedAt
    #[serde(rename = "archivedAt")]
    pub archived_at: Option<String>,

    /// homepageUrl
    #[serde(rename = "homepageUrl")]
    pub homepage_url: Option<String>,

    /// Fork count
    #[serde(rename = "forkCount")]
    pub fork_count: Option<u64>,

    /// license Info
    #[serde(rename = "licenseInfo")]
    pub license_info: Option<GhLicenseInfo>,

    /// Disk usage
    #[serde(skip_serializing)]
    #[serde(rename = "diskUsage")]
    pub disk_usage: Option<u64>,

    /// primaryLanguage
    #[serde(rename = "primaryLanguage")]
    pub primary_language: Option<GhLanguage>,
}
