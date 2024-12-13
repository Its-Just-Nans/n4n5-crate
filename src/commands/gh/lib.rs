//!
//! To see all subcommands, run:
//! ```shell
//! n4n5 gh
//! ```
//!
use std::{fs::write, path::PathBuf, process::Command};

use clap::{ArgMatches, Command as ClapCommand};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
};

use super::types::{GhPageInfo, GhResponse};

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
            .subcommand(ClapCommand::new("pulls").about("save pulls"))
            .subcommand(ClapCommand::new("projects").about("save projects"))
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
        if let Some(matches) = args_matches.subcommand_matches("pulls") {
            Gh::save_pulls(config, matches);
        } else if let Some(matches) = args_matches.subcommand_matches("projects") {
            Gh::save_projects(config, matches);
        }
    }
}

impl Gh {
    /// Save the pulls to the specified file
    fn save_pulls(config: &mut Config, _matches: &ArgMatches) {
        let pulls_path = match &config.config_data.gh {
            Some(Gh {
                file_pulls: Some(path),
                ..
            }) => PathBuf::from(path),
            _ => {
                println!("Please enter the path to the folder where to save the files:");
                let file_path = input_path();
                let cloned_path = file_path.1.clone();
                config.update(|config_data| {
                    if let Some(gh_config) = config_data.gh.as_mut() {
                        gh_config.file_pulls = Some(cloned_path);
                    } else {
                        config_data.gh = Some(Gh {
                            file_pulls: Some(cloned_path),
                            ..Default::default()
                        });
                    }
                    config_data
                });
                file_path.0
            }
        };
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
            .replace("100)", format!("100{})", add).as_str());
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .expect("failed to execute process");
            let output = String::from_utf8_lossy(&output.stdout).to_string();
            let output = serde_json::from_str::<GhResponse>(&output)
                .expect("Unable to parse json from gh command");
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
        all_pulls.serialize(&mut ser).unwrap();
        write(&pulls_path, buf).expect("Unable to write to file");
        println!(
            "Saving {} pulls to {}",
            all_pulls.len(),
            pulls_path.display()
        );
    }

    /// Save the projects to the specified file
    fn save_projects(config: &mut Config, _matches: &ArgMatches) {
        let projects_path = match &config.config_data.gh {
            Some(gh) => gh.file_projects.clone(),
            None => None,
        };
        let projects_path = match projects_path {
            Some(path) => path,
            None => {
                eprintln!("No projects file path specified");
                return;
            }
        };
        eprintln!("Saving projects to {}", projects_path);
    }
}
