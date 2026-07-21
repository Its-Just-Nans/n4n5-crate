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
            "command failed: {} {:?}\n{}",
            cmd,
            args,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("invalid UTF-8 output from {cmd}: {e}"))
}

/// Helper to split lines
fn split_lines(s: &str) -> HashSet<String> {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

impl Commands {
    /// Watching
    /// # Errors
    /// Returns errors if the request fails
    pub(crate) fn watching(debug: bool) -> Result<(), GeneralError> {
        let username = "Its-Just-Nans";


        let watched_raw = run(
            "gh",
            &[
                "api",
                "-X",
                "GET",
                &format!("/users/{username}/subscriptions"),
                "-q",
                ".[].full_name",
                "--paginate",
            ],
            debug
        )?;
        if watched_raw.is_empty(){
            return Err(GeneralError::new(format!("/users/{username}/subscriptions is empty - weird")));
        }
        println!("Repositories you are watching:");
        println!("{watched_raw}");
        let watched = split_lines(&watched_raw);


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
            debug
        )?;

        println!("All your repositories (excluding forks):");
        println!("{all_raw}");
        let all = split_lines(&all_raw);


        let mut not_watching: Vec<_> = all.difference(&watched).cloned().collect();
        not_watching.sort();

        println!("Repositories you own (not forks) but are NOT watching:");
        for repo in not_watching {
            println!("{repo}");
        }

        Ok(())
    }
}
