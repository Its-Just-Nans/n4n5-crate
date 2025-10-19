//! To see all subcommands, run:
//! ```shell
//! n4n5 gh
//! ```

use clap::{arg, ArgAction, ArgMatches, Command as ClapCommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::write, path::PathBuf, process::Command};

use crate::{
    cli::{input_path, CliCommand},
    commands::gh::types::GhProject,
    config::Config,
    config_path,
    errors::GeneralError,
};

use super::types::{GhPageInfo, GhResponse};

/// Get github username
pub(crate) fn get_github_username() -> String {
    "Its-Just-Nans".to_string()
}

/// Github configuration
#[derive(Deserialize, Serialize, Default)]
pub struct Gh {
    /// Path to the movies file
    pub username: Option<String>,

    /// Path to the pulls file
    pub file_pulls: Option<String>,

    /// Path to the projects file
    pub file_projects: Option<String>,
}

impl CliCommand for Gh {
    fn get_subcommand() -> ClapCommand {
        ClapCommand::new("gh")
            .about("Github cli wrap")
            .subcommand(
                ClapCommand::new("pulls").about("save pulls").arg(
                    arg!(
                        -j --json "print as json"
                    )
                    .action(ArgAction::SetTrue)
                    .required(false),
                ),
            )
            .subcommand(
                ClapCommand::new("projects").about("save projects").arg(
                    arg!(
                        -j --json "print as json"
                    )
                    .action(ArgAction::SetTrue)
                    .required(false),
                ),
            )
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) -> Result<(), GeneralError> {
        if let Some(matches) = args_matches.subcommand_matches("pulls") {
            return Gh::save_pulls(config, Some(matches));
        } else if let Some(matches) = args_matches.subcommand_matches("projects") {
            return Gh::save_projects(config, Some(matches));
        }
        Ok(())
    }
}

/// Project type
enum ProjectType {
    /// Gists
    Gists,
    /// Repos
    Repos,
}

impl Gh {
    /// Sync the github data
    /// # Errors
    /// Fails if unable to save the pulls or projects
    pub fn sync_github(
        config: &mut Config,
        matches: Option<&ArgMatches>,
    ) -> Result<(), GeneralError> {
        println!("Syncing github data");
        Gh::save_pulls(config, matches)?;
        Gh::save_projects(config, matches)?;
        Ok(())
    }

    /// Save the pulls to the specified file
    /// # Errors
    /// Fails if unable to write to file
    fn save_pulls(config: &mut Config, _matches: Option<&ArgMatches>) -> Result<(), GeneralError> {
        let pulls_path = config_path!(config, gh, Gh, file_pulls, "pulls file");
        println!("Saving pulls to {}", pulls_path.display());
        let mut response_data = GhPageInfo {
            has_next_page: true,
            ..Default::default()
        };
        let mut all_pulls = Vec::new();
        while response_data.has_next_page {
            let add = match response_data.end_cursor.trim().is_empty() {
                true => "".to_string(),
                false => format!(", after: \"{}\"", response_data.end_cursor),
            };
            let command = "gh api graphql -F owner='Its-Just-Nans' -f query='
    query($owner: String!) {
        user(login: $owner) {
            pullRequests(first: 100) {
                edges {
                    node {
                        id
                        number
                        title
                        url
                        state
                        createdAt
                        baseRepository {
                            url
                            name
                            description
                            owner {
                            login
                            }
                            languages(first: 1) {
                                nodes {
                                    name
                                    color
                                }
                            }
                        }
                    }
                }
                pageInfo {
                    endCursor
                    startCursor
                    hasNextPage
                    hasPreviousPage
                }
            }
        }
    }'"
            .replace("100)", format!("100{add})",).as_str());
            if config.debug > 0 {
                println!("Running command:");
                println!("{command}");
            }
            let output = Command::new("sh").arg("-c").arg(command).output()?;
            let output = String::from_utf8_lossy(&output.stdout).to_string();
            if config.debug > 1 {
                println!("Output:");
                println!("{output}");
            }
            let output = serde_json::from_str::<GhResponse>(&output)?;
            println!(
                "Received {} pulls requests",
                output.data.user.pull_requests.edges.len()
            );
            all_pulls.extend(output.data.user.pull_requests.edges);
            response_data = output.data.user.pull_requests.page_info;
        }
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        all_pulls.serialize(&mut ser)?;
        write(&pulls_path, buf)?;
        println!(
            "Saving {} pulls to {}",
            all_pulls.len(),
            pulls_path.display()
        );
        Ok(())
    }

    /// Fetch projects with gh cli
    /// # Errors
    /// Fails if unable to fetch the projects
    fn fetch_projects(
        project_type: ProjectType,
        debug: u8,
    ) -> Result<Vec<GhProject>, GeneralError> {
        let mut response_data = GhPageInfo {
            has_next_page: true,
            ..Default::default()
        };
        let fetch_type = match project_type {
            ProjectType::Gists => "gists",
            ProjectType::Repos => "repositories",
        };
        let repo_arg = match project_type {
            ProjectType::Gists => "",
            ProjectType::Repos => "isFork: false, ownerAffiliations: [OWNER]",
        };
        let repo_data = match project_type {
            ProjectType::Gists => "",
            ProjectType::Repos => {
                "primaryLanguage {
                        name
                        color
                    }
                    homepageUrl
                    diskUsage
                    forkCount
                    licenseInfo {
                        name
                    }"
            }
        };
        let mut all_projects = Vec::new();
        while response_data.has_next_page {
            let add = match response_data.end_cursor.trim().is_empty() {
                true => "".to_string(),
                false => format!(", after: \"{}\", ", response_data.end_cursor),
            };
            let command = "gh api graphql -F owner='Its-Just-Nans' -f query='
    query( $owner: String!){
        user(login: $owner) {
            TYPE(first: 100,ADD REPO_ARG, privacy: PUBLIC) {
                pageInfo {
                    hasNextPage
                    endCursor
                    startCursor
                }
                nodes {
                    url
                    name
                    REPO_DATA
                    description
                    stargazerCount
                }
            }
        }
    }'"
            .replace("TYPE", fetch_type)
            .replace(",ADD", &add)
            .replace("REPO_ARG", repo_arg)
            .replace("REPO_DATA", repo_data);
            if debug > 1 {
                println!("Running command:");
                println!("{command}");
            }
            let output = Command::new("sh").arg("-c").arg(command).output()?;
            let output = String::from_utf8_lossy(&output.stdout).to_string();
            if debug > 2 {
                println!("Output:");
                println!("{output}");
            }
            let output = serde_json::from_str::<Value>(&output)?;
            match output {
                Value::Object(map) => {
                    if let Some(Value::Object(data)) = map.get("data") {
                        if let Some(Value::Object(user)) = data.get("user") {
                            if let Some(Value::Object(projects)) = user.get(fetch_type) {
                                if let Some(nodes) = projects.get("nodes") {
                                    let nodes: Vec<GhProject> =
                                        serde_json::from_value(nodes.clone())?;
                                    if debug > 0 {
                                        println!("Received {} {}", nodes.len(), fetch_type);
                                    }
                                    all_projects.extend(nodes);
                                }
                                response_data = serde_json::from_value(
                                    projects
                                        .get("pageInfo")
                                        .ok_or_else(|| {
                                            GeneralError::new(
                                                "Unable to find pageInfo in gh command".to_owned(),
                                            )
                                        })?
                                        .clone(),
                                )?;
                            }
                        }
                    }
                }
                _ => {
                    println!("Unable to parse json from gh command");
                }
            }
        }
        Ok(all_projects)
    }

    /// Save the projects to the specified file
    /// # Errors
    /// Fails if unable to write to file
    fn save_projects(
        config: &mut Config,
        matches: Option<&ArgMatches>,
    ) -> Result<(), GeneralError> {
        let is_json = match matches {
            Some(matches) => matches.get_flag("json"),
            None => false,
        };
        let projects_path = config_path!(config, gh, Gh, file_projects, "projects file");
        if !is_json {
            println!("Saving projects to {}", projects_path.display());
        }
        let debug_level = match is_json {
            true => 0,
            false => config.debug + 1,
        };
        let mut repos = Gh::fetch_projects(ProjectType::Repos, debug_level)?;
        repos.sort_by(|a, b| a.name.cmp(&b.name));
        let mut gists = Gh::fetch_projects(ProjectType::Gists, debug_level)?;
        gists.sort_by(|a, b| a.name.cmp(&b.name));
        if !is_json {
            println!(
                "Saving {} repos and {} gists to {}",
                repos.len(),
                gists.len(),
                projects_path.display()
            );
        }
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        repos.append(&mut gists);
        repos.serialize(&mut ser)?;
        if is_json {
            println!("{}", String::from_utf8_lossy(&buf));
        }
        write(&projects_path, buf)?;
        Ok(())
    }
}
