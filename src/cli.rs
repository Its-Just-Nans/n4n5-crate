//! The CLI module

use std::path::PathBuf;

use crate::{
    commands::{
        config::ConfigSubcommand, gh::lib::GhSubCommand, helpers::HelpersSubcommand,
        movies::MoviesSubCommand, music::MusicSubcommand, sync::SyncSubcommand,
        utils::lib::UtilsSubCommand,
    },
    config::Config,
    errors::GeneralError,
};
use clap::{arg, command, Parser, Subcommand};

/// Example CLI using clap derive and subcommands
#[derive(Parser, Debug)]
#[command(name = "n4n5")]
#[command(about = "n4n5 CLI", long_about = None)]
pub struct CliArgs {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// utils subcommand
    Utils {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: UtilsSubCommand,
    },

    /// music subcommand
    Music {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: MusicSubcommand,
    },

    /// config subcommand
    Config {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },

    /// gh subcommand
    Gh {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: GhSubCommand,
    },

    /// helpers subcommand
    Helpers {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: HelpersSubcommand,
    },
    /// movies subcommand
    Movies {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: MoviesSubCommand,
    },

    /// sync subcommand
    Sync {
        /// list of subcommands
        #[command(subcommand)]
        subcommand: SyncSubcommand,
    },
}

/// The CLI main function
/// Handle all arguments and invoke the correct command
/// # Errors
/// Returns a GeneralError if the command fails
pub fn cli_main() -> Result<(), GeneralError> {
    let cli = CliArgs::parse();
    let mut config = match cli.config {
        Some(config_path) => Config::new_from_path(config_path.clone())?,
        None => Config::new(),
    };
    config.set_debug(cli.debug);
    match cli.command {
        Commands::Utils { subcommand } => subcommand.invoke(&mut config),
        Commands::Music { subcommand } => subcommand.invoke(&mut config),
        Commands::Config { subcommand } => subcommand.invoke(&mut config),
        Commands::Gh { subcommand } => subcommand.invoke(&mut config),
        Commands::Helpers { subcommand } => subcommand.invoke(&mut config),
        Commands::Movies { subcommand } => subcommand.invoke(&mut config),
        Commands::Sync { subcommand } => subcommand.invoke(&mut config),
    }
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
