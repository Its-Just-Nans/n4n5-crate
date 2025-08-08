//! The CLI module
use std::path::PathBuf;

use crate::{
    commands::{gh::lib::Gh, movies::Movies, music::MusicCliCommand, sync::SyncCliCommand},
    config::Config,
    errors::GeneralError,
};
use clap::{arg, command, value_parser, ArgMatches, Command};

/// A trait for CLI commands
pub(crate) trait CliCommand {
    /// Invoke the command
    /// # Errors
    /// Returns a GeneralError if the command fails
    fn invoke(config: &mut Config, args_matches: &ArgMatches) -> Result<(), GeneralError>;

    /// Get the subcommand
    fn get_subcommand() -> Command;
}

/// The CLI main function
/// Handle all arguments and invoke the correct command
/// # Errors
/// Returns a GeneralError if the command fails
pub fn cli_main() -> Result<(), GeneralError> {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(
            -d --debug ... "Turn debugging information on"
        ))
        .subcommand(Movies::get_subcommand())
        .subcommand(SyncCliCommand::get_subcommand())
        .subcommand(Gh::get_subcommand())
        .subcommand(Config::get_subcommand())
        .subcommand(MusicCliCommand::get_subcommand())
        .arg_required_else_help(true)
        .get_matches();
    let mut config = match matches.get_one::<PathBuf>("config") {
        Some(config_path) => Config::new_from_path(config_path.clone())?,
        None => Config::new(),
    };
    match matches
        .get_one::<u8>("debug")
        .ok_or_else(|| GeneralError::new("Debug value not found".to_string()))?
    {
        0 => {}
        value => {
            config.set_debug(*value);
        }
    }
    if let Some(matches) = matches.subcommand_matches("movies") {
        return Movies::invoke(&mut config, matches);
    } else if let Some(matches) = matches.subcommand_matches("sync") {
        return SyncCliCommand::invoke(&mut config, matches);
    } else if let Some(matches) = matches.subcommand_matches("gh") {
        return Gh::invoke(&mut config, matches);
    } else if let Some(matches) = matches.subcommand_matches("config") {
        return Config::invoke(&mut config, matches);
    } else if let Some(matches) = matches.subcommand_matches("music") {
        return MusicCliCommand::invoke(&mut config, matches);
    }
    Ok(())
}

/// Get input from the user with a prompt
pub fn get_input(text: &str) -> String {
    println!("{text}");
    input()
}

/// Get input from the user
/// # Panics
/// Panics if the input is not a valid string
pub fn input() -> String {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

/// Get a yes input from the user
pub fn input_yes() -> bool {
    let s = input();
    matches!(s.to_lowercase().as_str(), "y" | "yes")
}

/// Get a no input from the user
pub fn input_no() -> bool {
    !input_yes()
}

/// Get a valid path from the user
/// # Panics
/// Panics if the path is not valid
/// # Errors
/// Returns a GeneralError if the path does not exist
pub fn input_path() -> Result<(PathBuf, String), GeneralError> {
    let mut s = input();
    let mut path = PathBuf::from(&s);
    loop {
        if s == "\\" {
            return Err(GeneralError::new("no path".to_string()));
        }
        if path.exists() {
            break;
        }
        println!("Path does not exist. Please enter a valid path:");
        s = input();
        path = PathBuf::from(&s);
    }
    Ok((
        path.clone(),
        path.to_str()
            .expect("Cannot convert path to string")
            .to_string(),
    ))
}
