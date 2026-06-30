//! The CLI module

use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use std::path::PathBuf;

use crate::{config::Config, errors::GeneralError};

use crate::commands::Commands;

/// CLI colors styles
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

/// Example CLI using clap derive and subcommands
#[derive(Parser, Debug)]
#[command(version, name = "n4n5", about = "n4n5 CLI", long_about = None, styles = STYLES)]
pub struct CliArgs {
    /// Sets a custom config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// whether to use input for configuration
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub use_input: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

/// The CLI main function
/// Handle all arguments and invoke the correct command
/// # Errors
/// Returns a [`GeneralError`] if the command fails
pub fn cli_main() -> Result<(), GeneralError> {
    let cli_args = CliArgs::parse();
    let CliArgs {
        command,
        use_input,
        debug,
        config,
    } = cli_args;
    let mut config = Config::try_new(config, debug, use_input)?;
    command.invoke(&mut config)
}
