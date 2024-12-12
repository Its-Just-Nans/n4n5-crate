use std::path::PathBuf;

use crate::{commands::movies::Movies, config::Config};
use clap::{arg, command, value_parser, ArgMatches, Command};

/// A trait for CLI commands
pub(crate) trait CliCommand {
    /// Invoke the command
    fn invoke(config: &mut Config, args_matches: &ArgMatches);

    /// Get the subcommand
    fn get_subcommand() -> Command;
}

/// The CLI main function
/// Handle all arguments and invoke the correct command
/// # Panics
/// May panic
pub fn cli_main() {
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
        .arg_required_else_help(true)
        .get_matches();
    let mut config = match matches.get_one::<PathBuf>("config") {
        Some(config_path) => Config::new_from_path(config_path),
        None => Config::new(),
    };
    match matches
        .get_one::<u8>("debug")
        .expect("Counts are defaulted")
    {
        0 => {}
        value => {
            config.set_debug(value);
            println!("Crazy")
        }
    }
    if let Some(matches) = matches.subcommand_matches("movies") {
        Movies::invoke(&mut config, matches);
    }
}

/// Get input from the user
pub fn input() -> String {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    print!("Please enter some text: ");
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

/// Get a valid path from the user
pub fn input_path() -> (PathBuf, String) {
    let s = input();
    let mut path = PathBuf::from(s);
    loop {
        if path.exists() {
            break;
        }
        println!("Path does not exist. Please enter a valid path:");
        let s = input();
        path = PathBuf::from(s);
    }
    (
        path.clone(),
        path.to_str()
            .expect("Cannot convert path to string")
            .to_string(),
    )
}
