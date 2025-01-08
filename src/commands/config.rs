//! Config subcommand
use std::process::Command;

use clap::{arg, ArgAction, ArgMatches, Command as ClapCommand};

use crate::{cli::CliCommand, config::Config};

impl CliCommand for Config {
    fn get_subcommand() -> ClapCommand {
        ClapCommand::new("config")
            .about("config subcommand")
            .subcommand(
                ClapCommand::new("open")
                    .about("open config with default editor")
                    .arg(
                        arg!(
                            -p --path "Print the path"
                        )
                        .action(ArgAction::SetTrue)
                        .required(false),
                    ),
            )
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
        if let Some(matches) = args_matches.subcommand_matches("open") {
            Config::open(config, matches);
        }
    }
}

impl Config {
    /// Open the config file with the default editor
    /// # Panics
    /// Panics if the editor returns a non-zero status
    fn open(config: &mut Config, matches: &ArgMatches) {
        let config_path = &config.config_path;
        let only_path = matches.get_flag("path");
        if only_path {
            println!("{}", config_path.display());
            return;
        }
        println!("Opening config {}", config_path.display());
        Command::new("vi")
            .arg(config_path)
            .spawn()
            .expect("Unable to open config with default editor")
            .wait()
            .expect("Error: Editor returned a non-zero status");
    }
}
