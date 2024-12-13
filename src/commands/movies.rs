//! To see all subcommands, run:
//! ```shell
//! n4n5 movies
//! ```
//!
use std::{fs::read_to_string, path::PathBuf};

use clap::{arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct Movies {
    /// Path to the movies file
    pub file_path: Option<String>,
}

/// Movie data
#[derive(Deserialize, Debug)]
pub struct OneMovie {
    /// Movie title
    pub title: String,

    /// Movie note
    pub note: f64,

    /// Movie publication date
    pub date: u64,

    /// Comment about the movie
    pub comment: String,

    /// Seen date
    pub seen: Option<String>,

    /// Summary of the movie
    pub summary: Option<String>,
}

impl OneMovie {
    /// Create a new movie
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

    /// Display the movie
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
            .subcommand(
                Command::new("stats").about("show stats about movies").arg(
                    arg!(
                        -j --json ... "print as json"
                    )
                    .required(false),
                ),
            )
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

    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
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
                    if let Some(movies) = config_data.movies.as_mut() {
                        movies.file_path = Some(cloned_path);
                    } else {
                        config_data.movies = Some(Movies {
                            file_path: Some(cloned_path),
                        });
                    }
                    config_data
                });
                file_path.0
            }
        };
        let movies_file_to_str = read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Unable to read movies file at {}", file_path.display()));
        let mut all_movies: Vec<OneMovie> =
            serde_json::from_str(&movies_file_to_str).expect("Unable to parse movies file");
        if let Some(matches) = args_matches.subcommand_matches("show") {
            Movies::print_sorted_movies(&mut all_movies, matches);
        } else if let Some(matches) = args_matches.subcommand_matches("stats") {
            Movies::print_stats(&all_movies, matches);
        }
    }
}

impl Movies {
    /// Print the movies sorted by note
    fn print_sorted_movies(movies: &mut Vec<OneMovie>, matches: &ArgMatches) {
        let reverse = !matches!(
            matches
                .get_one::<u8>("reverse")
                .expect("Counts are defaulted"),
            0
        );
        let show_comment = !matches!(
            matches
                .get_one::<u8>("comment")
                .expect("Counts are defaulted"),
            0
        );
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

    fn get_stats(movies: &Vec<OneMovie>) -> (u64, u64, f64, f64, Vec<(u64, usize)>) {
        // calculate the min date
        let min_date = movies.iter().map(|m| m.date).min().unwrap();
        // calculate the max date
        let max_date = movies.iter().map(|m| m.date).max().unwrap();
        // calculate the average note
        let avg_note = movies.iter().map(|m| m.note).sum::<f64>() / movies.len() as f64;
        // calculate the median note
        let mut notes = movies.iter().map(|m| m.note).collect::<Vec<f64>>();
        notes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_note = notes[notes.len() / 2];

        // group movies by date
        let movies_by_date =
            movies
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, movie| {
                    let entry = acc.entry(movie.date).or_insert_with(Vec::new);
                    entry.push(movie);
                    acc
                });
        let movies_by_date = movies_by_date
            .iter()
            .map(|(date, movies)| (*date, movies.len()))
            .collect::<Vec<(u64, usize)>>();
        return (min_date, max_date, avg_note, median_note, movies_by_date);
    }

    fn print_stats(movies: &Vec<OneMovie>, _matches: &ArgMatches) {
        let (min_date, max_date, avg_note, median_note, movies_by_date) = Movies::get_stats(movies);
        match _matches
            .get_one::<u8>("json")
            .expect("Counts are defaulted")
        {
            0 => {
                println!("Number of movies: {}", movies.len());
                println!("Min date: {}", min_date);
                println!("Max date: {}", max_date);
                println!("Average note: {:.3}", avg_note);
                println!("Median note: {:.3}", median_note);
            }
            _ => {
                println!(
                    "{}",
                    serde_json::to_string(&movies_by_date).expect("Unable to serialize")
                );
            }
        }
    }
}
