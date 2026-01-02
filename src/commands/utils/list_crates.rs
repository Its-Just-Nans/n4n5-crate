//! [`list_crates`] function

use std::{fmt::Write, fs, path::PathBuf, thread, time::Duration, vec};

use crate::{
    commands::{
        gh::lib::get_github_username,
        utils::types::{CrateData, CrateInnerData, CrateResponse, UserResponse},
    },
    config::Config,
    errors::GeneralError,
    utils::{pretty_print, table_to_markdown_table},
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
    #[arg(long, default_value_t = get_github_username())]
    username: String,
    /// Specify user agent
    #[arg(long, default_value_t = get_user_agent())]
    user_agent: String,

    /// Output markdown
    #[arg(long)]
    output_markdown: Option<PathBuf>,

    /// Output list
    #[arg(long)]
    output_list: Option<PathBuf>,

    /// Output list long/full
    #[arg(long)]
    output_list_full: Option<PathBuf>,

    /// Request delay (in milliseconds)
    #[arg(long, default_value_t = 500)]
    delay: u64,

    /// Filter crates
    #[arg(long)]
    filtered: Option<String>,

    /// Filter crates
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Filter crates
    #[arg(long)]
    specials: Option<String>,
}

impl UtilsListCrates {
    /// Get all crates name
    /// # Errors
    /// Error if request fails
    pub fn get_all_crates(&self, verbose: bool, delay: u64) -> Result<Vec<String>, GeneralError> {
        let client = Client::builder().user_agent(&self.user_agent).build()?;
        let per_page: usize = 50;

        // Step 1: Fetch user ID
        let user_url = format!("https://crates.io/api/v1/users/{}", self.username);
        let user_res: UserResponse = client.get(&user_url).send()?.json()?;

        let Some(user_id) = user_res.user else {
            let msg = format!("User '{}' not found on crates.io.", self.username);
            eprintln!("{msg}");
            return Err(GeneralError::new(msg));
        };
        if verbose {
            println!(
                "Fetching crates for user '{}' (ID: {})...",
                self.username, user_id.id
            );
        }
        let mut page = 1;
        let mut all_crates: Vec<String> = Vec::new();
        loop {
            thread::sleep(Duration::from_millis(delay)); // avoid rate limit

            let url = format!(
                "https://crates.io/api/v1/crates?user_id={}&page={}&per_page={}",
                user_id.id, page, per_page
            );

            let resp: CrateResponse = client.get(&url).send()?.json()?;

            if resp.crates.is_empty() {
                break;
            }

            let crates_len = resp.crates.len();

            for c in resp.crates {
                all_crates.push(c.id.clone());
            }

            if crates_len < per_page {
                break;
            }

            page += 1;
        }
        if verbose {
            println!("Found {} crates", all_crates.len());
        }
        Ok(all_crates)
    }

    /// Generate markdown tables as string
    /// # Errors
    /// Error if fails to convert to string
    pub fn generate_markdown_table<I>(&self, rows: I) -> Result<String, GeneralError>
    where
        I: Iterator<Item = [String; 4]>,
    {
        let specials_crates = if let Some(spe) = &self.specials {
            spe.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec![]
        };
        let header = [[
            "Crate".to_string(),
            "Description".to_string(),
            "Homepage && Repo".to_string(),
        ]
        .to_vec()];
        let (mut table1, mut table2, mut table3) = (Vec::new(), Vec::new(), Vec::new());

        if let Some(pattern) = &self.filtered {
            for row in rows {
                let to_push = row[1..].to_vec();
                if specials_crates.contains(&row[0]) {
                    table3.push(to_push);
                } else if row[2].to_lowercase().starts_with(&pattern.to_lowercase()) {
                    table2.push(to_push);
                } else {
                    table1.push(to_push);
                }
            }
        } else {
            table1 = rows.map(|row| row.to_vec()).collect();
        }
        let mut buf = String::new();
        let table1 = header.clone().into_iter().chain(table1);
        let table1_markdown = table_to_markdown_table(table1, 3)?;
        write!(&mut buf, "{table1_markdown}")?;
        if !table2.is_empty() {
            if let Some(pattern) = &self.filtered {
                let mut patt = pattern.clone();
                if let Some(r) = patt.get_mut(0..1) {
                    r.make_ascii_uppercase();
                }
                writeln!(&mut buf, "\n## {patt}\n")?;
            } else {
                writeln!(&mut buf, "\n## Filtered\n")?;
            }
            let table2 = header.clone().into_iter().chain(table2);
            let table2_markdown = table_to_markdown_table(table2, 3)?;
            write!(&mut buf, "{table2_markdown}")?;
        }
        if !table3.is_empty() {
            writeln!(&mut buf, "\n## Others\n")?;
            let table3 = header.into_iter().chain(table3);
            let table3_markdown = table_to_markdown_table(table3, 3)?;
            write!(&mut buf, "{table3_markdown}")?;
        }
        Ok(buf)
    }

    /// Get the music file path
    /// # Errors
    /// Fails if the file cannot be found
    pub fn list_crates(&self, _config: &mut Config) -> Result<(), GeneralError> {
        let all_crates = self.get_all_crates(self.verbose, self.delay)?;
        if let Some(list_file) = &self.output_list {
            pretty_print(&all_crates, list_file)?;
        }
        if self.output_list_full.is_none() && self.output_markdown.is_none() {
            return Ok(());
        }
        let all_crates_infos: Vec<CrateData> = all_crates
            .iter()
            .map(|crate_name| Self::get_one_crate(crate_name, self.delay))
            .filter_map(|res| match res {
                Ok(val) => Some(val),
                Err(err) => {
                    eprintln!("Error fetching crate: {err}");
                    None
                }
            })
            .collect();
        if let Some(file_list_full) = &self.output_list_full {
            pretty_print(&all_crates_infos, file_list_full)?;
        }
        let Some(file_markdown) = &self.output_markdown else {
            return Ok(());
        };
        let rows = all_crates_infos.iter().map(|one_crate| {
            let CrateInnerData {
                description,
                name,
                repository,
                homepage,
                documentation,
                ..
            } = &one_crate.krate;
            let default_text = "N/A".to_string();
            let name_with_url = format!("[{}](https://crates.io/crates/{})", &name, &name);
            let desc = description.clone().unwrap_or(default_text.clone());
            let homepage = if let Some(h) = homepage {
                format!("<{h}>")
            } else {
                default_text.clone()
            };
            let url = if let Some(repo) = repository {
                format!("<{repo}>")
            } else {
                default_text.clone()
            };
            let docs = if let Some(doc) = documentation {
                format!("<{doc}>")
            } else {
                default_text.clone()
            };
            let infos = format!("{homepage} <br/> {url} <br/> {docs}");
            [name.clone(), name_with_url, desc, infos]
        });
        let tables = self.generate_markdown_table(rows)?;
        let mut buf = String::new();
        writeln!(&mut buf, "# crates")?;
        writeln!(&mut buf)?;
        writeln!(&mut buf, "- <https://crates.io/users/{}>", self.username)?;
        writeln!(&mut buf, "- <https://lib.rs/~{}/dash>", self.username)?;
        writeln!(&mut buf)?;
        writeln!(&mut buf, "## Crates")?;
        writeln!(&mut buf)?;
        write!(&mut buf, "{tables}")?;

        if file_markdown == &PathBuf::from("-") {
            print!("{buf}");
            return Ok(());
        }
        fs::write(file_markdown, buf)?;
        println!("Written to {}", file_markdown.display());
        Ok(())
    }

    /// Get info for one crate
    /// # Errors
    /// Error if request fails or serde fails
    pub fn get_one_crate(crate_name: &String, delay: u64) -> Result<CrateData, GeneralError> {
        // Sleep 0.5 seconds to avoid rate limiting
        thread::sleep(Duration::from_millis(delay));
        let user_agent = get_user_agent();
        let url = format!("https://crates.io/api/v1/crates/{crate_name}");
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
