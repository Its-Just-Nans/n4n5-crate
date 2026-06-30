//! man commands

use clap::CommandFactory;
use clap_mangen::generate_to as man_generate_to;
use home::home_dir;
use std::fs::create_dir_all;

use crate::cli::CliArgs;
use crate::commands::Commands;
use crate::config::Config;
use crate::errors::GeneralError;

impl Commands {
    /// generate man page
    /// # Errors
    /// Fails if error
    pub fn gen_man(_config: &mut Config) -> Result<(), GeneralError> {
        let cmd = CliArgs::command();
        let outdir = home_dir().ok_or(GeneralError::new("Cannot get home dir"))?;
        let outdir = outdir.join(".config").join(".n4n5").join("man");
        create_dir_all(&outdir)?;

        man_generate_to(cmd, &outdir)?;
        println!(
            "Generated man to {}{}",
            outdir.display(),
            std::path::MAIN_SEPARATOR
        );
        Ok(())
    }
}
