use std::{fs::read_to_string, path::PathBuf};

use clap::{arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
};

#[derive(Deserialize, Serialize, Default)]
pub struct Movies {
    pub file_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OneMovie {
    pub title: String,
    pub note: f64,
    pub date: u64,
    pub comment: String,
    pub seen: Option<String>,
    pub summary: Option<String>,
}

impl OneMovie {
    pub fn new(
        title: String,
        note: f64,
        date: u64,
        comment: String,
        seen: Option<String>,
        summary: Option<String>,
    ) -> Self {
        OneMovie {
            title,
            seen,
            note,
            date,
            comment,
            summary,
        }
    }
    pub fn display(&self, show_comment: bool) -> String {
        if show_comment {
            format!(
                "{} - {} ({}) - {}",
                self.note, self.title, self.date, self.comment
            )
        } else {
            format!("{} - {} ({}) ", self.note, self.title, self.date,)
        }
    }
}

impl CliCommand for Movies {
    fn get_subcommand() -> clap::Command {
        Command::new("movies")
            .about("movies subcommand")
            .subcommand(Command::new("add").about("adds a movie"))
            .subcommand(Command::new("stats").about("show stats about movies"))
            .subcommand(
                Command::new("show")
                    .about("removes a movie")
                    .arg(
                        arg!(
                            -r --reverse ... "Reverse order"
                        )
                        .required(false),
                    )
                    .arg(
                        arg!(
                            -c --comment ... "Show comment"
                        )
                        .required(false),
                    ),
            )
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, _args_matches: &ArgMatches) {
        let file_path = match &config.config_data.movies {
            Some(movies_config) => movies_config.file_path.clone(),
            None => {
                eprintln!("No file path set in config");
                None
            }
        };
        let file_path = match file_path {
            Some(path) => PathBuf::from(path),
            None => {
                eprintln!("No file path set in config");
                println!("Enter a file path:");
                let file_path = input_path();
                let cloned_path = file_path.1.clone();
                config.update(|config_data| {
                    if config_data.movies.is_none() {
                        config_data.movies = Some(Movies {
                            file_path: Some(cloned_path),
                        });
                    } else {
                        config_data.movies.as_mut().map(|movies| {
                            movies.file_path = Some(cloned_path);
                        });
                    }
                    config_data
                });
                file_path.0
            }
        };
        let movies_file_to_str = read_to_string(&file_path)
            .expect(format!("Unable to read movies file at {}", file_path.display()).as_str());
        let mut all_movies: Vec<OneMovie> =
            serde_json::from_str(&movies_file_to_str).expect("Unable to parse movies file");
        if let Some(matches) = _args_matches.subcommand_matches("show") {
            Movies::print_sorted_movies(&mut all_movies, matches);
        }
    }
}

impl Movies {
    fn print_sorted_movies(movies: &mut Vec<OneMovie>, matches: &ArgMatches) {
        let reverse = match matches
            .get_one::<u8>("reverse")
            .expect("Counts are defaulted")
        {
            0 => false,
            _ => true,
        };
        let show_comment = match matches
            .get_one::<u8>("comment")
            .expect("Counts are defaulted")
        {
            0 => false,
            _ => true,
        };
        movies.sort_by(|a, b| {
            if reverse {
                b.note.partial_cmp(&a.note).unwrap()
            } else {
                a.note.partial_cmp(&b.note).unwrap()
            }
        });
        for movie in movies {
            println!("{}", movie.display(show_comment));
        }
    }
}