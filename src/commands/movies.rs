//! To see all subcommands, run:
//! ```shell
//! n4n5 movies
//! ```
//!
use std::{collections::BTreeMap, fs::read_to_string, path::PathBuf};

use clap::{arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};

use crate::{
    cli::{input_path, CliCommand},
    config::Config,
    config_path,
};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct Movies {
    /// Path to the movies file
    pub file_path: Option<String>,

    /// public path to the movies file
    pub public_file_path: Option<String>,
}

/// All movies data
pub struct AllMovies {
    /// List of movies
    pub movies: Vec<OneMovie>,
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
            .subcommand(
                Command::new("sync").about("sync movies file").arg(
                    arg!(
                        -j --json ... "print as json"
                    )
                    .required(false),
                ),
            )
            .arg_required_else_help(true)
    }

    fn invoke(config: &mut Config, args_matches: &ArgMatches) {
        if let Some(matches) = args_matches.subcommand_matches("show") {
            Movies::print_sorted_movies(config, matches);
        } else if let Some(matches) = args_matches.subcommand_matches("stats") {
            Movies::print_stats(config, matches);
        } else if let Some(matches) = args_matches.subcommand_matches("sync") {
            Movies::sync_movies(config, Some(matches));
        }
    }
}

impl Movies {
    /// Get Movie path
    pub fn get_movie_path(config: &mut Config) -> PathBuf {
        config_path!(config, movies, Movies, file_path, "movies file")
    }

    /// Get all movies
    pub fn get_all_movies(config: &mut Config) -> AllMovies {
        let file_path = Movies::get_movie_path(config);
        if config.debug > 0 {
            println!("Reading movies file at {}", file_path.display());
        }
        let movies_file_to_str = read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Unable to read movies file at {}", file_path.display()));
        let all_movies: Vec<OneMovie> =
            serde_json::from_str(&movies_file_to_str).expect("Unable to parse movies file");
        AllMovies { movies: all_movies }
    }

    /// Print the movies sorted by note
    fn print_sorted_movies(config: &mut Config, matches: &ArgMatches) {
        let mut movies = Movies::get_all_movies(config).movies;
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

    /// Group movies by date
    fn group_movies_by_date(movies: &AllMovies) -> std::collections::HashMap<u64, Vec<&OneMovie>> {
        movies
            .movies
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, movie| {
                let entry: &mut Vec<&OneMovie> = acc.entry(movie.date).or_default();
                entry.push(movie);
                acc
            })
    }

    /// Get the stats of the movies
    fn get_stats(movies: &AllMovies) -> (u64, u64, f64, f64) {
        // calculate the min date
        let min_date = movies.movies.iter().map(|m| m.date).min().unwrap();
        // calculate the max date
        let max_date = movies.movies.iter().map(|m| m.date).max().unwrap();
        // calculate the average note
        let avg_note =
            movies.movies.iter().map(|m| m.note).sum::<f64>() / movies.movies.len() as f64;
        // calculate the median note
        let mut notes = movies.movies.iter().map(|m| m.note).collect::<Vec<f64>>();
        notes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_note = notes[notes.len() / 2];

        (min_date, max_date, avg_note, median_note)
    }

    /// Print the stats of the movies
    fn print_stats(config: &mut Config, matches: &ArgMatches) {
        let movies = Movies::get_all_movies(config);
        let (min_date, max_date, avg_note, median_note) = Movies::get_stats(&movies);
        let is_json = !matches!(
            matches.get_one::<u8>("json").expect("Counts are defaulted"),
            0
        );
        if is_json {
            let stats = serde_json::json!({
                "movies": movies.movies.len(),
                "min_date": min_date,
                "max_date": max_date,
                "avg_note": avg_note,
                "median_note": median_note,
            });
            println!("{}", stats);
        } else {
            println!("Number of movies: {}", movies.movies.len());
            println!("Min date: {}", min_date);
            println!("Max date: {}", max_date);
            println!("Average note: {:.3}", avg_note);
            println!("Median note: {:.3}", median_note);
        }
    }

    /// Sync the public movie file
    pub fn sync_movies(config: &mut Config, opt_matches: Option<&ArgMatches>) {
        let movies = Movies::get_all_movies(config);
        let public_movies_path = config_path!(
            config,
            movies,
            Movies,
            public_file_path,
            "the public file for movies"
        );
        let is_json = match opt_matches {
            Some(matches) => !matches!(
                matches.get_one::<u8>("json").expect("Counts are defaulted"),
                0
            ),
            None => false,
        };
        let movies_by_date = Movies::group_movies_by_date(&movies);
        // create an hashmap with the date as key and the movies number for that date as value
        let movie_by_date_count: std::collections::HashMap<u64, u64> = movies_by_date
            .iter()
            .map(|(date, movies)| (*date, movies.len() as u64))
            .collect();
        // sort the hashmap by date
        let movie_by_date_count = BTreeMap::from_iter(movie_by_date_count);

        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        movie_by_date_count
            .serialize(&mut ser)
            .expect("Unable to serialize movies");
        if is_json {
            println!(
                "{}",
                String::from_utf8(buf).expect("Unable to convert to string")
            );
        } else {
            std::fs::write(&public_movies_path, buf).expect("Unable to write movies file");
            println!("Movies file saved to {:?}", public_movies_path);
        }
    }
}
