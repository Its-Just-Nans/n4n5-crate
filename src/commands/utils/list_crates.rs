//! list_crates

use std::{fs, thread, time::Duration};

use crate::{
    commands::{
        gh::lib::get_github_username,
        utils::types::{CrateData, CrateResponse, UserResponse},
    },
    config::Config,
    errors::GeneralError,
};
use clap::Parser;
use reqwest::blocking::Client;

/// Get user agent
fn get_user_agent() -> String {
    "n4n5 (https://github.com/Its-Just-Nans/n4n5)".to_string()
}

/// A simple CLI example
#[derive(Parser, Debug, Clone)]
#[command(name = "list_crates")]
pub struct UtilsListCrates {
    /// Specify username
    #[arg(short, long)]
    username: Option<String>,

    /// Output filename
    #[arg(short, long)]
    output_file: Option<String>,

    /// Request delay (in seconds)
    #[arg(short, long)]
    delay: Option<f64>,

    /// Full crates info
    #[arg(short = 'f', long, default_value_t = false)]
    full: bool,
}

impl UtilsListCrates {
    /// Get the music file path
    /// # Errors
    /// Fails if the file cannot be found
    pub fn list_crates(_config: &mut Config, options: UtilsListCrates) -> Result<(), GeneralError> {
        let username = options.username.unwrap_or(get_github_username());
        let output_file = options.output_file.unwrap_or("list.json".to_string());
        let delay = options.delay.unwrap_or(0.5);
        let delay = (delay * 1000.0) as u64;
        let need_full = options.full;
        let verbose = output_file != "-";
        let per_page: usize = 50;
        let user_agent = get_user_agent();

        let client = Client::builder().user_agent(user_agent).build()?;

        // Step 1: Fetch user ID
        let user_url = format!("https://crates.io/api/v1/users/{username}");
        let user_res: UserResponse = client.get(&user_url).send()?.json()?;

        let user_id = match user_res.user {
            Some(u) => u.id,
            None => {
                eprintln!("User '{}' not found on crates.io.", username);
                std::process::exit(1);
            }
        };

        if verbose {
            println!(
                "Fetching crates for user '{}' (ID: {})...",
                username, user_id
            );
        }

        // Step 2: Paginate crates
        let mut page = 1;
        let mut all_crates: Vec<String> = Vec::new();

        loop {
            thread::sleep(Duration::from_millis(delay)); // avoid rate limit

            let url = format!(
                "https://crates.io/api/v1/crates?user_id={}&page={}&per_page={}",
                user_id, page, per_page
            );

            let resp: CrateResponse = client.get(&url).send()?.json()?;

            if resp.crates.is_empty() {
                break;
            }

            for c in resp.crates.iter() {
                all_crates.push(c.id.clone());
            }

            if resp.crates.len() < per_page {
                break;
            }

            page += 1;
        }
        if verbose {
            println!("Found {} crates", all_crates.len());
        }
        if !need_full {
            let stringify = serde_json::to_string_pretty(&all_crates)?;
            if output_file == "-" {
                println!("{}", stringify);
            } else {
                fs::write(&output_file, stringify)?;
                println!("Written to {}", output_file);
            }
            return Ok(());
        }

        let all_crates_infos: Vec<CrateData> = all_crates
            .iter()
            .map(|crate_name| Self::get_one_crate(crate_name, delay))
            .filter_map(|res| match res {
                Ok(val) => Some(val),
                Err(err) => {
                    eprintln!("Error fetching crate: {}", err);
                    None
                }
            })
            .collect();
        let stringify = serde_json::to_string_pretty(&all_crates_infos)?;
        if output_file == "-" {
            println!("{}", stringify);
        } else {
            fs::write(&output_file, stringify)?;
            println!("Written to {}", output_file);
        }
        Ok(())
    }

    /// Get info for one crate
    /// # Errors
    /// Error if request fails or serde fails
    pub fn get_one_crate(crate_name: &String, delay: u64) -> Result<CrateData, GeneralError> {
        // Sleep 0.5 seconds to avoid rate limiting
        thread::sleep(Duration::from_millis(delay));
        let user_agent = get_user_agent();
        let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
        let client = Client::new();

        let response = client
            .get(&url)
            .header("User-Agent", user_agent)
            .send()?
            .error_for_status()?
            .text()?;

        let crate_data: CrateData = serde_json::from_str(&response)?;
        Ok(crate_data)
    }
}
