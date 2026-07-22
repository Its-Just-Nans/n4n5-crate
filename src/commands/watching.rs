//! Watching command

use std::collections::HashSet;
use std::process::{Command, Output};

use crate::commands::Commands;
use crate::errors::GeneralError;

/// Helper to run a `Command`
/// # Errors
/// Return error if the command fails
fn run(cmd: &str, args: &[&str], debug: bool) -> Result<String, String> {
    if debug {
        println!("{} {}", cmd, args.join(" "));
    }
    let output: Output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to execute {cmd}: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "command failed: {} {}\n{}",
            cmd,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("invalid UTF-8 output from {cmd}: {e}"))
}

impl Commands {
    /// Watching
    /// # Errors
    /// Returns errors if the request fails
    pub(crate) fn watching(debug: bool) -> Result<(), GeneralError> {
        let username = "Its-Just-Nans";

        let all_raw = run(
            "gh",
            &[
                "repo",
                "list",
                username,
                "--limit",
                "10000000",
                "--source",
                "--json",
                "nameWithOwner,isFork",
                "-q",
                ".[] | select(.isFork==false) | .nameWithOwner",
            ],
            debug,
        )?;

        let mut not_watching = HashSet::new();
        let prefix = format!("{username}/");
        for repo_link in all_raw.lines() {
            let repo = repo_link.strip_prefix(&prefix).unwrap_or(repo_link);

            let subscribers: String = run(
                "gh",
                &[
                    "api",
                    &format!("/repos/{username}/{repo}/subscribers"),
                    "--paginate",
                    "-q",
                    ".[].login",
                ],
                debug,
            )?;

            let owner_is_subscriber = subscribers.lines().any(|login| login == username);

            println!("{repo_link:<45} {owner_is_subscriber}");
            if !owner_is_subscriber {
                not_watching.insert(repo_link.to_string());
            }
        }

        println!();
        println!("Repositories you own (not forks) but are NOT watching:");
        for repo in not_watching {
            println!("{repo}");
        }

        Ok(())
    }
}
