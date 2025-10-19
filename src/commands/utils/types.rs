//! Type for utils CLI

use serde::{Deserialize, Serialize};

/// User Response
#[derive(Deserialize)]
pub(crate) struct UserResponse {
    /// User definition
    pub user: Option<User>,
}

/// User type
#[derive(Deserialize)]
pub(crate) struct User {
    /// Id of user
    pub id: i64,
}

/// Crates info list
#[derive(Deserialize)]
pub(crate) struct CrateResponse {
    /// crates list
    pub crates: Vec<CrateInfo>,
}

/// crate info
#[derive(Deserialize)]
pub(crate) struct CrateInfo {
    /// Id of crate
    pub id: String,
}

/// crate data from crates.io
#[derive(Deserialize, Serialize, Debug)]
pub struct CrateData {
    /// inner crate data
    #[serde(rename = "crate")]
    pub krate: CrateInnerData,
}

/// Crate inner data
#[derive(Deserialize, Serialize, Debug)]
pub struct CrateInnerData {
    /// crate name
    pub name: String,
    /// repository url
    pub repository: Option<String>,
    /// homepage url
    pub homepage: Option<String>,
    /// documentation url
    pub documentation: Option<String>,
    /// description
    pub description: Option<String>,
}
