//! To see all subcommands, run:
//! ```shell
//! n4n5 movies
//! ```
//!
use std::{collections::BTreeMap, fs::read_to_string, path::PathBuf, process::Command};

use clap::{ArgAction, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    config_path,
    errors::GeneralError,
    utils::{get_input, input_path},
};

/// Movies configuration
#[derive(Deserialize, Serialize, Default)]
pub struct Movies {
    /// Path to the movies file
    pub file_path: Option<String>,

    /// public path to the movies file
    pub public_file_path: Option<String>,
}

/// Display mode
pub enum DisplayMode {
    /// Short display
    Short,

    /// Display with comment
    Comment,

    /// Full display
    Full,
}

/// All movies data
pub struct AllMovies {
    /// List of movies
    pub movies: Vec<OneMovie>,
}

impl AllMovies {
    /// Display the movies
    pub fn display(&self, mode: DisplayMode) {
        for movie in &self.movies {
            match mode {
                DisplayMode::Short => println!("{}", movie.display()),
                DisplayMode::Comment => println!("{}", movie.display_comment()),
                DisplayMode::Full => println!("{}\n", movie.display_full()),
            }
        }
    }
}

/// Movie data
#[derive(Deserialize, Serialize, Debug)]
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
    /// Display the movie
    pub fn display(&self) -> String {
        format!("{} - {} ({}) ", self.note, self.title, self.date,)
    }

    /// Display the movie with comment
    pub fn display_comment(&self) -> String {
        format!(
            "{} - {} ({}) - {}",
            self.note, self.title, self.date, self.comment
        )
    }

    /// Display the full movie
    pub fn display_full(&self) -> String {
        format!(
            "{} - {} ({}) - {} - {}\n{}",
            self.note,
            self.title,
            self.date,
            self.seen.as_deref().unwrap_or(""),
            self.comment,
            self.summary.as_deref().unwrap_or("")
        )
    }
}

/// Movies sub command
#[derive(Subcommand, Debug)]
pub enum MoviesSubCommand {
    /// add a movie
    Add,
    /// open movie file
    Open {
        /// print path of movies file
        #[arg(short = 'p', long = "path", action = ArgAction::SetTrue)]
        show_path: bool,
    },
    /// Show stats of movies
    Stats {
        /// print stats as json
        #[arg(short ='j', long = "json", action = ArgAction::SetTrue)]
        print_json: bool,
    },
    /// Show movies list
    Show {
        /// reverse mode
        #[arg(short = 'r', long = "reverse", action = ArgAction::SetTrue)]
        reverse: bool,
        /// show full mode
        #[arg(short = 'f', long = "full", action = ArgAction::SetTrue)]
        show_full: bool,
        /// show comment
        #[arg(short = 'c', long = "comment", action = ArgAction::SetTrue)]
        show_comment: bool,
    },
    /// Sync movies file
    Sync {
        /// print as json
        #[arg(short ='j', long = "json", action = ArgAction::SetTrue)]
        print_json: bool,
    },
}

impl MoviesSubCommand {
    /// invoke subcommand
    /// # Errors
    /// Error if error in subcommand
    pub fn invoke(self, config: &mut Config) -> Result<(), GeneralError> {
        match self {
            Self::Add => Movies::add_movie(config),
            Self::Open { show_path } => Movies::open_movies(config, show_path),
            Self::Show {
                reverse,
                show_full,
                show_comment,
            } => Movies::print_sorted_movies(config, reverse, show_comment, show_full),
            Self::Stats { print_json } => Movies::print_stats(config, print_json),
            Self::Sync { print_json } => Movies::sync_movies(config, print_json),
        }
    }
}

impl Movies {
    /// Get the movie path
    /// # Errors
    /// Returns an error if unable to read the movies file
    pub fn get_movie_path(config: &mut Config) -> Result<PathBuf, GeneralError> {
        let path = config_path!(config, movies, Movies, file_path, "movies file");
        Ok(path)
    }

    /// Add a movie
    /// # Errors
    /// Returns an error if unable to read the movies file
    fn add_movie(config: &mut Config) -> Result<(), GeneralError> {
        let file_path = Movies::get_movie_path(config)?;
        let title = get_input("Title")?;
        let note = get_input("Note")?.parse()?;
        let date = get_input("Date")?.parse()?;
        let comment = get_input("Comment")?;
        let seen = get_input("Seen")?;
        let summary = get_input("Summary")?;
        let movie = OneMovie {
            title,
            note,
            date,
            comment,
            seen: Some(seen),
            summary: Some(summary),
        };
        let mut all_movies = Movies::get_all_movies(config)?;
        all_movies.movies.push(movie);
        let movies_file_to_str = serde_json::to_string_pretty(&all_movies.movies)?;
        std::fs::write(&file_path, movies_file_to_str)?;
        println!("Movie added to '{}'", file_path.display());
        Ok(())
    }

    /// Open movie file
    /// # Errors
    /// Returns an error if unable to open the movies file
    pub fn open_movies(config: &mut Config, show_path: bool) -> Result<(), GeneralError> {
        let file_path = Movies::get_movie_path(config)?;
        if show_path {
            println!("{}", file_path.display());
            return Ok(());
        }
        println!("Opening movies file at {}", file_path.display());
        Command::new("vi").arg(&file_path).spawn()?.wait()?;
        Ok(())
    }

    /// Get all movies
    /// # Errors
    /// Returns an error if unable to read the movies file
    pub fn get_all_movies(config: &mut Config) -> Result<AllMovies, GeneralError> {
        let file_path = Movies::get_movie_path(config)?;
        if config.debug > 0 {
            println!("Reading movies file at {}", file_path.display());
        }
        if !file_path.exists() {
            return Err(GeneralError::new(format!(
                "Movies file not found at '{}'",
                file_path.display()
            )));
        }
        if !file_path.is_file() {
            return Err(GeneralError::new(format!(
                "Movies file is not a file at '{}'",
                file_path.display()
            )));
        }
        let movies_file_to_str = read_to_string(&file_path)?;
        let all_movies: Vec<OneMovie> = serde_json::from_str(&movies_file_to_str)?;
        Ok(AllMovies { movies: all_movies })
    }

    /// Print the movies sorted by note
    /// # Errors
    /// Returns an error if unable to read the movies file
    fn print_sorted_movies(
        config: &mut Config,
        reverse: bool,
        show_comment: bool,
        show_full: bool,
    ) -> Result<(), GeneralError> {
        let mut all_movies = Movies::get_all_movies(config)?;
        all_movies.movies.sort_by(|a, b| {
            if reverse {
                b.note
                    .partial_cmp(&a.note)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.note
                    .partial_cmp(&b.note)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });
        if show_full {
            all_movies.display(DisplayMode::Full);
        } else if show_comment {
            all_movies.display(DisplayMode::Comment);
        } else {
            all_movies.display(DisplayMode::Short);
        }
        Ok(())
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
        let min_date = movies.movies.iter().map(|m| m.date).min().unwrap_or(0);
        // calculate the max date
        let max_date = movies.movies.iter().map(|m| m.date).max().unwrap_or(0);
        // calculate the average note
        let avg_note =
            movies.movies.iter().map(|m| m.note).sum::<f64>() / movies.movies.len() as f64;
        // calculate the median note
        let mut notes = movies.movies.iter().map(|m| m.note).collect::<Vec<f64>>();
        notes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_note = notes[notes.len() / 2];

        (min_date, max_date, avg_note, median_note)
    }

    /// Print the stats of the movies
    /// # Errors
    /// Returns an error if unable to read the movies file
    fn print_stats(config: &mut Config, is_json: bool) -> Result<(), GeneralError> {
        let movies = Movies::get_all_movies(config)?;
        let (min_date, max_date, avg_note, median_note) = Movies::get_stats(&movies);
        if is_json {
            let stats = serde_json::json!({
                "movies": movies.movies.len(),
                "min_date": min_date,
                "max_date": max_date,
                "avg_note": avg_note,
                "median_note": median_note,
            });
            println!("{stats}");
        } else {
            println!("Number of movies: {}", movies.movies.len());
            println!("Min date: {min_date}");
            println!("Max date: {max_date}");
            println!("Average note: {avg_note:.3}");
            println!("Median note: {median_note:.3}");
        }
        Ok(())
    }

    /// Sync the public movie file
    /// # Errors
    /// Returns an error if unable to read the movies file
    pub fn sync_movies(config: &mut Config, print_json: bool) -> Result<(), GeneralError> {
        if config.debug > 1 {
            println!("Syncing movies");
        }
        let movies = Movies::get_all_movies(config)?;
        let public_movies_path = config_path!(
            config,
            movies,
            Movies,
            public_file_path,
            "the public file for movies"
        );
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
        movie_by_date_count.serialize(&mut ser)?;
        if print_json {
            let movies_str = String::from_utf8(buf)?;
            println!("{movies_str}");
        } else {
            std::fs::write(&public_movies_path, buf)?;
            println!("Movies file saved to '{}'", public_movies_path.display());
        }
        Ok(())
    }
}
