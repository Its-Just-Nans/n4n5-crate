//! list_crates

use std::{fmt::Write, fs, thread, time::Duration, vec};

use crate::{
    commands::{
        gh::lib::get_github_username,
        utils::types::{CrateData, CrateInnerData, CrateResponse, UserResponse},
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

/// Get default output file
fn get_default_output_file() -> String {
    "list.json".to_string()
}

/// A simple CLI example
#[derive(Parser, Debug, Clone)]
#[command(name = "list_crates")]
pub struct UtilsListCrates {
    /// Specify username
    #[arg(short, long, default_value_t = get_github_username())]
    username: String,
    /// Specify user agent
    #[arg(long, default_value_t = get_user_agent())]
    user_agent: String,

    /// Output filename
    #[arg(short, long, default_value_t = get_default_output_file())]
    output_file: String,

    /// Request delay (in seconds)
    #[arg(short, long, default_value_t = 0.5)]
    delay: f64,

    /// Full crates info
    #[arg(short = 'f', long, default_value_t = false)]
    full: bool,

    /// Output markdown
    #[arg(short = 'm', long, default_value_t = false)]
    markdown: bool,

    /// Filter crates
    #[arg(long, default_value_t = false)]
    filtered: bool,

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

        let user_id = match user_res.user {
            Some(u) => u.id,
            None => {
                let msg = format!("User '{}' not found on crates.io.", self.username);
                eprintln!("{}", msg);
                return Err(GeneralError::new(msg));
            }
        };
        if verbose {
            println!(
                "Fetching crates for user '{}' (ID: {})...",
                self.username, user_id
            );
        }
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
        Ok(all_crates)
    }

    /// Format a table to markdown
    /// # Errors
    /// Fails if fmt error
    pub fn format_table<I>(table: I) -> Result<String, std::fmt::Error>
    where
        I: Iterator<Item = Vec<String>> + Clone,
    {
        let mut buf = String::new();
        let max_sizes = table.clone().fold([0usize; 3], |mut acc, row| {
            for (i, cell) in row.iter().enumerate() {
                acc[i] = acc[i].max(cell.len());
            }
            acc
        });

        for (i, row) in table.enumerate() {
            let line = row
                .iter()
                .zip(max_sizes)
                .map(|(s, width)| format!("{:width$}", s, width = width))
                .collect::<Vec<_>>()
                .join(" | ");
            writeln!(&mut buf, "| {} |", line)?;

            // separator after header
            if i == 0 {
                let sep = max_sizes
                    .iter()
                    .map(|&w| "-".repeat(w))
                    .collect::<Vec<_>>()
                    .join(" | ");
                writeln!(&mut buf, "| {} |", sep)?;
            }
        }
        Ok(buf)
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

        if self.filtered {
            for row in rows {
                let to_push = row[1..].to_vec();
                if specials_crates.contains(&row[0]) {
                    table3.push(to_push)
                } else if row[2].to_lowercase().starts_with("incoming") {
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
        let table1_markdown = Self::format_table(table1)?;
        write!(&mut buf, "{}", table1_markdown)?;
        if !table2.is_empty() {
            writeln!(&mut buf, "\n## Incoming\n")?;
            let table2 = header.clone().into_iter().chain(table2);
            let table2_markdown = Self::format_table(table2)?;
            write!(&mut buf, "{}", table2_markdown)?;
        }
        if !table3.is_empty() {
            writeln!(&mut buf, "\n## Others\n")?;
            let table3 = header.into_iter().chain(table3);
            let table3_markdown = Self::format_table(table3)?;
            write!(&mut buf, "{}", table3_markdown)?;
        }
        Ok(buf)
    }

    /// Get the music file path
    /// # Errors
    /// Fails if the file cannot be found
    pub fn list_crates(&self, _config: &mut Config) -> Result<(), GeneralError> {
        let delay = (self.delay * 1000.0) as u64;
        let need_full = self.full;
        let verbose = self.output_file != "-";

        let all_crates = self.get_all_crates(verbose, delay)?;
        if !need_full {
            let stringify = serde_json::to_string_pretty(&all_crates)?;
            if self.output_file == "-" {
                println!("{}", stringify);
            } else {
                fs::write(&self.output_file, stringify)?;
                println!("Written to {}", self.output_file);
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
        if !self.markdown {
            let stringify = serde_json::to_string_pretty(&all_crates_infos)?;
            if self.output_file == "-" {
                println!("{}", stringify);
                return Ok(());
            }
            fs::write(&self.output_file, stringify)?;
            println!("Written to {}", self.output_file);
            return Ok(());
        }
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
                format!("<{}>", h)
            } else {
                default_text.clone()
            };
            let url = if let Some(repo) = repository {
                format!("<{}>", repo)
            } else {
                default_text.clone()
            };
            let docs = if let Some(doc) = documentation {
                format!("<{}>", doc)
            } else {
                default_text.clone()
            };
            let infos = format!("{homepage} <br/> {url} <br/> {docs}");
            [name.clone(), name_with_url, desc, infos]
        });
        let tables = self.generate_markdown_table(rows)?;
        let mut buf = String::new();
        writeln!(
            &mut buf,
            "# crates\n\n- <https://crates.io/users/{}>\n\n## Crates\n",
            self.username
        )?;
        write!(&mut buf, "{}", tables)?;
        if self.output_file == "-" {
            print!("{}", buf);
            return Ok(());
        }
        let output_markdown = if self.output_file == get_default_output_file() {
            "README.md".to_string()
        } else {
            self.output_file.clone()
        };
        fs::write(&output_markdown, buf)?;
        println!("Written to {}", output_markdown);
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
